use serde::{Deserialize, Serialize};
use strum_macros::*;
use async_graphql::*;

#[derive(Eq, PartialEq, Deserialize, Serialize, Debug, Clone, Enum, Copy)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum SignalKind {
    HomeSignal,     //進站信號機
    StartingSignal, //出站信號機
    ShuntingSignal, //調車信號機
}
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum SignalMounting {
    PostMounting,   //高柱
    GroundMounting, //矮柱
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum ButtonKind {
    Pass,  //通過按鈕
    Shunt, //調車按鈕
    Train, //列車按鈕（接發車）
    Guide, //引導按鈕
    LZA,   //列車終端按鈕
}

#[derive(Deserialize, Serialize, Debug)]
struct Signal {
    pub id: String,
    pub pos: Option<(f64, f64)>, //位置 渲染用
    pub is_left: Option<bool>,   //左右朝向 業務，渲染，防护区段的方向
    pub is_up: bool,             // 上下朝向 渲染
    pub sgn_kind: SignalKind,    //信號類型 渲染用
    pub sgn_mnt: SignalMounting, //安裝方式 渲染用
    pub protect_node_id: usize,  //防护node 的 ID 业务&渲染，防护node指的是其所防护的node
    pub toward_node_id: usize, //面朝的node ID, 若和上述node有公共点，则公共点就是信号机的位置, 再判断两个node的相对位置来决定左右朝向，如果没有则使用pos
    pub btns: Vec<ButtonKind>, //按钮
    pub jux_sgn: Option<String>, //并置信號機
    pub dif_sgn: Option<String>, //差置信号机
}

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Debug, Display, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
enum JointKind {
    Normal,    //普通
    Clearance, //侵限绝缘
    End,       //尽头
    Empty,     //无
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum NodeKind {
    Mainline, //正线股道
    Siding,   //站线股道
    Siding18, //18道岔以上展现
    Normal,   //一般节点
}

#[derive(Deserialize, Serialize, Debug)]
struct Node {
    pub id: usize,
    pub node_kind: NodeKind,
    pub turnout_id: Vec<usize>, //无岔區段則空，len即为包含道岔数，通過計算得出岔心
    pub track_id: String,       //所属軌道電路， 用於構建 B 關係，特殊區段（接近、 離去）通過id識別
    pub left_adj: Vec<usize>,   //左鄰 用於構建 R 關係
    pub right_adj: Vec<usize>,  //右鄰 用於構建 R 關係
    pub conflicted_nodes: Vec<usize>, //牴觸節點, 用於構建 S 關係
    pub line: ((f64, f64), (f64, f64)), //綫段，用於渲染
    pub joint: (JointKind, JointKind), //兩端是否有絕緣節，用於渲染
}

#[derive(Deserialize, Serialize, Debug)]
pub struct IndButton {
    pub id: String,
    pub kind: ButtonKind,
    pub pos: (f64, f64),
    pub protect_node_id: usize,
}

/// Returns
#[derive(Deserialize, Serialize, Debug)]
struct Station {
    pub title: String,
    pub nodes: Vec<Node>,
    pub signals: Vec<Signal>,
    pub independent_btns: Vec<IndButton>,
}

impl Station {
    pub fn from_yaml(yaml: &str) -> serde_yaml::Result<Self> {
        serde_yaml::from_str(yaml)
    }
}

pub type RawStation = Station;
pub type RawSignal = Signal;
pub type RawNode = Node;
pub type RawSignalKind = SignalKind;
pub type RawSignalMounting = SignalMounting;
pub type RawButtonKind = ButtonKind;
pub type RawJointKind = JointKind;
pub type RawNodeKind = NodeKind;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn test_serialize_station() {
        let mut file = std::fs::File::open("./test_data.yml").unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let x = serde_yaml::from_str::<Station>(&contents).unwrap();
        println!("{:#?}", x);
    }
}