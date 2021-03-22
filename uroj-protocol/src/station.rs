enum InstanceData {
    Exam{exam_id: String, },
    Chain,
    Exercise
}

pub struct NewInstanceRequest {
    pub instance_type: InstanceType,
    pub station_id: String,
    pub nodes: Vec<Node>,
    pub signals: Vec<Signal>,
}
