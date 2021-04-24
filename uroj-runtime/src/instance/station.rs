use crate::{
    instance::fsm::InstanceFSM,
    raw_station::{RawSignalKind, RawSignalMounting},
};

use crate::raw_station::{RawNode, RawSignal};
use async_graphql::*;

use std::collections::HashMap;

use super::fsm::NodeID;

#[derive(SimpleObject, Clone)]
pub(crate) struct Point {
    x: f64,
    y: f64,
}

impl From<(f64, f64)> for Point {
    fn from(p: (f64, f64)) -> Self {
        Point { x: p.0, y: p.1 }
    }
}

#[derive(SimpleObject, Clone)]
pub(crate) struct NodeData {
    pub(crate) node_id: usize,
    pub(crate) track_id: String,
    pub(crate) start: Point,
    pub(crate) end: Point,
    pub(crate) start_joint: String, //始端绝缘节
    pub(crate) end_joint: String,   //终端绝缘节
}

impl From<&RawNode> for NodeData {
    fn from(node: &RawNode) -> Self {
        NodeData {
            node_id: node.id,
            track_id: node.track_id.clone(),
            start: node.line.0.into(),
            end: node.line.1.into(),
            start_joint: node.joint.0.to_string(),
            end_joint: node.joint.1.to_string(),
        }
    }
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub(crate) enum Direction {
    LeftUp,
    LeftDown,
    RightUp,
    RightDown,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub(crate) enum SignalMounting {
    PostMounting,   //高柱
    GroundMounting, //矮柱
}

impl From<RawSignalMounting> for SignalMounting {
    fn from(m: RawSignalMounting) -> Self {
        match m {
            RawSignalMounting::PostMounting => Self::PostMounting,
            RawSignalMounting::GroundMounting => Self::GroundMounting,
        }
    }
}

#[derive(SimpleObject, Clone)]
pub(crate) struct SignalData {
    pub(crate) signal_id: String,
    pub(crate) sgn_type: RawSignalKind,    //信號類型
    pub(crate) sgn_mnt: SignalMounting, //安裝方式
    pub(crate) protect_node_id: NodeID, //防护node 的 ID
    pub(crate) dir: Direction,          //
    pub(crate) pos: Point,
}

// front models
#[derive(SimpleObject, Clone)]
pub(crate) struct StationData {
    pub(crate) title: String,
    pub(crate) nodes: Vec<NodeData>,
    pub(crate) signals: Vec<SignalData>,
}

impl StationData {
    pub(crate) fn new(
        raw_nodes: &Vec<RawNode>,
        raw_sgns: &Vec<RawSignal>,
        fsm: &InstanceFSM,
        title: &str,
    ) -> Result<Self, String> {
        //需要在创建时初始化好StationData，在用户访问时直接返回，同时还可以避免互斥锁的时间开销
        let nodes = raw_nodes.iter().map(|n| n.into()).collect();
        let nodes_map = raw_nodes
            .iter()
            .map(|n| (n.id, n))
            .collect::<HashMap<_, _>>();

        let mut signal_vec = vec![];

        for s in raw_sgns {
            let prt_node = fsm.nodes.get(&s.protect_node_id).ok_or("not find")?;
            let twd_node = fsm.nodes.get(&s.toward_node_id).ok_or("not find")?;

            // if (prt_node.left_sgn_id)

            let sgn_data = SignalData {
                signal_id: s.id,
                sgn_type: s.sgn_kind.into(),
                sgn_mnt: s.sgn_mnt.into(),
                protect_node_id: s.protect_node_id,
                dir: (),
                pos: pos,
            };
            signal_vec.push(sgn_data);
        }

        Ok(StationData {
            title: title.into(),
            nodes: nodes,
            signals: (),
        })
    }
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum ButtonKind {
    Pass,  //通過按鈕
    Shunt, //調車按鈕
    Train, //列車按鈕（接發車）
    Guide, //引導按鈕
    LZA,   //列車終端按鈕
}
