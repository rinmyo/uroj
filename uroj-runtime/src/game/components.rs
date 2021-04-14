use std::collections::HashSet;

use uroj_common::station::SignalKind;

use super::station::{NodeData, SignalData};

pub(crate) type NodeId = usize;
//Node 狀態機，沒有耦合信息
pub(crate) struct TrackNode {
    pub(crate) node_id: NodeId,
    pub(crate) used_count: u32, //被征用计数，每次征用则INC，每次作为S扩展集中的点被解除征用则减1，为0则说明未被征用
    pub(crate) state: NodeStatus,
}

impl From<&NodeData> for TrackNode {
    fn from(data: &NodeData) -> Self {
        TrackNode {
            node_id: data.node_id,
            used_count: 0,
            state: Default::default(),
        }
    }
}


//由车辆的位置和预设变量决定，是轨道电路的表征
// 在毕业设计中使用状态转移图，可以凑字数
#[derive(Eq, PartialEq)]
pub(crate) enum NodeStatus {
    Lock,         //锁闭, 白，锁闭的前提条件是未被锁闭 未被征用
    Occupied,     //占用，赤
    OnceOcc,      //曾占用, 赤
    Unexpected,   //异常，条
    Vacant,       //空闲，蓝
}

impl Default for NodeStatus {
    fn default() -> Self {
        NodeStatus::Vacant
    }
}

pub(crate) type Path<'a> = Vec<&'a TrackNode>;

pub(crate) type Track<'a> = HashSet<&'a TrackNode>;

pub(crate) struct Signal {
    signal_id: String,
    filament_status: (FilamentStatus, FilamentStatus),
    state: SignalStatus,
    used_flag: bool, //征用
}

impl From<&SignalData> for Signal {
    fn from(data: &SignalData) -> Self {
        let filament_status = match data.sgn_type {
            //调车信号机只有一个灯丝
            SignalKind::ShuntingSignal => (Default::default(), FilamentStatus::None),
            _ => (Default::default(), Default::default()),
        };

        let state = match data.sgn_type {
            SignalKind::ShuntingSignal => SignalStatus::A,
            _ => SignalStatus::H,
        };

        Signal {
            signal_id: data.signal_id,
            filament_status: filament_status,
            state: state,
            used_flag: false,
        }
    }
}

pub(crate) enum SignalStatus {
    L,
    U,
    H,
    B,
    A,
    UU,
    LU,
    LL,
    US,
    HB,
    OFF,
}

pub(crate) enum FilamentStatus {
    Normal,
    Fused,
    None,
}

impl Default for FilamentStatus {
    fn default() -> Self {
        FilamentStatus::Normal
    }
}
