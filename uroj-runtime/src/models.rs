use async_graphql::*;
use async_stream::try_stream;
use futures::Stream;
use serde::{Deserialize, Serialize};
use uroj_common::rpc::GamerRole;

use super::borrow_instance_from_ctx;
use crate::{
    game::components::{NodeID, NodeStatus, SignalStatus},
    get_id_from_ctx,
};

#[derive(SimpleObject)]
pub struct Point {
    x: f64,
    y: f64,
}

#[derive(SimpleObject)]
pub struct NodeData {
    pub node_id: usize,
    pub track_id: String,
    pub start: Point,
    pub end: Point,
    pub start_joint: String, //始端绝缘节
    pub end_joint: String,   //终端绝缘节
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum Direction {
    LeftUp,
    LeftDown,
    RightUp,
    RightDown,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum SignalKind {
    HomeSignal,     //進站信號機
    StartingSignal, //出站信號機
    ShuntingSignal, //調車信號機
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum SignalMounting {
    PostMounting,   //高柱
    GroundMounting, //矮柱
}

#[derive(SimpleObject)]
pub struct SignalData {
    pub signal_id: String,
    pub pos: Point,              //位置
    pub dir: Direction,          //朝向
    pub sgn_type: SignalKind,    //信號類型
    pub sgn_mnt: SignalMounting, //安裝方式
    pub protect_node_id: usize,  //防护node 的 ID
}

// front models
#[derive(SimpleObject)]
pub struct StationData {
    pub station_name: String,
    pub nodes: Vec<NodeData>,
    pub signals: Vec<SignalData>,
}

#[derive(SimpleObject)]
pub struct GlobalStatus {
    pub nodes: Vec<UpdateNode>,
    pub signals: Vec<UpdateSignal>,
}

pub(crate) struct Query;

#[Object]
impl Query {
    //获取车站布局
    async fn station_layout(&self, ctx: &Context<'_>, id: String) -> Result<StationData> {
        4
    }

    //获取全局状态
    async fn global_status(&self, ctx: &Context<'_>, id: String) -> Result<GlobalStatus> {
        4
    }
}

#[derive(Union)]
pub(crate) enum GameFrame {
    UpdateSignal(UpdateSignal),
    UpdateNode(UpdateNode),
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

pub struct Mutation;

#[Object]
impl Mutation {
    //开始
    async fn start(&self, ctx: &Context<'_>, id: String) -> Result<()> {
        let user = get_id_from_ctx(ctx)?;
        let instance = borrow_instance_from_ctx(ctx, &id)?;
        match instance.user_role(&user) {
            Some(GamerRole::Operator) | Some(GamerRole::Player) => {
                //start game
                Ok(())
            }
            _ => Err("Forbidden".into())
        }
    }

    //暫停
    async fn pause(&self, ctx: &Context<'_>, id: String) -> Result<()> {
        let instance = borrow_instance_from_ctx(ctx, &id)?;

        Ok(())
    }

    //结束
    async fn stop(&self, ctx: &Context<'_>, id: String) -> Result<()> {
        let instance = borrow_instance_from_ctx(ctx, &id)?;

        Ok(())
    }

    //創建進路
    async fn create_route(
        &self,
        ctx: &Context<'_>,
        id: String,
        input: CreateRouteInput,
    ) -> Result<()> {
        let instance = borrow_instance_from_ctx(ctx, &id)?;
        instance.create_path();

        Ok(())
    }

    //取消進路
    async fn cancel_route(
        &self,
        ctx: &Context<'_>,
        id: String,
        start_signal: String,
    ) -> Result<()> {
        let instance = borrow_instance_from_ctx(ctx, &id)?;
        instance.create_path();

        Ok(())
    }

    //人工解鎖
    async fn manually_unlock(
        &self,
        ctx: &Context<'_>,
        id: String,
        start_signal: String,
    ) -> Result<()> {
        let instance = borrow_instance_from_ctx(ctx, &id)?;
        instance.create_path();

        Ok(())
    }

    //區間故障解鎖
    async fn fault_unlock(&self, ctx: &Context<'_>, id: String, node: NodeID) -> Result<()> {
        let instance = borrow_instance_from_ctx(ctx, &id)?;
        instance.create_path();

        Ok(())
    }
}
#[derive(InputObject)]
struct CreateRouteInput {
    start_btn: ButtonKind,
    start_signal: String,
    end_btn: ButtonKind,
    end_node: usize,
}

pub struct Subscription;

#[Subscription]
impl Subscription {
    async fn game_update<'ctx>(
        &self,
        ctx: &'ctx Context<'_>,
        id: String,
    ) -> impl Stream<Item = Result<GameFrame>> + 'ctx {
        try_stream! {
            let instance = borrow_instance_from_ctx(ctx, id)?;

        }
    }
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum ButtonKind {
    Pass,  //通過按鈕
    Shunt, //調車按鈕
    Train, //列車按鈕（接發車）
    Guide, //引導按鈕
}

pub type AppSchema = Schema<Query, EmptyMutation, Subscription>;
