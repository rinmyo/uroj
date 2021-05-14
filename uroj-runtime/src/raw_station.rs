use async_graphql::*;
use serde::{Deserialize, Serialize};
use strum_macros::*;

#[derive(Eq, PartialEq, Deserialize, Serialize, Debug, Clone, Enum, Copy)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum SignalKind {
    HomeSignal,     //進站信號機
    StartingSignal, //出站信號機
    ShuntingSignal, //調車信號機
}
#[derive(Deserialize, Serialize, Debug, Enum, Copy, Clone, Eq, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum SignalMounting {
    PostMounting,   //高柱
    GroundMounting, //矮柱
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum ButtonKind {
    Pass,  //通過按鈕
    Shunt, //調車按鈕
    Train, //列車按鈕（接發車）
    Guide, //引導按鈕
    LZA,   //列車終端按鈕
}

#[derive(Deserialize, Serialize, Debug, Enum, Copy, Clone, Eq, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum NodeSide {
    Upper,
    Under,
}

#[derive(Deserialize, Serialize, Debug, Enum, Copy, Clone, Eq, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum Direction {
    Left,
    Right,
}

impl Direction {
    pub(crate) fn reverse(&self) -> Self {
        match self {
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct Signal {
    pub(crate) id: String,
    pub(crate) pos: Option<(f64, f64)>, //位置 渲染用
    pub(crate) side: NodeSide,          // 上下朝向 渲染
    pub(crate) dir: Option<Direction>,
    pub(crate) sgn_kind: SignalKind,    //信號類型 渲染用
    pub(crate) sgn_mnt: SignalMounting, //安裝方式 渲染用
    pub(crate) protect_node_id: usize,  //防护node 的 ID 业务&渲染，防护node指的是其所防护的node
    pub(crate) toward_node_id: usize,
    pub(crate) btns: Vec<ButtonKind>,   //按钮
    pub(crate) jux_sgn: Option<String>, //并置信號機
    pub(crate) dif_sgn: Option<String>, //差置信号机
}

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Debug, Display, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum JointKind {
    Normal,    //普通
    Clearance, //侵限绝缘
    End,       //尽头
    Empty,     //无
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum NodeKind {
    Mainline, //正线股道
    Siding,   //站线股道
    Siding18, //18道岔以上展现
    Normal,   //一般节点
}

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct Node {
    pub(crate) id: usize,
    pub(crate) node_kind: NodeKind,
    pub(crate) turnout_id: Vec<usize>, //无岔區段則空，len即为包含道岔数，通過計算得出岔心
    pub(crate) track_id: String, //所属軌道電路， 用於構建 B 關係，特殊區段（接近、 離去）通過id識別
    pub(crate) left_adj: Vec<usize>, //左鄰 用於構建 R 關係
    pub(crate) right_adj: Vec<usize>, //右鄰 用於構建 R 關係
    pub(crate) conflicted_nodes: Vec<usize>, //牴觸節點, 用於構建 S 關係
    pub(crate) line: ((f64, f64), (f64, f64)), //綫段，用於渲染
    pub(crate) joint: (JointKind, JointKind), //兩端是否有絕緣節，用於渲染
}

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct IndButton {
    pub(crate) id: String,
    pub(crate) kind: ButtonKind,
    pub(crate) pos: (f64, f64),
    pub(crate) protect_node_id: usize,
}

/// Returns
#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct Station {
    pub(crate) title: String,
    pub(crate) nodes: Vec<Node>,
    pub(crate) signals: Vec<Signal>,
    pub(crate) independent_btns: Vec<IndButton>,
}

impl Station {
    pub(crate) fn from_json(yaml: &str) -> serde_json::Result<Self> {
        serde_json::from_str(yaml)
    }
}

pub(crate) type RawStation = Station;
pub(crate) type RawSignal = Signal;
pub(crate) type RawNode = Node;
pub(crate) type RawSignalKind = SignalKind;
pub(crate) type RawSignalMounting = SignalMounting;
pub(crate) type RawButtonKind = ButtonKind;
pub(crate) type RawJointKind = JointKind;
pub(crate) type RawNodeKind = NodeKind;
pub(crate) type RawNodeSide = NodeSide;
pub(crate) type RawDirection = Direction;

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
