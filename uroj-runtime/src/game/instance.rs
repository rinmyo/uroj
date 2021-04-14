use std::collections::{HashMap, HashSet};

use petgraph::{
    algo,
    graphmap::{DiGraphMap, UnGraphMap},
};

use super::{components::Path, station::StationData};

use super::components::{NodeId, NodeStatus, Signal, TrackNode};

// 進路になる必要な条件：
// １．r_graphの点でできるパースのこと
// 2. パースの点たちは相互にS関係がないこと
// 3. パースの点はLOCKでもUSEでもいけないこと

//站場圖
pub(crate) struct StationGraph {
    r_graph: DiGraphMap<NodeId, ()>,
    s_graph: UnGraphMap<NodeId, ()>,
    b_graph: UnGraphMap<NodeId, ()>,
}

impl StationGraph {
    //可能な進路を探す
    fn available_path(&self, start: NodeId, goal: NodeId) -> Option<Vec<NodeId>> {
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
        Some(path)
    }
}

//實例狀態機
pub(crate) struct InstanceFSM {
    sgns: HashMap<String, Signal>,
    nodes: HashMap<NodeId, TrackNode>,
}

impl InstanceFSM {
    fn new(data: StationData) -> InstanceFSM {
        let sgns_iter = data
            .signals
            .iter()
            .map(|data| (data.into(), data.signal_id, data.protect_node_id));
        let sgns = sgns_iter.map(|(sgn, id, _)| (id, sgn)).collect();
        let protect_table = sgns_iter
            .map(|(_, id, node_id)| (node_id, id))
            .collect::<HashMap<_, _>>();
        let nodes = data.nodes.iter().map(|n| (n.node_id, n.into())).collect();
        InstanceFSM {
            sgns: sgns,
            nodes: nodes,
        }
    }
}

pub(crate) struct Instance {
    fsm: InstanceFSM,
    graph: StationGraph,
    paths: HashSet<Vec<NodeId>>,
    //map a signal id to the id of node which is protected by the signal
    protected: HashMap<String, NodeId>,
    //若某個信號機作爲終端信號機被點擊，則其可能防護的是差置信號機所實際防護的node
    alias_end: HashMap<String, NodeId>,
}

impl Instance {
    //パースを作成する
    fn create_path(&self, start_sgn_id: &str, end_sgn_id: &str) -> Result<Vec<NodeId>, String> {
        let &start = self
            .protected
            .get(start_sgn_id)
            .ok_or(format!("unknown signal id: {}", start_sgn_id))?;
        let &goal = self
            .alias_end
            .get(end_sgn_id)
            .or_else(|| self.protected.get(end_sgn_id))
            .ok_or(format!("unknown signal id: {}", start_sgn_id))?;
        let maybe_path = self
            .graph
            .available_path(start, goal)
            .ok_or("no available path exists")?;
        //ensure that all nodes are not used or locked by another existing path
        for id in maybe_path {
            let node = self.fsm.nodes.get(&id).expect("invalid node id");
            if node.state != NodeStatus::Vacant {
                return Err("target path is conflicting".into());
            }
            if node.used_count > 0 {
                return Err("target path is mutex".into());
            }
        }
        //為保證進路建立的原子性
        maybe_path.iter().for_each(|id| {
            self.fsm.nodes[id].state = NodeStatus::Lock;
            self.graph.s_graph.neighbors(*id).for_each(|id| {
                self.fsm.nodes[&id].used_count += 1;
            })
        });

        self.paths.insert(maybe_path);

        Ok(maybe_path)
    }

    fn handle_train_movement(&self, from: NodeId, to: NodeId) {
        self.fsm.nodes[&to].state = NodeStatus::Occupied;
        for path in self.paths {
            if path.contains(&from) {
                self.fsm.nodes[&from].state = NodeStatus::OnceOcc;
                return;
            }
        }
        self.fsm.nodes[&to].state = NodeStatus::Vacant;
    }

    //calling when train move to new node
    fn update(&self, event: Event) {
        let nodes = self.fsm.nodes;

        for (i, j) in ids.iter().enumerate() {
            if nodes[&ids[i - 1]].state == NodeStatus::Vacant
                && nodes[j].state == NodeStatus::OnceOcc
                && nodes[&ids[i + 1]].state == NodeStatus::Occupied
            {
                // delay 3 sec, at p38 of textbook
                nodes[j].state = NodeStatus::Vacant
            }
        }
    }
}

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
