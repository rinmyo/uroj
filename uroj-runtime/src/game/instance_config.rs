use std::collections::HashMap;

use chrono::NaiveDateTime;
use diesel::sql_types::Uuid;
use serde::{Deserialize, Serialize};

pub enum SignalKind {
    HomeSignal,     //進站信號機
    StartingSignal, //出站信號機
    ShuntingSignal, //調車信號機
}

pub enum SignalMounting {
    PostMounting,   //高柱
    GroundMounting, //矮柱
}


pub enum ButtonKind {
    Pass,  //通過按鈕
    Shunt, //調車按鈕
    Train, //列車按鈕（接發車）
    Guide, //引導按鈕
}

pub struct Signal {
    pub id: String,
    pub pos: Option<(f64, f64)>, //位置 渲染用
    pub is_left: Option<bool>,   //左右朝向 業務，渲染，防护区段的方向
    pub is_up: bool,             // 上下朝向 渲染
    pub sig_type: SignalKind,    //信號類型 渲染用
    pub sig_mnt: SignalMounting, //安裝方式 渲染用
    pub protect_node_id: usize,  //防护node 的 ID 业务&渲染，防护node指的是其所防护的node
    pub toward_node_id: usize, //面朝的node ID, 若和上述node有公共点，则公共点就是信号机的位置, 再判断两个node的相对位置来决定左右朝向，如果没有则使用pos
    pub btns: Vec<ButtonKind>, //按钮
}

pub enum InstanceData {
    Exam { exam_id: String },
    Chain,
    Exercise,
}

pub enum JointKind {
    Normal,    //普通
    Clearance, //侵限绝缘
    End,       //尽头
    Empty,     //无
}

pub struct Node {
    pub node_id: usize,
    pub turnout_id: Vec<usize>, //无岔區段則空，len即为包含道岔数，通過計算得出道岔中心
    pub track_id: String,       //所属軌道電路， 用於構建 B 關係，特殊區段（接近、 離去）通過id識別
    pub left_adj: Vec<usize>,   //左鄰 用於構建 R 關係
    pub right_adj: Vec<usize>,  //右鄰 用於構建 R 關係
    pub conflicted_nodes: Vec<usize>, //牴觸節點, 用於構建 S 關係
    pub line: ((f64, f64), (f64, f64)), //綫段，用於渲染
    pub joint: (JointKind, JointKind), //兩端是否有絕緣節，用於渲染
}

pub struct IndButton {
    kind: ButtonKind,
    pos: (f64, f64),
    pub protect_node_id: usize,
}

pub enum GamerRole {
    Operator,
    Player,
    Spectator,
}

#[derive(Insertable, Serialize, Deserialize, Debug, AsChangeset, Associations)]
pub struct NewInstanceConfig {
    pub id: Uuid,
    pub instance_type: InstanceData,
    pub station_id: String,
    pub nodes: Vec<Node>,
    pub signals: Vec<Signal>,
    pub independent_btns: Vec<IndButton>,
    pub whitelist: HashMap<String, GamerRole>,
    pub start_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Identifiable, Associations, Queryable, AsChangeset)]
pub struct InstanceConfig {
    pub id: Uuid,
    pub instance_type: InstanceData,
    pub station_id: String,
    pub nodes: Vec<Node>,
    pub signals: Vec<Signal>,
    pub independent_btns: Vec<IndButton>,
    pub whitelist: HashMap<String, GamerRole>,
    pub start_at: NaiveDateTime,
}
