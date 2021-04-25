// 進路になる必要な条件：
// １．r_graphの点でできるパースのこと
// 2. パースの点たちは相互にS関係がないこと
// 3. パースの点はLOCKでもUSEでもいけないこと

use crate::raw_station::{RawNode, RawDirection};
use petgraph::{
    algo,
    graphmap::{DiGraphMap, UnGraphMap},
};

use super::fsm::{NodeID};

//站場圖
pub(crate) struct StationGraph {
    pub(crate) r_graph: DiGraphMap<NodeID, RawDirection>,
    pub(crate) s_graph: UnGraphMap<NodeID, ()>,
    // b_graph: UnGraphMap<NodeID, ()>,
}

impl StationGraph {
    pub(crate) fn new(data: &Vec<RawNode>) -> StationGraph {
        let mut r_graph = DiGraphMap::new();
        let mut s_graph = UnGraphMap::new();
        // let b_graph= UnGraphMap::new();

        data.iter().for_each(|n| {
            for i in &n.left_adj {
                r_graph.add_edge(n.id, *i, RawDirection::Left);
            }
            for i in &n.right_adj {
                r_graph.add_edge(n.id, *i, RawDirection::Right);
            }
            for i in &n.conflicted_nodes {
                s_graph.add_edge(n.id, *i, ());
            }
        });

        StationGraph {
            r_graph: r_graph,
            s_graph: s_graph,
            // b_graph: b_graph,
        }
    }

    //可能な進路を探す
    pub(crate) fn available_path(
        &self,
        start: NodeID,
        goal: NodeID,
    ) -> Option<(Vec<NodeID>, RawDirection)> {
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

    pub(crate) fn direction(&self, from: NodeID, to: NodeID) -> Option<RawDirection> {
        self.r_graph
            .edge_weight(from, to)
            .map(|d| d.clone())
    }
}
