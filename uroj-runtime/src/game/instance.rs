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
use uroj_common::station::Direction;

use super::{components::{Path, Train}};
use tokio::time::sleep;

use super::components::{NodeID, NodeStatus, Signal, TrackNode};

// 進路になる必要な条件：
// １．r_graphの点でできるパースのこと
// 2. パースの点たちは相互にS関係がないこと
// 3. パースの点はLOCKでもUSEでもいけないこと

//站場圖
pub(crate) struct StationGraph {
    r_graph: DiGraphMap<NodeID, Direction>,
    s_graph: UnGraphMap<NodeID, ()>,
    b_graph: UnGraphMap<NodeID, ()>,
}

impl StationGraph {

    fn new(data: Vec<&NodeData>) -> StationGraph {
        
    }

    //可能な進路を探す
    fn available_path(&self, start: NodeID, goal: NodeID, dir: &Direction) -> Option<Vec<NodeID>> {
        if start == goal {
            return None;
        }
        let path = algo::astar(&self.r_graph, start, |node| node == goal, |_| 1, |_| 0)
            .map(|(_, path)| path)?;
        //pathはS関係に違反すると、必ず有効な進路じゃないということになる
        for (i, &j) in path.iter().enumerate() {
            for k in i + 1..path.len() {
                if self.s_graph.contains_edge(j, path[k]) {
                    return None;
                }
            }
        }
        //見つけたパースの方向を確認する
        if self.r_graph.edge_weight(path[0], path[1])? != dir {
            return None;
        }
        Some(path)
    }

    pub(crate) fn is_adjacent(&self, from: &NodeID, to: &NodeID) -> bool {
        self.r_graph.contains_edge(from.clone(), from.clone())
    }
}

//實例狀態機
pub(crate) struct InstanceFSM {
    sgns: HashMap<String, Signal>,
    nodes: HashMap<NodeID, TrackNode>,
}

impl InstanceFSM {
    fn new(data: StationData) -> InstanceFSM {
        let sgns_iter = data
            .signals
            .iter()
            .map(|data| (data.into(), data.signal_id, data.protect_node_id));
        let sgns = sgns_iter.map(|(sgn, id, _)| (id, sgn)).collect();
        let protect_table = sgns_iter
            .map(|(_, sgn_id, node_id)| (node_id, sgn_id))
            .collect::<HashMap<_, _>>();
        let nodes = data
            .nodes
            .iter()
            .map(|n| {
                let mut n: TrackNode = n.into();
                n.pro_sgn_id = protect_table.get(&n.node_id).map(|sid| sid.clone());
                (n.node_id, n)
            })
            .collect();
        InstanceFSM {
            sgns: sgns,
            nodes: nodes,
        }
    }

    pub(crate) fn node_mut(&self, id: &NodeID) -> &mut TrackNode {
        &mut self.nodes[id]
    }

    pub(crate) fn node(&self, id: &NodeID) -> &TrackNode {
        &self.nodes[id]
    }

    fn shared_node(&self, id: &NodeID) -> Arc<Mutex<TrackNode>> {
        Mutex::new(self.nodes[id]).into()
    }

    pub(crate) fn sgn(&self, id: &str) -> &Signal {
        &self.sgns[id]
    }
}

pub(crate) struct Instance {
    pub(crate) fsm: InstanceFSM,
    pub(crate) graph: StationGraph,
    trains: Vec<Train>,
    //若某個信號機作爲終端信號機被點擊，則其可能防護的是差置信號機所實際防護的node
    alias_end: HashMap<String, NodeID>,
}

impl Instance {
    //パースを作成する
    fn create_path(&self, start_sgn_id: &str, end_sgn_id: &str) -> Result<Vec<NodeID>, String> {
        let fsm = &self.fsm;
        let graph = &self.graph;
        let alias_end = &self.alias_end;

        let start = fsm
            .sgns
            .get(start_sgn_id)
            .ok_or(format!("unknown signal id: {}", start_sgn_id))?;
        let &goal = alias_end
            .get(end_sgn_id)
            .or_else(|| fsm.sgns.get(end_sgn_id).map(|s| &s.pro_node_id))
            .ok_or(format!("unknown signal id: {}", start_sgn_id))?;
        let maybe_path = self
            .graph
            .available_path(start.pro_node_id, goal, &start.dir)
            .ok_or("no available path exists")?;

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

enum GameState {
    GamePlay,
    Pause,
    Result,
}

impl Default for GameState {
    fn default() -> Self {
        GameState::Pause
    }
}
