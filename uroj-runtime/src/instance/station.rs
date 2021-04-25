use crate::raw_station::*;
use async_graphql::*;

use super::fsm::NodeID;

#[derive(SimpleObject, Clone)]
pub(crate) struct Point {
    x: f64,
    y: f64,
}

impl<T: Into<f64>> From<(T, T)> for Point {
    fn from(p: (T, T)) -> Self {
        Point {
            x: p.0.into(),
            y: p.1.into(),
        }
    }
}

#[derive(SimpleObject, Clone)]
pub(crate) struct NodeData {
    pub(crate) node_id: usize,
    pub(crate) track_id: String,
    pub(crate) left_p: Point,
    pub(crate) right_p: Point,
    pub(crate) left_joint: String,  //始端绝缘节
    pub(crate) right_joint: String, //终端绝缘节
}

impl From<&RawNode> for NodeData {
    fn from(node: &RawNode) -> Self {
        NodeData {
            node_id: node.id,
            track_id: node.track_id.clone(),
            left_p: node.line.0.into(),
            right_p: node.line.1.into(),
            left_joint: node.joint.0.to_string(),
            right_joint: node.joint.1.to_string(),
        }
    }
}

#[derive(SimpleObject, Clone)]
pub(crate) struct SignalData {
    pub(crate) signal_id: String,
    pub(crate) sgn_type: RawSignalKind,    //信號類型
    pub(crate) sgn_mnt: RawSignalMounting, //安裝方式
    pub(crate) protect_node_id: NodeID,    //防护node 的 ID
    pub(crate) side: RawNodeSide,
    pub(crate) dir: RawDirection, //
    pub(crate) pos: Point,
}

//從rawsignal 到 signaldata，不能推斷的類型先缺省
impl From<&RawSignal> for SignalData {
    fn from(sgn: &RawSignal) -> Self {
        SignalData {
            signal_id: sgn.id.clone(),
            sgn_type: sgn.sgn_kind,
            sgn_mnt: sgn.sgn_mnt,
            protect_node_id: sgn.protect_node_id,
            side: sgn.side,
            dir: RawDirection::Left,  //缺省，需要更新
            pos: Point::from((0, 0)), //缺省，需要更新
        }
    }
}

// front models
#[derive(SimpleObject, Clone)]
pub(crate) struct StationData {
    pub(crate) title: String,
    pub(crate) nodes: Vec<NodeData>,
    pub(crate) signals: Vec<SignalData>,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum ButtonKind {
    Pass,  //通過按鈕
    Shunt, //調車按鈕
    Train, //列車按鈕（接發車）
    Guide, //引導按鈕
    LZA,   //列車終端按鈕
}
