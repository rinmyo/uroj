use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

use petgraph::{
    algo,
    graphmap::{DiGraphMap, UnGraphMap},
};

use tokio::sync::{
    mpsc,
    mpsc::{Receiver, Sender},
};
use uroj_common::rpc::InstanceConfig;
use uroj_common::rpc::{Node as StaticNode, Signal as StaticSignal};
use uroj_common::rpc::{NodeKind, SignalKind};

use crate::models::{ButtonKind, GameFrame};

use super::components::{Node, NodeID, NodeStatus, Signal, Train};

// 進路になる必要な条件：
// １．r_graphの点でできるパースのこと
// 2. パースの点たちは相互にS関係がないこと
// 3. パースの点はLOCKでもUSEでもいけないこと

#[derive(Clone, PartialEq, Eq)]
pub(crate) enum Direction {
    Left,
    Right,
}

impl Direction {
    pub(crate) fn reverse(&self) -> Self {
        match self {
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

//站場圖
pub(crate) struct StationGraph {
    r_graph: DiGraphMap<NodeID, Direction>,
    s_graph: UnGraphMap<NodeID, ()>,
    // b_graph: UnGraphMap<NodeID, ()>,
}

impl StationGraph {
    fn new(data: Vec<StaticNode>) -> StationGraph {
        let mut r_graph = DiGraphMap::new();
        let mut s_graph = UnGraphMap::new();
        // let b_graph= UnGraphMap::new();

        data.iter().for_each(|n| {
            for i in &n.left_adj {
                r_graph.add_edge(n.node_id, *i, Direction::Left);
            }
            for i in &n.right_adj {
                r_graph.add_edge(n.node_id, *i, Direction::Right);
            }
            for i in &n.conflicted_nodes {
                s_graph.add_edge(n.node_id, *i, ());
            }
        });

        StationGraph {
            r_graph: r_graph,
            s_graph: s_graph,
            // b_graph: b_graph,
        }
    }

    //可能な進路を探す
    fn available_path(&self, start: NodeID, goal: NodeID) -> Option<(Vec<NodeID>, Direction)> {
        if start == goal {
            return None;
        }
        let maybe_path = algo::astar(&self.r_graph, start, |node| node == goal, |_| 1, |_| 0)
            .map(|(_, path)| path)?;
        //pathはS関係に違反すると、必ず有効な進路じゃないということになる
        for (i, &j) in maybe_path.iter().enumerate() {
            for k in i + 1..maybe_path.len() {
                if self.s_graph.contains_edge(j, maybe_path[k]) {
                    return None;
                }
            }
        }
        let dir = self
            .r_graph
            .edge_weight(maybe_path[0], maybe_path[1])?
            .clone();
        Some((maybe_path, dir))
    }

    pub(crate) fn direction(&self, from: &NodeID, to: &NodeID) -> Option<Direction> {
        self.r_graph
            .edge_weight(from.clone(), to.clone())
            .map(|d| d.clone())
    }
}

struct Route {
    path: Vec<NodeID>,
    dir: Direction,
}

//實例狀態機
pub(crate) struct InstanceFSM {
    sgns: HashMap<String, Signal>,
    nodes: HashMap<NodeID, Node>,
}

impl InstanceFSM {
    fn new(
        sgn_vec: Vec<StaticSignal>,
        node_vec: Vec<StaticNode>,
        graph: &StationGraph,
    ) -> Result<InstanceFSM, String> {
        let mut sgns: HashMap<String, Signal> =
            sgn_vec.iter().map(|s| (s.id.clone(), s.into())).collect();
        let mut nodes: HashMap<NodeID, Node> =
            node_vec.iter().map(|n| (n.node_id, n.into())).collect();

        for s in sgn_vec {
            let pid = s.protect_node_id;
            let tid = s.toward_node_id;

            let dir = graph
                .direction(&pid, &tid)
                .or_else(|| match s.is_left {
                    Some(true) => Some(Direction::Left),
                    Some(false) => Some(Direction::Right),
                    None => None,
                })
                .ok_or(format!("connot parse the direction of signal {}", s.id))?;

            let sgn = sgns.get_mut(&s.id).unwrap();
            (*sgn).direction = dir.clone();

            let p_node = nodes
                .get_mut(&pid)
                .ok_or(format!("unknown node id: {}", pid))?;

            match dir {
                Direction::Left => (*p_node).left_sgn_id = Some(s.id),
                Direction::Right => (*p_node).right_sgn_id = Some(s.id),
            }
        }

        Ok(InstanceFSM {
            sgns: sgns,
            nodes: nodes,
        })
    }

    pub(crate) fn node_mut(&mut self, id: NodeID) -> &mut Node {
        &mut self.nodes[&id]
    }

    pub(crate) fn node(&self, id: NodeID) -> &Node {
        &self.nodes[&id]
    }

    pub(crate) fn sgn(&self, id: &str) -> &Signal {
        &self.sgns[id]
    }

    pub(crate) fn sgn_mut(&self, id: &str) -> &mut Signal {
        &mut self.sgns[id]
    }
}

pub(crate) struct InstanceInfo {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) player: String,
    pub(crate) token: String,
}

impl From<&InstanceConfig> for InstanceInfo {
    fn from(config: &InstanceConfig) -> Self {
        InstanceInfo {
            id: config.id,
            title: config.title,
            player: config.player,
            token: config.token,
        }
    }
}

pub(crate) struct Instance {
    pub(crate) fsm: Arc<Mutex<InstanceFSM>>,
    pub(crate) graph: StationGraph,
    pub(crate) dif_relation: HashMap<String, String>,
    pub(crate) jux_relation: HashMap<String, String>, //signal -> signal
    pub(crate) ind_btn: HashMap<String, NodeID>,      //按鈕ID -> 防護節點ID
    pub(crate) trains: Vec<Train>,
    pub(crate) info: InstanceInfo,
    pub(crate) frame_tx: Sender<GameFrame>,
    pub(crate) frame_rx: Receiver<GameFrame>,
}

pub(crate) struct PathBtn {
    pub(crate) id: String, //signal or button id
    pub(crate) kind: ButtonKind,
}

impl Instance {
    pub(crate) fn new(cfg: &InstanceConfig) -> Result<Self, String> {
        let signals = cfg.station.signals;
        let nodes = cfg.station.nodes;

        let graph = StationGraph::new(nodes);
        let fsm = InstanceFSM::new(signals, nodes, &graph)?;

        let jux_relation = signals
            .iter()
            .filter_map(|s| s.jux_sgn.map(|k| (s.id, k)))
            .collect();

        let dif_relation = signals
            .iter()
            .filter_map(|s| s.dif_sgn.map(|k| (s.id, k)))
            .collect();

        let ind_btn = cfg
            .station
            .independent_btns
            .iter()
            .map(|b| (b.id, b.protect_node_id))
            .collect();

        let (mut tx, rx) = mpsc::channel(32);

        Ok(Instance {
            fsm: Arc::new(Mutex::new(fsm)),
            graph: graph,
            dif_relation: dif_relation,
            jux_relation: jux_relation,
            ind_btn: ind_btn,
            trains: Vec::new(),
            info: cfg.into(),
            frame_tx: tx,
            frame_rx: rx,
        })
    }

    //パースを作成する
    pub(crate) async fn create_path(
        &self,
        start: PathBtn,
        end: PathBtn,
    ) -> Result<Vec<NodeID>, String> {
        let arc_fsm = self.fsm.clone();
        let mut fsm = arc_fsm.lock().await;

        let graph = &self.graph;

        let start_sgn = fsm
            .sgns
            .get(&start.id)
            .ok_or(format!("unknown signal id: {}", &start.id))?;

        let (start_node, start_dir) = (start_sgn.protect_node_id, start_sgn.direction.reverse());
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
                (end_sgn.toward_node_id, end_sgn.direction.clone())
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
                    (SignalKind::HomeSignal, SignalKind::StartingSignal) => {
                        is_recv = true;
                        (end_sgn.toward_node_id, end_sgn.direction.clone())
                    }
                    //出站信號機 -> 進站信號機 => 發車進路
                    (SignalKind::StartingSignal, SignalKind::HomeSignal) => {
                        is_send = true;
                        (end_sgn.protect_node_id, end_sgn.direction.clone())
                    }
                    _ => return Err("no route found".to_string()),
                }
            }
            //發車進路
            (ButtonKind::Train, ButtonKind::LZA) => {
                is_send = true;
                match start_sgn.kind {
                    SignalKind::StartingSignal => {
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

                (end_sgn.toward_node_id, end_sgn.direction.reverse())
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
                Direction::Left => node.right_sgn_id.clone(),
                Direction::Right => node.left_sgn_id.clone(),
            };
            if let Some(sgn) = sgn {
                sgn_id.push(sgn);
            }
        }

        //锁闭区段
        for id in &maybe_path {
            let node = fsm.node_mut(*id);
            node.lock(&self.frame_tx);
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
            start_sgn.open_pass(&self.frame_tx);
        }

        if is_send {
            start_sgn.open_send(&self.frame_tx);
        }

        if is_recv {
            let arc_fsm = self.fsm.clone();
            let fsm = arc_fsm.lock().await;
            let node = fsm.node(goal_node);
            start_sgn.open_recv(node.kind.clone(), &self.frame_tx);
        }

        if is_shnt {
            let sgns = sgn_id.iter().map(|id| fsm.sgn_mut(id));
            Signal::open_shnt(sgns, &self.frame_tx);
        }

        Ok(maybe_path)
    }

    //進路を消す
    pub(crate) async fn cancel_path(&self, start: PathBtn) -> Result<(), String> {
        let arc_fsm = self.fsm.clone();
        let fsm = arc_fsm.lock().await;
        let graph = &self.graph;

        let start_sgn = fsm
            .sgns
            .get(&start.id)
            .ok_or(format!("unknown signal id: {}", &start.id))?;
        let (start_node, start_dir) = (start_sgn.protect_node_id, start_sgn.direction.reverse());

        let maybe_route = self
            .find_a_route(start_node, &start_dir)
            .await
            .ok_or("not find a existed route")?;

        //調車進路解鎖所有調車信號機
        if start_sgn.kind == SignalKind::ShuntingSignal {
            for n in maybe_route {
                let frame_tx = self.frame_tx.clone();

                let arc_fsm = self.fsm.clone();
                tokio::spawn(async move {
                    let mut fsm = arc_fsm.lock().await;
                    fsm.node_mut(n).unlock(&frame_tx).await;
                });

                let sgn_id = match start_dir {
                    Direction::Left => &fsm.node(n).right_sgn_id,
                    Direction::Right => &fsm.node(n).left_sgn_id,
                };

                if let Some(sgn_id) = sgn_id {
                    let sgn = fsm.sgn_mut(&sgn_id);
                    if sgn.kind == SignalKind::ShuntingSignal {
                        sgn.protect(&self.frame_tx)
                    }
                }
            }
        }

        Ok(())
    }

    async fn find_a_route(&self, nid: NodeID, dir: &Direction) -> Option<Vec<NodeID>> {
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
        dir: &Direction,
    ) -> Option<NodeID> {
        let arc_fsm = self.fsm.clone();
        let fsm = arc_fsm.lock().await;
        let graph = &self.graph;
        let edges = graph.r_graph.edges(start_node);

        for (f, t, d) in edges {
            let to = fsm.node(t);
            if d == dir && to.is_lock && to.state == NodeStatus::Vacant {
                return Some(t);
            }
        }
        None
    }
}

pub(crate) type InstancePool = HashMap<String, Instance>;
type TrainID = usize;
