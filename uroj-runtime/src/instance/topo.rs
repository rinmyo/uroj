// 進路になる必要な条件：
// １．r_graphの点でできるパースのこと
// 2. パースの点たちは相互にS関係がないこと
// 3. パースの点はLOCKでもUSEでもいけないこと

use std::collections::HashMap;

use crate::raw_station::{IndButton, RawDirection, RawNode, RawSignal};
use petgraph::{
    algo,
    graphmap::{DiGraphMap, UnGraphMap},
};

use super::fsm::NodeID;

//站場圖
pub(crate) struct Topo {
    pub(crate) r_graph: DiGraphMap<NodeID, RawDirection>,
    pub(crate) s_graph: UnGraphMap<NodeID, ()>,
    pub(crate) dif_relation: HashMap<String, String>,
    pub(crate) jux_relation: HashMap<String, String>,
    pub(crate) ind_btn: HashMap<String, NodeID>,
    // b_graph: UnGraphMap<NodeID, ()>,
}

impl Topo {
    pub(crate) fn new(
        nodes: &Vec<RawNode>,
        sgns: &Vec<RawSignal>,
        ind_btns: &Vec<IndButton>,
    ) -> Self {
        let mut r_graph = DiGraphMap::new();
        let mut s_graph = UnGraphMap::new();
        // let b_graph= UnGraphMap::new();

        nodes.iter().for_each(|n| {
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

        let jux_relation = sgns
            .iter()
            .filter_map(|s| s.jux_sgn.as_ref().map(|k| (s.id.clone(), k.clone())))
            .collect();

        let dif_relation = sgns
            .iter()
            .filter_map(|s| s.dif_sgn.as_ref().map(|k| (s.id.clone(), k.clone())))
            .collect();

        let ind_btn = ind_btns
            .iter()
            .map(|b| (b.id.clone(), b.protect_node_id))
            .collect();

        Topo {
            r_graph: r_graph,
            s_graph: s_graph,
            jux_relation: jux_relation,
            dif_relation: dif_relation,
            ind_btn: ind_btn,
        }
    }

    //可能な進路を探す
    pub(crate) fn available_path(
        &self,
        start: NodeID,
        goal: NodeID,
    ) -> Option<(Vec<NodeID>, RawDirection, RawDirection)> {
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
        let entry_dir = self
            .r_graph
            .edge_weight(maybe_path[0], maybe_path[1])?
            .clone();

        let end_dir = self
            .r_graph
            .edge_weight(maybe_path[maybe_path.len() - 2], maybe_path[maybe_path.len() - 1])?
            .clone();

        Some((maybe_path, entry_dir, end_dir))
    }

    pub(crate) fn direction(&self, from: NodeID, to: NodeID) -> Option<RawDirection> {
        self.r_graph.edge_weight(from, to).map(|d| d.clone())
    }
}
