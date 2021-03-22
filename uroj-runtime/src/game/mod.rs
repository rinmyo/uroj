pub mod components;
pub mod user;

use std::collections::HashMap;

use chrono::{prelude::*, Duration};
use petgraph::graph::{DiGraph, UnGraph};
use uroj_protocol::station::NewInstanceRequest;

use self::{components::{Route, Signal, Track, TrackNode, Train}, user::User};


trait Executor {
    fn new(config: T) -> Self;
}

struct InstanceBase<'a> {
    instance_id: String,
    start_time: Utc,
    state: GameState,
    white_list: Vec<User>,
    logger: Option<GameLogger>,
    nodes: HashMap<String, TrackNode>,
    r_graph: DiGraph<&'a TrackNode, ()>,
    s_graph: UnGraph<&'a TrackNode, ()>,
    b_graph: UnGraph<&'a TrackNode, ()>,
    tracks: HashMap<String, Track<'a>>,
    trains: HashMap<String, Train<'a>>,
    signals: HashMap<String, Signal>,
}

impl<T: InstanceData> Instance<'_, T> {
    fn from_req(req: NewInstanceRequest) -> Instance<T> {
        
    }
}

struct Exam {
    instance: InstanceBase,
    exam_id: String,
    duration: Duration,
}

struct ChainData {
    
}

struct ExcerciseData {}

struct GameLogger {}

enum GameState {
    Loading,
    GamePlay,
    Pause,
    Result,
}
