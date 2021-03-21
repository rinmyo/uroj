use serde::{Deserialize};
#[derive(Debug, PartialEq, Deserialize)]
pub struct NewInstanceData {
    pub name: Option<String>,
    pub instance_type: Option<String>,
    pub start_time: Option<String>,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct InstanceData {
    pub uuid: String,
    pub name: String,
    pub instance_type: String,
    pub ws_uri: String, //實例的 websocket api
}
