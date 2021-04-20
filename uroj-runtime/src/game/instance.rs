use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
    time::Duration,
};

use futures::Stream;
use petgraph::{
    algo,
    graphmap::{DiGraphMap, UnGraphMap},
};

use tokio::time::sleep;
use uroj_common::rpc::InstanceConfig;
use uroj_common::rpc::SignalKind;
use uroj_common::rpc::{Node as StaticNode, Signal as StaticSignal};

use crate::models::ButtonKind;

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
        let r_graph = DiGraphMap::new();
        let s_graph = UnGraphMap::new();
        // let b_graph= UnGraphMap::new();

        data.iter().for_each(|n| {
            for i in n.left_adj {
                r_graph.add_edge(n.node_id, i, Direction::Left);
            }
            for i in n.right_adj {
                r_graph.add_edge(n.node_id, i, Direction::Right);
            }
            for i in n.conflicted_nodes {
                s_graph.add_edge(n.node_id, i, ());
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
        let sgns: HashMap<String, Signal> = sgn_vec.iter().map(|s| (s.id, s.into())).collect();
        let nodes: HashMap<NodeID, Node> = node_vec.iter().map(|n| (n.node_id, n.into())).collect();

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

    pub(crate) fn node_mut(&self, id: &NodeID) -> &mut Node {
        &mut self.nodes[id]
    }

    pub(crate) fn node(&self, id: &NodeID) -> &Node {
        &self.nodes[id]
    }

    fn shared_node(&self, id: &NodeID) -> Arc<Mutex<Node>> {
        Mutex::new(self.nodes[id]).into()
    }

    pub(crate) fn sgn(&self, id: &str) -> &Signal {
        &self.sgns[id]
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
    pub(crate) fsm: InstanceFSM,
    pub(crate) graph: StationGraph,
    dif_relation: HashMap<String, String>,
    jux_relation: HashMap<String, String>, //signal -> signal
    ind_btn: HashMap<String, NodeID>,      //按鈕ID -> 防護節點ID
    trains: Vec<Train>,
    pub(crate) info: InstanceInfo,
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

        Ok(Instance {
            fsm: fsm,
            graph: graph,
            dif_relation: dif_relation,
            jux_relation: jux_relation,
            ind_btn: ind_btn,
            trains: Vec::new(),
            info: cfg.into(),
        })
    }

    //パースを作成する
    pub(crate) fn create_path(&self, start: PathBtn, end: PathBtn) -> Result<Vec<NodeID>, String> {
        let fsm = &self.fsm;
        let graph = &self.graph;

        let start_sgn = fsm
            .sgns
            .get(&start.id)
            .ok_or(format!("unknown signal id: {}", &start.id))?;
        let (start_node, start_dir) = (start_sgn.protect_node_id, start_sgn.direction.reverse());
        //這裏的方向是根據用戶輸入判斷的朝向，和最終尋到的路徑的前後朝向做判斷
        //使用按鈕類型判斷進路類型
        let (goal_node, goal_dir) = match (start.kind, end.kind) {
            //通過按鈕 -> 列車按鈕 = 通過進路
            (ButtonKind::Pass, ButtonKind::Train) => {
                let end_sgn = fsm
                    .sgns
                    .get(&end.id)
                    .ok_or(format!("unknown signal id: {}", &end.id))?;
                (end_sgn.toward_node_id, end_sgn.direction)
            }
            //通過按鈕 -> 列車終端按鈕 = 通過進路
            (ButtonKind::Pass, ButtonKind::LZA) => {
                let node_id = self
                    .ind_btn
                    .get(&end.id)
                    .ok_or(format!("unknown button id: {}", &end.id))?
                    .clone();

                (node_id, start_dir)
            }
            //列車按鈕 -> 列車按鈕 = 接發車進路
            (ButtonKind::Train, ButtonKind::Train) => {
                let end_sgn = fsm
                    .sgns
                    .get(&end.id)
                    .ok_or(format!("unknown signal id: {}", &end.id))?;
                match (start_sgn.kind, end_sgn.kind) {
                    //進站信號機 -> 出戰信號機 => 接車進路
                    (SignalKind::HomeSignal, SignalKind::StartingSignal) => {
                        (end_sgn.toward_node_id, end_sgn.direction)
                    }
                    //出站信號機 -> 進站信號機 => 發車進路
                    (SignalKind::StartingSignal, SignalKind::HomeSignal) => {
                        (end_sgn.protect_node_id, end_sgn.direction)
                    }
                    _ => return Err("no route found".to_string()),
                }
            }
            //發車進路
            (ButtonKind::Train, ButtonKind::LZA) => match start_sgn.kind {
                SignalKind::StartingSignal => {
                    let node_id = self
                        .ind_btn
                        .get(&end.id)
                        .ok_or(format!("unknown button id: {}", &end.id))?
                        .clone();

                    (node_id, start_dir)
                }

                _ => return Err("no route found".to_string()),
            },
            //調車進路
            (ButtonKind::Shunt, ButtonKind::Shunt) => {
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
        for node in maybe_path {
            let node = self.fsm.nodes.get(&node).expect("invalid node id");
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
                Direction::Left => node.right_sgn_id,
                Direction::Right => node.left_sgn_id,
            };
            if let Some(sgn) = sgn {
                sgn_id.push(sgn);
            }
        }

        //為保證進路建立的原子性
        maybe_path.iter().for_each(|id| {
            fsm.nodes[id].lock();
            fsm.nodes[id].once_occ = false; //重置曾占用flag
            graph.s_graph.neighbors(*id).for_each(|id| {
                fsm.nodes[&id].used_count += 1;
            })
        });

        //開放長調車進路途經信號機
        if (ButtonKind::Shunt, ButtonKind::Shunt) == (start.kind, end.kind) {
            sgn_id.iter().for_each(|s| {
                let mut sgn = fsm.sgn(s);
                if sgn.kind == SignalKind::ShuntingSignal {
                    sgn.open();
                }
            })
        } else {
            start_sgn.open()
        }

        Ok(maybe_path)
    }

    //進路を消す
    pub(crate) fn cancel_path(&self, start: PathBtn) -> Result<(), String> {
        let fsm = &self.fsm;
        let graph = &self.graph;

        let start_sgn = fsm
            .sgns
            .get(&start.id)
            .ok_or(format!("unknown signal id: {}", &start.id))?;
        let (start_node, start_dir) = (start_sgn.protect_node_id, start_sgn.direction.reverse());

        let maybe_route = self
            .find_a_route(start_node, &start_dir)
            .ok_or("not find a existed route")?;

        maybe_route.iter().for_each(|n| {});
        Ok(())
    }

    fn find_a_route(&self, nid: NodeID, dir: &Direction) -> Option<Vec<NodeID>> {
        let fsm = &self.fsm;
        let curr = fsm.node(&nid);
        if !curr.is_lock || curr.state != NodeStatus::Vacant {
            return None;
        }
        let mut res = vec![nid];
        while let Some(next) = self.next_route_node(*res.last().unwrap(), dir) {
            res.push(next);
        }
        Some(res)
    }

    pub(crate) fn next_route_node(&self, start_node: NodeID, dir: &Direction) -> Option<NodeID> {
        let fsm = &self.fsm;
        let graph = &self.graph;
        let edges = graph.r_graph.edges(start_node);

        for (f, t, d) in edges {
            let to = fsm.node(&t);
            if d == dir && to.is_lock && to.state == NodeStatus::Vacant {
                return Some(t);
            }
        }
        None
    }
}

pub(crate) type InstancePool = HashMap<String, Instance>;
type TrainID = usize;
