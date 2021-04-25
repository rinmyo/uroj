pub(crate) mod fsm;
pub(crate) mod graph;
pub(crate) mod station;
pub(crate) mod train;

use rdkafka::producer::FutureProducer;
use std::{collections::HashMap, sync::Arc};
use strum_macros::*;
use tokio::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::raw_station::{RawDirection, RawSignalKind, RawStation};

use self::{fsm::*, station::Point};
use self::graph::StationGraph;
use self::station::{ButtonKind, NodeData, SignalData, StationData};
use self::train::Train;

pub struct Instance {
    pub(crate) fsm: Arc<Mutex<InstanceFSM>>,
    pub(crate) graph: StationGraph,
    pub(crate) trains: Vec<Train>,
    pub(crate) station: StationData,
    pub(crate) dif_relation: HashMap<String, String>,
    pub(crate) jux_relation: HashMap<String, String>, //signal -> signal
    pub(crate) ind_btn: HashMap<String, NodeID>,      //按鈕ID -> 防護節點ID
}

pub(crate) struct PathBtn {
    pub(crate) id: String, //signal or button id
    pub(crate) kind: ButtonKind,
}

impl Instance {
    pub(crate) fn new(cfg: &InstanceConfig) -> Result<Self, String> {
        let signals = &cfg.station.signals;
        let nodes = &cfg.station.nodes;

        let graph = StationGraph::new(nodes);

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

            let dir = graph
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

        let station = StationData {
            title: cfg.station.title.clone(),
            nodes: stn_nodes.values().cloned().collect(),
            signals: stn_sgns.values().cloned().collect(),
        };

        let fsm = InstanceFSM {
            sgns: fsm_sgns,
            nodes: fsm_nodes,
        };

        let jux_relation = signals
            .iter()
            .filter_map(|s| s.jux_sgn.as_ref().map(|k| (s.id.clone(), k.clone())))
            .collect();

        let dif_relation = signals
            .iter()
            .filter_map(|s| s.dif_sgn.as_ref().map(|k| (s.id.clone(), k.clone())))
            .collect();

        let ind_btn = cfg
            .station
            .independent_btns
            .iter()
            .map(|b| (b.id.clone(), b.protect_node_id))
            .collect();

        Ok(Instance {
            fsm: Arc::new(Mutex::new(fsm)),
            graph: graph,
            dif_relation: dif_relation,
            jux_relation: jux_relation,
            ind_btn: ind_btn,
            trains: Vec::new(),
            station: station,
        })
    }

    //パースを作成する
    pub(crate) async fn create_path(
        &self,
        start: PathBtn,
        end: PathBtn,
        producer: &FutureProducer,
    ) -> Result<Vec<NodeID>, String> {
        let arc_fsm = self.fsm.clone();
        let mut fsm = arc_fsm.lock().await;

        let graph = &self.graph;

        let start_sgn = fsm
            .sgns
            .get(&start.id)
            .ok_or(format!("unknown signal id: {}", &start.id))?;

        let (start_node, start_dir) = (start_sgn.protect_node_id, start_sgn.dir.reverse());
        //這裏的方向是根據用戶輸入判斷的朝向，和最終尋到的路徑的前後朝向做判斷
        //使用按鈕類型判斷進路類型
        let (mut is_pass, mut is_send, mut is_recv, mut is_shnt) = (false, false, false, false);

        let (goal_node, goal_dir) = match (start.kind, end.kind) {
            //通過按鈕 -> 列車按鈕 = 通過進路
            (ButtonKind::Pass, ButtonKind::Train) => {
                is_pass = true;
                let end_sgn = fsm
                    .sgns
                    .get(&end.id)
                    .ok_or(format!("unknown signal id: {}", &end.id))?;
                (end_sgn.toward_node_id, end_sgn.dir.clone())
            }
            //通過按鈕 -> 列車終端按鈕 = 通過進路
            (ButtonKind::Pass, ButtonKind::LZA) => {
                is_pass = true;
                let node_id = self
                    .ind_btn
                    .get(&end.id)
                    .ok_or(format!("unknown button id: {}", &end.id))?
                    .clone();

                (node_id, start_dir.clone())
            }
            //列車按鈕 -> 列車按鈕 = 接發車進路
            (ButtonKind::Train, ButtonKind::Train) => {
                let end_sgn = fsm
                    .sgns
                    .get(&end.id)
                    .ok_or(format!("unknown signal id: {}", &end.id))?;
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
                    .dif_relation
                    .get(&end.id)
                    .or(self.jux_relation.get(&end.id))
                    .unwrap_or(&end.id);

                let end_sgn = fsm
                    .sgns
                    .get(end_id)
                    .ok_or(format!("unknown signal id: {}", end_id))?;

                (end_sgn.toward_node_id, end_sgn.dir.reverse())
            }
            _ => return Err("no route found".to_string()),
        };

        //dir 是檢索到的可用方向
        let (maybe_path, dir) = self
            .graph
            .available_path(start_node, goal_node)
            .ok_or("no available path exists")?;

        //進路方向 bound
        if dir != start_dir || dir != goal_dir {
            return Err("no available route exists".to_string());
        }

        //ensure that all nodes are not used or locked by another existing path
        let mut sgn_id = Vec::new();
        for node in &maybe_path {
            let node = fsm.nodes.get(node).expect("invalid node id");
            if node.state != NodeStatus::Vacant {
                return Err("target path is not vacant".into());
            }
            if node.is_lock {
                return Err("target path is conflicting".into());
            }
            if node.used_count > 0 {
                return Err("target path is mutex".into());
            }

            let sgn = match dir {
                RawDirection::Left => node.right_sgn_id.clone(),
                RawDirection::Right => node.left_sgn_id.clone(),
            };
            if let Some(sgn) = sgn {
                sgn_id.push(sgn);
            }
        }

        //锁闭区段
        for id in &maybe_path {
            let node = fsm.node_mut(*id);
            node.lock(producer).await;
            node.once_occ = false; //重置曾占用flag
            graph.s_graph.neighbors(*id).for_each(|id| {
                fsm.node_mut(id).used_count += 1;
            })
        }

        //開放長調車進路途經信號機

        let start_sgn = fsm
            .sgns
            .get_mut(&start.id)
            .ok_or(format!("unknown signal id: {}", &start.id))?;

        if is_pass {
            start_sgn.open_pass(producer).await;
        }

        if is_send {
            start_sgn.open_send(producer).await;
        }

        if is_recv {
            let arc_fsm = self.fsm.clone();
            let fsm = arc_fsm.lock().await;
            let node = fsm.node(goal_node);
            start_sgn.open_recv(node.kind.clone(), producer).await;
        }

        if is_shnt {
            for id in sgn_id {
                let sgn = fsm.sgn_mut(&id);
                sgn.open_shnt(producer).await;
            }
        }

        Ok(maybe_path)
    }

    //進路を消す
    pub(crate) async fn cancel_path(
        &self,
        start: PathBtn,
        producer: &FutureProducer,
    ) -> Result<(), String> {
        let arc_fsm = self.fsm.clone();
        let fsm = arc_fsm.lock().await;

        let start_sgn = fsm
            .sgns
            .get(&start.id)
            .ok_or(format!("unknown signal id: {}", &start.id))?;
        let (start_node, start_dir) = (start_sgn.protect_node_id, start_sgn.dir.reverse());

        let maybe_route = self
            .find_a_route(start_node, &start_dir)
            .await
            .ok_or("not find a existed route")?;

        //調車進路解鎖所有調車信號機
        if start_sgn.kind == RawSignalKind::ShuntingSignal {
            for n in maybe_route {
                let arc_fsm = self.fsm.clone();
                let asy_producer = producer.clone();
                tokio::spawn(async move {
                    let mut fsm = arc_fsm.lock().await;
                    fsm.node_mut(n).unlock(&asy_producer).await;
                });

                let sgn_id = match start_dir {
                    RawDirection::Left => &fsm.node(n).right_sgn_id,
                    RawDirection::Right => &fsm.node(n).left_sgn_id,
                };

                if let Some(sgn_id) = sgn_id {
                    let arc_fsm = self.fsm.clone();
                    let mut fsm = arc_fsm.lock().await;
                    let sgn = fsm.sgn_mut(&sgn_id);
                    if sgn.kind == RawSignalKind::ShuntingSignal {
                        sgn.protect(&producer).await;
                    }
                }
            }
        }

        Ok(())
    }

    async fn find_a_route(&self, nid: NodeID, dir: &RawDirection) -> Option<Vec<NodeID>> {
        let arc_fsm = self.fsm.clone();
        let fsm = arc_fsm.lock().await;
        let curr = fsm.node(nid);
        if !curr.is_lock || curr.state != NodeStatus::Vacant {
            return None;
        }
        let mut res = vec![nid];
        while let Some(next) = self.next_route_node(*res.last().unwrap(), dir).await {
            res.push(next);
        }
        Some(res)
    }

    pub(crate) async fn next_route_node(
        &self,
        start_node: NodeID,
        dir: &RawDirection,
    ) -> Option<NodeID> {
        let arc_fsm = self.fsm.clone();
        let fsm = arc_fsm.lock().await;
        let graph = &self.graph;
        let edges = graph.r_graph.edges(start_node);

        for (_, t, d) in edges {
            let to = fsm.node(t);
            if d == dir && to.is_lock && to.state == NodeStatus::Vacant {
                return Some(t);
            }
        }
        None
    }

    pub(crate) async fn spawn_train(&mut self, node: NodeID) {
        let id = self.trains.len()+ 1;
        let new_train = Train::new(node, id);
        self.trains.push(new_train);
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct InstanceConfig {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) player: String,
    pub(crate) station: RawStation,
    pub(crate) token: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub(crate) enum InstanceData {
    Exam { exam_id: String },
    Chain,
    Exercise,
}

#[derive(Eq, PartialEq, Display, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum InstanceStatus {
    Prestart, //启动前
    Playing,  //使用中
    Finished, //已结束
}

pub type InstancePool = HashMap<String, Instance>;
