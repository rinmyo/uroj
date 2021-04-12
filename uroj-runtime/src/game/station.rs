use serde::Serialize;
use uroj_common::station::{Direction, SignalKind, SignalMounting};

#[derive(Debug, Serialize)]
pub struct NodeData {
    pub node_id: i32,
    pub track_id: String,
    pub line: ((f64, f64), (f64, f64)), //綫段，用於渲染
    pub joint: (String, String),        //兩端是否有絕緣節，用於渲染
}

#[derive(Debug, Serialize)]
pub struct SignalData {
    pub signal_id: String,
    pub pos: (f64, f64),         //位置
    pub dir: Direction,          //朝向
    pub sgn_type: SignalKind,    //信號類型
    pub sgn_mnt: SignalMounting, //安裝方式
    pub protect_node_id: i32,    //防护node 的 ID
}

#[derive(Debug, Serialize)]
pub struct StationData {
    pub station_name: String,
    pub nodes: Vec<NodeData>,
    pub signals: Vec<SignalData>,
}
