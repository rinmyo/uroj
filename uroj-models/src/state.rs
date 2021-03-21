use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum SignalState {
    Green,
    Red,
    Black,
    Yellow,
    Blue,
    FlashYellow,
}

#[derive(Debug, Serialize)]
pub enum TrackState {
    Occupied,  //佔用
    Locked,    //鎖定
    Vacant,    //出清
    Defective, //分路不良
}
