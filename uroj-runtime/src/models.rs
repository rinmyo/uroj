use async_graphql::*;
use async_stream::try_stream;
use futures::Stream;
use serde::{Deserialize, Serialize};
use uroj_common::station::{Direction, SignalKind, SignalMounting};

use super::get_instance_from_ctx;
use crate::game::components::{NodeID, NodeStatus, SignalStatus};

#[derive(SimpleObject)]
pub struct NodeData {
    pub node_id: usize,
    pub track_id: String,
    pub line: ((f64, f64), (f64, f64)), //綫段，用於渲染
    pub joint: (String, String),        //兩端是否有絕緣節，用於渲染
}

#[derive(SimpleObject)]
pub struct SignalData {
    pub signal_id: String,
    pub pos: (f64, f64),         //位置
    pub dir: Direction,          //朝向
    pub sgn_type: SignalKind,    //信號類型
    pub sgn_mnt: SignalMounting, //安裝方式
    pub protect_node_id: usize,    //防护node 的 ID
}

// front models
#[derive(SimpleObject)]
pub struct StationData {
    pub station_name: String,
    pub nodes: Vec<NodeData>,
    pub signals: Vec<SignalData>,
}

pub(crate) struct Query;

#[Object]
impl Query {
    async fn d(&self) -> i32 {
        4
    }
}

#[derive(Union)]
pub(crate) enum GameFrame {
    UpdateSignal(UpdateSignal),
    UpdateNode(UpdateNode),
    SendNews(SendNews),
}

#[derive(SimpleObject, Clone, Eq, PartialEq, Serialize, Deserialize)]
struct UpdateSignal {
    id: String,
    state: SignalStatus,
}

#[derive(SimpleObject, Clone, Eq, PartialEq, Serialize, Deserialize)]
struct UpdateNode {
    id: NodeID,
    state: NodeStatus,
}

#[derive(SimpleObject, Clone, Eq, PartialEq, Serialize, Deserialize)]
struct SendNews {
    level: NewsLevel,
    content: String,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
enum NewsLevel {
    Warning,
    Info,
    Error,
}

pub struct Subscription;

#[Subscription]
impl Subscription {
    async fn subscribe_game_update<'ctx>(
        &self,
        ctx: &'ctx Context<'_>,
        id: String,
    ) -> impl Stream<Item = Result<GameFrame>> + 'ctx {
        try_stream! {
            let instance = get_instance_from_ctx(ctx, id).ok_or("instance not found")?;
            
        }
    }
}

pub struct Mutation;

#[Object]
impl Mutation {}

pub type AppSchema = Schema<Query, EmptyMutation, Subscription>;
