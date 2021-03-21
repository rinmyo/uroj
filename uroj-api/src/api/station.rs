use serde::Deserialize;

///
/// 拿到输入后先验证 yaml 有效性，解析失败直接返回错误
/// 若成功则會進行車站有效性判斷，判斷内容
/// 1. R图无孤立點  -> 孤立點的度為 0，即 adjoint_nodes 為空，不严格
/// 2. R, S 關係的對稱性验证，S 是严格的，R不严格
/// 3. 验证道岔区段组第一性质定理，严格
/// 4. 验证各种区段的度，尽头 1，严格
/// 以上严格项不满足则返回错误，不严格返回警告

#[derive(Debug, PartialEq, Deserialize)]
pub struct NodeData {
    pub id: i32,
    pub turnout_id: Option<Vec<i32>>, //无岔區段則無，
    pub track_id: String, //所属区段， 用於構建 B 關係，特殊區段（接近、 離去）通過id識別
    pub adjoint_nodes: Vec<i32>, //鄰接節點， 用於構建 R 關係
    pub conflicted_nodes: Vec<i32>, //牴觸節點, 用於構建 S 關係
    pub line: ((f64, f64), (f64, f64)), //綫段，用於渲染
    pub joint: (bool, bool), //兩端是否有絕緣節，用於渲染
}

//位置是其锚点的位置，即防护区段的起点
#[derive(Debug, PartialEq, Deserialize)]
pub struct SignalData {
    pub id: String,
    pub pos: (f64, f64),      //位置
    pub dir: String,          //朝向
    pub sig_type: String,     //信號類型
    pub sig_mnt: String,      //安裝方式
    pub btn_id: String,       //信号机按钮编号
    pub protect_node_id: i32, //防护node 的 ID
}

// 新車站的請求結構
#[derive(Debug, PartialEq, Deserialize)]
pub struct NewStationData {
    pub title: String,
    pub author: String,
    pub nodes: Vec<NodeData>,
    pub signals: Vec<SignalData>,
}
