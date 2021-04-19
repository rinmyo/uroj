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

use super::components::{Node, NodeID, NodeStatus, Signal, Train};

// 進路になる必要な条件：
// １．r_graphの点でできるパースのこと
// 2. パースの点たちは相互にS関係がないこと
// 3. パースの点はLOCKでもUSEでもいけないこと

#[derive(Clone)]
pub(crate) enum Direction {
    Left,
    Right,
    End,
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
    fn available_path(
        &self,
        start: NodeID,
        goal: NodeID,
        dir: &Direction,
    ) -> Option<(Vec<(NodeID, Direction)>)> {
        if start == goal {
            return None;
        }
        let maybe_path = algo::astar(&self.r_graph, start, |node| node == goal, |_| 1, |_| 0)
            .map(|(_, path)| path)?;
        //pathはS関係に違反すると、必ず有効な進路じゃないということになる
        let mut route: Vec<(NodeID, Direction)>;
        for (i, &j) in maybe_path.iter().enumerate() {
            for k in i + 1..maybe_path.len() {
                if self.s_graph.contains_edge(j, maybe_path[k]) {
                    return None;
                }
            }
            let dir = if i < maybe_path.len() {
                self.r_graph.edge_weight(j, maybe_path[i + 1])?.clone()
            } else {
                Direction::End
            };
            route.push((j, dir));
        }
        Some(route)
    }

    pub(crate) fn direction(&self, from: &NodeID, to: &NodeID) -> Option<Direction> {
        self.r_graph.edge_weight(from.clone(), to.clone()).map(|d|d.clone())
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
    fn new(sgn_vec: Vec<StaticSignal>, node_vec: Vec<StaticNode>) -> InstanceFSM {
        let sgns = sgn_vec.iter().map(|s| (s.id, s.into())).collect();
        let protect_table: HashMap<_, _> =
            sgn_vec.iter().map(|s| (s.protect_node_id, s.id)).collect();
        let nodes = node_vec
            .iter()
            .map(|n| {
                let mut n: Node = n.into();
                n.pro_sgn_id = protect_table.get(&n.node_id).map(|sid| sid.clone());
                (n.node_id, n)
            })
            .collect();
        InstanceFSM {
            sgns: sgns,
            nodes: nodes,
        }
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
    id: String,
    title: String,
    player: String,
    token: String,
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
    trains: Vec<Train>,
    info: InstanceInfo,
}

impl Instance {
    pub(crate) fn new(cfg: &InstanceConfig) -> Self {
        let fsm = InstanceFSM::new(cfg.station.signals, cfg.station.nodes);
        let graph = StationGraph::new(cfg.station.nodes);
        Instance {
            fsm: fsm,
            graph: graph,
            trains: Vec::new(),
            info: cfg.into(),
        }
    }

    //パースを作成する
    pub(crate) fn create_path(
        &self,
        start_btn_id: &str,
        end_btn_id: &str,
    ) -> Result<Vec<NodeID>, String> {
        let fsm = &self.fsm;
        let graph = &self.graph;

        let start = fsm
            .sgns
            .get(start_sgn_id)
            .ok_or(format!("unknown signal id: {}", start_sgn_id))?;
        let &goal = alias_end
            .get(end_sgn_id)
            .or_else(|| fsm.sgns.get(end_sgn_id).map(|s| &s.pro_node_id))
            .ok_or(format!("unknown signal id: {}", start_sgn_id))?;
        let (maybe_path, dir) = self
            .graph
            .available_path(start.pro_node_id, goal, &start.dir)
            .ok_or("no available path exists")?;

        // if dir !=  {}

        //ensure that all nodes are not used or locked by another existing path
        for id in maybe_path {
            let node = self.fsm.nodes.get(&id).expect("invalid node id");
            if node.state != NodeStatus::Vacant {
                return Err("target path is not vacant".into());
            }
            if node.is_lock {
                return Err("target path is conflicting".into());
            }
            if node.used_count > 0 {
                return Err("target path is mutex".into());
            }
        }

        //ensure that the start node can be a node of a path
        let entry_id = maybe_path.first().ok_or("path is empty!")?; //入口節點
        let sgn_id = fsm.nodes[entry_id]
            .pro_sgn_id
            .ok_or("no protect signal found!")?;

        //為保證進路建立的原子性
        maybe_path.iter().for_each(|id| {
            fsm.nodes[id].lock();
            fsm.nodes[id].once_occ = false; //重置曾占用flag
            graph.s_graph.neighbors(*id).for_each(|id| {
                fsm.nodes[&id].used_count += 1;
            })
        });

        fsm.sgn(&sgn_id).open(); //開放信號機

        Ok(maybe_path)
    }
}

pub(crate) type InstancePool = HashMap<String, Instance>;
type TrainID = usize;
