use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum Payload {
    UpdateSignal { sgn_id: String, state: String },
    UpdateTrack { trk_id: i32, state: String },
    TrainMovement { trn_id: i32, pos: (f64, f64) },
    SpawnTrain { trn_id: i32, pos: (f64, f64) },
    RemoveTrain { trn_id: i32, pos: (f64, f64) },
    PlayTTS { content: String },
    Alert { content: String },
}
#[derive(Debug, Serialize)]
pub struct FrameData {
    id: String,
    date: String,
    payload: Payload,
}
