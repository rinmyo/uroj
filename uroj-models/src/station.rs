use serde::{Deserialize, Serialize};
use wither::bson::{doc, oid::ObjectId};
use wither::prelude::*;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub id: i32,
    pub turnout_id: Vec<i32>, //无岔區段則空，len即为包含道岔数，通過計算得出道岔中心
    pub track_id: String,     //所属軌道電路， 用於構建 B 關係，特殊區段（接近、 離去）通過id識別
    pub adjoint_nodes: Vec<i32>, //鄰接節點， 用於構建 R 關係
    pub conflicted_nodes: Vec<i32>, //牴觸節點, 用於構建 S 關係
    pub line: ((f64, f64), (f64, f64)), //綫段，用於渲染
    pub joint: (bool, bool),  //兩端是否有絕緣節，用於渲染
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Left,  //向左
    Right, //向右
}

//信號機類型
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalType {
    HomeSignal,     //進站信號機
    StartingSignal, //出站信號機
    ShuntingSignal, //調車信號機
}

//信號機安裝方式
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalMounting {
    PostMounting,   //高柱
    GroundMounting, //矮柱
}

//在 frontend 通过防护 node 的角度作为其倾角
//位置是其锚点的位置，即防护区段的起点
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Signal {
    pub id: String,
    pub pos: (f64, f64),         //位置
    pub dir: Direction,          //朝向
    pub sig_type: SignalType,    //信號類型
    pub sig_mnt: SignalMounting, //安裝方式
    pub btn_id: String,          //信号机按钮编号
    pub protect_node_id: i32,    //防护node 的 ID
}

#[derive(Debug, Model, Serialize, Deserialize)]
#[model(index(keys = r#"doc!{"title": 1}"#, options = r#"doc!{"unique": true}"#))]
pub struct Station {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub title: String,
    pub author: String,
    pub nodes: Vec<Node>,
    pub signals: Vec<Signal>,
}
