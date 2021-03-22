use serde::{Deserialize, Serialize};

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
    pub pos: (f64, f64),         //位置 渲染用
    pub dir: Direction,          //朝向 渲染用
    pub sig_type: SignalType,    //信號類型 渲染用
    pub sig_mnt: SignalMounting, //安裝方式 渲染用
    pub protect_node_id: i32,    //防护node 的 ID 业务&渲染
}
