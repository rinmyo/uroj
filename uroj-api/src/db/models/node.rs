use serde::{Deserialize, Serialize};
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
