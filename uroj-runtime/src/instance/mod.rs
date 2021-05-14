pub(crate) mod exam;
pub(crate) mod fsm;
pub(crate) mod station;
pub(crate) mod topo;

use async_graphql::*;
use log::debug;
use std::{collections::HashMap, sync::Arc, time::Duration};
use strum_macros::*;
use tokio::{
    sync::{
        broadcast::{self, Receiver, Sender},
        Mutex,
    },
};

use serde::{Deserialize, Serialize};
use uroj_db::models::question::Question as QuestionModel;

use crate::raw_station::{RawDirection, RawSignalKind, RawStation};

use self::topo::Topo;
use self::{exam::ExamManager, fsm::*};
use self::{
    exam::UpdateQuestion,
    station::{ButtonKind, LayoutData, NodeData, SignalData},
};

#[derive(Union, Clone, Serialize, Deserialize)]
pub(crate) enum GameFrame {
    UpdateSignal(UpdateSignal),
    UpdateNode(UpdateNode),
    UpdateGlobalStatus(GlobalStatus),
    MoveTrain(MoveTrain),
    UpdateQuestion(UpdateQuestion),
}

impl GameFrame {
    pub(crate) async fn send_via(&self, sender: &Sender<GameFrame>) {
        sender.send(self.clone());
    }
}

pub(crate) type FrameSender = Sender<GameFrame>;

pub struct Instance {
    pub(crate) layout: LayoutData,
    pub(crate) fsm: Arc<Mutex<InstanceFSM>>,
    pub(crate) topo: Arc<Topo>,
    pub(crate) exam: Option<ExamManager>,
    pub(crate) tx: FrameSender,
    pub(crate) _rx: Receiver<GameFrame>,
}

pub(crate) struct PathBtn {
    pub(crate) id: String, //signal or button id
    pub(crate) kind: ButtonKind,
}

impl Instance {
    pub(crate) fn new(cfg: &InstanceConfig) -> Result<Self, String> {
        let signals = &cfg.station.signals;
        let nodes = &cfg.station.nodes;
        let ind_btns = &cfg.station.independent_btns;

        let topo = Topo::new(nodes, signals, ind_btns);

        //創建fsm
        let mut fsm_sgns: HashMap<String, Signal> =
            signals.iter().map(|s| (s.id.clone(), s.into())).collect();
        let mut fsm_nodes: HashMap<NodeID, Node> = nodes.iter().map(|n| (n.id, n.into())).collect();
        let mut stn_sgns: HashMap<_, SignalData> =
            signals.iter().map(|s| (s.id.clone(), s.into())).collect();
        let stn_nodes: HashMap<_, NodeData> = nodes.iter().map(|n| (n.id, n.into())).collect();

        //需要配置stn_sgn 的 dir 和 pos 缺省
        //需要配置fsm_node的left和right，
        //fsm_sgn的dir缺省
        for s in signals {
            let pid = s.protect_node_id;
            let tid = s.toward_node_id;

            let dir = topo
                .direction(pid, tid)
                .or(s.dir.into())
                .ok_or(format!("invalid signal {}", s.id))?;

            let stn_sgn = stn_sgns.get_mut(&s.id).unwrap();
            let fsm_sgn = fsm_sgns.get_mut(&s.id).unwrap();

            let p_node = fsm_nodes
                .get_mut(&pid)
                .ok_or(format!("unknown node id: {}", pid))?;
            let p_node_stn = stn_nodes.get(&pid).ok_or("fuck you bro")?;

            //對信號機進行所屬
            fsm_sgn.dir = dir;
            stn_sgn.dir = dir;
            match dir {
                RawDirection::Left => {
                    stn_sgn.pos = p_node_stn.left_p.clone();
                    p_node.left_sgn_id = Some(s.id.clone());
                }
                RawDirection::Right => {
                    stn_sgn.pos = p_node_stn.right_p.clone();
                    p_node.right_sgn_id = Some(s.id.clone());
                }
            }
        }

        let layout = LayoutData {
            title: cfg.station.title.clone(),
            nodes: stn_nodes.values().cloned().collect(),
            signals: stn_sgns.values().cloned().collect(),
        };

        let fsm = InstanceFSM {
            sgns: fsm_sgns
                .drain()
                .map(|(id, sgn)| (id.clone(), Mutex::new(sgn)))
                .collect(),
            nodes: fsm_nodes
                .drain()
                .map(|(id, node)| (id, Mutex::new(node)))
                .collect(),

            trains: Arc::new(Mutex::new(Vec::new())),
        };

        let exam = if (&cfg.questions).is_empty() {
            None
        } else {
            Some(ExamManager::new(&cfg.questions))
        };

        let (tx, rx) = broadcast::channel(32);
        Ok(Instance {
            fsm: Arc::new(Mutex::new(fsm)),
            topo: Arc::new(topo),
            layout: layout,
            exam: exam,
            tx: tx,
            _rx: rx,
        })
    }

    //パースを作成する
    pub(crate) async fn create_path(
        &self,
        start: PathBtn,
        end: PathBtn,
    ) -> Result<Vec<NodeID>, String> {
        debug!("flag0");
        if start.id == end.id {
            return Err("start and end cannot be same".to_string());
        }
        let topo = &self.topo;
        let fsm = &self.fsm.lock().await;

        let mut start_sgn = fsm.sgn(&start.id).await;

        let (start_node, start_dir) = (start_sgn.protect_node_id, start_sgn.dir.reverse());
        //這裏的方向是根據用戶輸入判斷的朝向，和最終尋到的路徑的前後朝向做判斷
        //使用按鈕類型判斷進路類型
        let (mut is_pass, mut is_send, mut is_recv, mut is_shnt) = (false, false, false, false);

        let (goal_node, goal_dir) = match (start.kind, end.kind) {
            //通過按鈕 -> 列車按鈕 = 通過進路
            (ButtonKind::Pass, ButtonKind::Train) => {
                is_pass = true;
                let end_sgn = fsm.sgn(&end.id).await;
                (end_sgn.protect_node_id, end_sgn.dir.clone())
            }
            //通過按鈕 -> 列車終端按鈕 = 通過進路
            (ButtonKind::Pass, ButtonKind::LZA) => {
                is_pass = true;
                let node_id = self
                    .topo
                    .ind_btn
                    .get(&end.id)
                    .ok_or(format!("unknown button id: {}", &end.id))?
                    .clone();

                (node_id, start_dir.clone())
            }
            //列車按鈕 -> 列車按鈕 = 接發車進路
            (ButtonKind::Train, ButtonKind::Train) => {
                let end_sgn = fsm.sgn(&end.id).await;
                match (&start_sgn.kind, &end_sgn.kind) {
                    //進站信號機 -> 出戰信號機 => 接車進路
                    (RawSignalKind::HomeSignal, RawSignalKind::StartingSignal) => {
                        is_recv = true;
                        (end_sgn.toward_node_id, end_sgn.dir.clone())
                    }
                    //出站信號機 -> 進站信號機 => 發車進路
                    (RawSignalKind::StartingSignal, RawSignalKind::HomeSignal) => {
                        is_send = true;
                        (end_sgn.protect_node_id, end_sgn.dir.clone())
                    }
                    _ => return Err("no route found".to_string()),
                }
            }
            //發車進路
            (ButtonKind::Train, ButtonKind::LZA) => {
                is_send = true;
                match start_sgn.kind {
                    RawSignalKind::StartingSignal => {
                        let node_id = self
                            .topo
                            .ind_btn
                            .get(&end.id)
                            .ok_or(format!("unknown button id: {}", &end.id))?
                            .clone();

                        (node_id, start_dir.clone())
                    }

                    _ => return Err("no route found".to_string()),
                }
            }
            //調車進路
            (ButtonKind::Shunt, ButtonKind::Shunt) => {
                is_shnt = true;
                //注意并置和差置
                let end_id = self
                    .topo
                    .dif_relation
                    .get(&end.id)
                    .or(topo.jux_relation.get(&end.id))
                    .unwrap_or(&end.id);

                let end_sgn = fsm.sgn(end_id).await;

                (end_sgn.toward_node_id, end_sgn.dir.reverse())
            }
            _ => return Err("no route found".to_string()),
        };

        //dir 是檢索到的可用方向
        let (maybe_path, s_dir, g_dir) = self
            .topo
            .available_path(start_node, goal_node)
            .ok_or("no available path exists")?;

        //進路方向 bound
        if s_dir != start_dir || g_dir != goal_dir {
            return Err("no available route exists".to_string());
        }

        //ensure that all nodes are not used or locked by another existing path
        let mut sgn_id = Vec::new();
        debug!("flag1");
        for id in &maybe_path {
            debug!("flag1-{}", id);

            let node = fsm.node(*id).await;
            if node.state != NodeStatus::Vacant {
                debug!("{} is not vacant", id);
                return Err("target path is not vacant".into());
            }
            if node.is_lock {
                debug!("{} is not locked", id);
                return Err("target path is conflicting".into());
            }
            if node.used_count > 0 {
                return Err("target path is mutex".into());
            }

            let sgn = match s_dir {
                RawDirection::Left => node.right_sgn_id.clone(),
                RawDirection::Right => node.left_sgn_id.clone(),
            };
            if let Some(sgn) = sgn {
                sgn_id.push(sgn);
            }
        }

        //锁闭区段
        debug!("flag2");
        for id in &maybe_path {
            debug!("flag2-{}", id);

            let mut node = fsm.node(*id).await;
            debug!("trying to lock: {}", id);
            node.lock(&self.tx).await;
            node.once_occ = false; //重置曾占用flag
            for id in topo.s_graph.neighbors(*id) {
                let mut node = fsm.node(id).await;
                node.used_count += 1;
                debug!("{}.used_count= {}", id, node.used_count);
            }
        }

        if is_recv {
            let kind = fsm.node(goal_node).await.kind.clone();
            start_sgn.open_recv(kind, &self.tx).await;
        }

        if is_pass || is_send {
            start_sgn.open(&self.tx).await;
        }

        if is_shnt {
            start_sgn.open(&self.tx).await;

            for id in sgn_id {
                if id == start_sgn.id {
                    continue;
                }
                let mut sgn = fsm.sgn(&id).await;
                sgn.open(&self.tx).await;
            }
        }

        Ok(maybe_path)
    }

    //進路を消す
    pub(crate) async fn cancel_path(&self, start: PathBtn) -> Result<(), String> {
        let fsm = &self.fsm.lock().await;
        let topo = &self.topo;

        let mut start_sgn = fsm.sgn(&start.id).await;
        if !start_sgn.is_allowed() {
            return Err("not find a existed route".into());
        }

        let (start_node, start_dir) = (start_sgn.protect_node_id, start_sgn.dir.reverse());

        debug!("开始寻径");
        let maybe_route = Self::find_a_route(fsm, topo, start_node, &start_dir)
            .await
            .ok_or("not find a existed route")?;
        debug!("寻得: {:?}", maybe_route.clone());

        let close_node = fsm.node(start_sgn.toward_node_id).await;
        debug!("接近区段: {}", close_node.node_id);

        if close_node.state != NodeStatus::Vacant {
            return Err("approching node is not vacant".into());
        }
        if close_node.is_lock {
            return Err("not a complete route".into());
        }

        //鎖閉始端信號機
        start_sgn.protect(&self.tx).await;

        //解鎖所有節點
        for n in maybe_route {
            let mut node = fsm.node(n).await;

            node.unlock(&self.tx).await;

            let sgn_id = match start_dir {
                RawDirection::Left => node.right_sgn_id.clone(),
                RawDirection::Right => node.left_sgn_id.clone(),
            };

            for id in self.topo.s_graph.neighbors(n) {
                let mut node = fsm.node(id).await;
                node.used_count -= 1; //对扩展集中的点解除征用
            }

            //調車進路鎖閉所有調車信號機
            if let Some(sgn_id) = sgn_id {
                if sgn_id == start.id {
                    continue;
                }
                let mut sgn = fsm.sgn(&sgn_id).await;
                if sgn.kind == RawSignalKind::ShuntingSignal {
                    sgn.protect(&self.tx).await;
                }
            }
        }

        Ok(())
    }

    pub(crate) async fn find_a_route(
        fsm: &InstanceFSM,
        topo: &Topo,
        nid: NodeID,
        dir: &RawDirection,
    ) -> Option<Vec<NodeID>> {
        let curr = fsm.node(nid).await;
        if !curr.is_lock || curr.state != NodeStatus::Vacant {
            return None;
        }
        let mut res = vec![nid];
        while let Some(next) = next_route_node(fsm, topo, &res, dir).await {
            res.push(next);
        }
        Some(res)
    }
}

pub(crate) async fn next_route_node(
    fsm: &InstanceFSM,
    topo: &Topo,
    his: &Vec<NodeID>,
    dir: &RawDirection,
) -> Option<NodeID> {
    let edges = topo.r_graph.edges(*his.last().unwrap());

    for (_, t, d) in edges {
        if d == dir && !his.contains(&t) {
            let to = fsm.node(t).await;
            if to.is_lock && to.state == NodeStatus::Vacant {
                return Some(t);
            }
        }
    }
    None
}

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct InstanceConfig {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) player: String,
    pub(crate) station: RawStation,
    pub(crate) questions: HashMap<i32, QuestionModel>,
    pub(crate) token: String,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Deserialize, Serialize, Debug)]
pub(crate) enum InstanceKind {
    Exam,
    Chain,
    Exercise,
}

#[derive(Eq, PartialEq, Display, EnumString, Deserialize, Serialize, Enum, Copy, Clone)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum InstanceStatus {
    Prestart, //启动前
    Playing,  //使用中
    Finished, //已结束
}

pub type InstancePool = HashMap<String, Instance>;
