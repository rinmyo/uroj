use strum_macros::{Display, EnumString};

#[derive(Eq, PartialEq, Display, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum Direction {
    Left,  //向左
    Right, //向右
}

#[derive(Eq, PartialEq, Display, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum SignalKind {
    HomeSignal,     //進站信號機
    StartingSignal, //出站信號機
    ShuntingSignal, //調車信號機
}

#[derive(Eq, PartialEq, Display, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum SignalMounting {
    PostMounting,   //高柱
    GroundMounting, //矮柱
}

#[derive(PartialEq)]
pub struct Signal {
    pub id: String,
    pub pos: (f64, f64),         //位置 渲染用
    pub dir: Direction,          //朝向 渲染用
    pub sig_type: SignalKind,    //信號類型 渲染用
    pub sig_mnt: SignalMounting, //安裝方式 渲染用
    pub protect_node_id: usize,    //防护node 的 ID 业务&渲染
}

pub enum InstanceData {
    Exam { exam_id: String },
    Chain,
    Exercise,
}

#[derive(Eq, PartialEq, Display, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum JointKind {
    Normal,    //普通
    Clearance, //侵限绝缘
    End,       //尽头
    Empty,     //无
}

pub struct Node {
    pub node_id: usize,
    pub turnout_id: Vec<usize>, //无岔區段則空，len即为包含道岔数，通過計算得出道岔中心
    pub track_id: String,     //所属軌道電路， 用於構建 B 關係，特殊區段（接近、 離去）通過id識別
    pub left_adj: Vec<usize>, //左鄰 用於構建 R 關係
    pub right_adj: Vec<usize>, //右鄰 用於構建 R 關係
    pub conflicted_nodes: Vec<usize>, //牴觸節點, 用於構建 S 關係
    pub line: ((f64, f64), (f64, f64)), //綫段，用於渲染
    pub joint: (JointKind, JointKind), //兩端是否有絕緣節，用於渲染
    pub default_end_btn_sgn: Option<usize>, //缺省別名終端信號機，若非空則説明其值為該終端信號機
}

pub struct NewInstanceRequest {
    pub instance_type: InstanceData,
    pub station_id: String,
    pub nodes: Vec<Node>,
    pub signals: Vec<Signal>,
}
