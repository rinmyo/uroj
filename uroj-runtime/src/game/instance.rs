use std::collections::HashMap;

use chrono::{prelude::*, Duration};
use petgraph::graph::{DiGraph, NodeIndex, UnGraph};
use petgraph::algo;

use super::{components::Path, station::StationData};

use super::{components::{Signal, Track, TrackNode, Train}, user::User};
pub(crate) struct InstanceBase<'a> {
    instance_id: String,
    start_time: Utc,
    state: GameState,
    white_list: Vec<User>,
    logger: Option<GameLogger>,
    nodes: HashMap<u32, TrackNode>,
    r_graph: DiGraph<&'a TrackNode, ()>,
    s_graph: UnGraph<&'a TrackNode, ()>,
    b_graph: UnGraph<&'a TrackNode, ()>,
    tracks: HashMap<String, Track<'a>>,
    trains: HashMap<String, Train<'a>>,
    signals: HashMap<String, Signal>,
    station: StationData,
}

impl InstanceBase<'_> {
    fn load_graph() {
        
    }

    fn available_path(&self, start: NodeIndex, goal: NodeIndex) -> Option<Path> {
        let (len, nodes) = algo::astar(
            &self.r_graph, 
            start, 
            |node| node == goal, 
            |_|1, 
            |_|0
        )?;

        let path = nodes.iter().map(|t| {
            self.r_graph[t.clone()]
        }).collect::<Path>();

        Some(path)
    }
}

struct ChainInstance<'a> {
    instance: InstanceBase<'a>,
}

struct ExcerciseInstance<'a> {
    instance: InstanceBase<'a>,
}

struct GameLogger {}

enum GameState {
    Loading,
    GamePlay,
    Pause,
    Result,
}
