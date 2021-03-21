use std::collections::HashSet;

pub(crate) struct TrackNode {
    node_id: String,
    state: NodeStatus,
    used_count: u32, //被征用计数，每次征用则加一，每次作为S扩展集中的点被解除征用则减1，为0则说明未被征用
    protect_signal_id: String, //防护信号机id
}

//由车辆的位置和预设变量决定，是轨道电路的表征
// 在毕业设计中使用状态转移图，可以凑字数
pub(crate) enum NodeStatus {
    Lock,       //锁闭, 白，锁闭的前提条件是未被锁闭 未被征用
    Occupied,   //占用，赤
    Unexpected, //异常，条
    Vacant,     //空闲，蓝
}

pub(crate) struct Train<'a> {
    id: String,
    velocity: (f32, f32),
    position: (f32, f32),
    curr_node: &'a TrackNode,
}

pub(crate) type Route<'a> = Vec<&'a TrackNode>;

pub(crate) type Track<'a> = HashSet<&'a TrackNode>;

pub(crate) struct Signal {
    signal_id: String,
    state: SignalStatus,
    filament_status: (FilamentStatus, FilamentStatus),
    used_flag: bool, //征用
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
}
