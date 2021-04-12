use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Direction {
    Left,  //向左
    Right, //向右
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SignalKind {
    HomeSignal,     //進站信號機
    StartingSignal, //出站信號機
    ShuntingSignal, //調車信號機
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SignalMounting {
    PostMounting,   //高柱
    GroundMounting, //矮柱
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Signal {
    pub id: String,
    pub pos: (f64, f64),         //位置 渲染用
    pub dir: Direction,          //朝向 渲染用
    pub sig_type: SignalKind,    //信號類型 渲染用
    pub sig_mnt: SignalMounting, //安裝方式 渲染用
    pub protect_node_id: i32,    //防护node 的 ID 业务&渲染
}

pub enum InstanceData {
    Exam { exam_id: String },
    Chain,
    Exercise,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JointKind {
    Normal,    //普通
    Clearance, //侵限绝缘
    End,       //尽头
    Empty,     //无
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Node {
    pub node_id: i32,
    pub turnout_id: Vec<i32>, //无岔區段則空，len即为包含道岔数，通過計算得出道岔中心
    pub track_id: String,     //所属軌道電路， 用於構建 B 關係，特殊區段（接近、 離去）通過id識別
    pub adjoint_nodes: Vec<i32>, //鄰接節點， 用於構建 R 關係
    pub conflicted_nodes: Vec<i32>, //牴觸節點, 用於構建 S 關係
    pub line: ((f64, f64), (f64, f64)), //綫段，用於渲染
    pub joint: (JointKind, JointKind), //兩端是否有絕緣節，用於渲染
}

pub struct NewInstanceRequest {
    pub instance_type: InstanceData,
    pub station_id: String,
    pub nodes: Vec<Node>,
    pub signals: Vec<Signal>,
}
