use async_graphql::*;
use async_stream::try_stream;
use futures::Stream;
use serde::{Deserialize, Serialize};
use strum_macros::*;

use super::borrow_instance_from_ctx;
use crate::{
    game::{
        components::{NodeID, SignalStatus},
        instance::PathBtn,
    },
    get_id_from_ctx, get_instance_pool_from_ctx,
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
        todo!()
    }

    //获取全局状态
    async fn global_status(&self, ctx: &Context<'_>, id: String) -> Result<GlobalStatus> {
        todo!()
    }
}

#[derive(Union)]
pub(crate) enum GameFrame {
    UpdateSignal(UpdateSignal),
    UpdateNode(UpdateNode),
}

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Enum, Display, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum NodeStatus {
    Lock,
    Vacant,
    Occupied,
}

#[derive(SimpleObject, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct UpdateSignal {
    pub(crate) id: String,
    state: SignalStatus,
}

#[derive(SimpleObject, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct UpdateNode {
    id: NodeID,
    state: NodeStatus,
}

pub struct Mutation;

#[Object]
impl Mutation {
    //结束
    async fn stop(&self, ctx: &Context<'_>, id: String) -> Result<String> {
        let mut instance = get_instance_pool_from_ctx(ctx);
        match instance.remove(&id) {
            Some(i) => Ok(i.info.id),
            None => Err(format!("not found instance {}", id).into()),
        }
    }

    //創建進路
    async fn create_route(
        &self,
        ctx: &Context<'_>,
        id: String,
        input: CreateRouteInput,
    ) -> Result<String> {
        let instance = borrow_instance_from_ctx(ctx, &id)?;

        let start = PathBtn {
            id: input.start_sgn,
            kind: input.start_btn,
        };
        let end = PathBtn {
            kind: input.end_btn,
            id: match input.end_btn {
                ButtonKind::Train | ButtonKind::Shunt => input.end_sgn.ok_or("error input")?,
                ButtonKind::LZA => input.end_ind_btn.ok_or("error input")?,
                _ => return Err("no valid route".into()),
            },
        };

        instance.create_path(start, end)?;

        Ok(id)
    }

    //取消進路
    async fn cancel_route(
        &self,
        ctx: &Context<'_>,
        id: String,
        input: CancelRouteInput,
    ) -> Result<String> {
        let instance = borrow_instance_from_ctx(ctx, &id)?;

        let start = PathBtn {
            id: input.start_sgn,
            kind: input.start_btn,
        };

        instance.cancel_path(start).await?;
        Ok(id)
    }

    //人工解鎖
    async fn manually_unlock(
        &self,
        ctx: &Context<'_>,
        id: String,
        input: CancelRouteInput,
    ) -> Result<String> {
        let instance = borrow_instance_from_ctx(ctx, &id)?;

        let start = PathBtn {
            id: input.start_sgn,
            kind: input.start_btn,
        };

        instance.cancel_path(start).await?;
        Ok(id)
    }

    //區間故障解鎖
    async fn fault_unlock(&self, ctx: &Context<'_>, id: String, node: NodeID) -> Result<String> {
        let instance = borrow_instance_from_ctx(ctx, &id)?;
        // instance.create_path();

        Ok(id)
    }
}

//tag 可以是信號機ID或者獨立ButtonID
#[derive(InputObject)]
struct CreateRouteInput {
    start_btn: ButtonKind,
    start_sgn: String,
    end_btn: ButtonKind,
    end_sgn: Option<String>, //from independent button or signal button
    end_ind_btn: Option<String>,
}

#[derive(InputObject)]
struct CancelRouteInput {
    start_btn: ButtonKind,
    start_sgn: String,
}

// pub struct Subscription;

// #[Subscription]
// impl Subscription {
//     async fn game_update<'ctx>(
//         &self,
//         ctx: &'ctx Context<'_>,
//         id: String,
//     ) -> impl Stream<Item = Result<GameFrame>> + 'ctx {
//         todo!()
//     }
// }

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum ButtonKind {
    Pass,  //通過按鈕
    Shunt, //調車按鈕
    Train, //列車按鈕（接發車）
    Guide, //引導按鈕
    LZA,   //列車終端按鈕
}

pub type AppSchema = Schema<Query, EmptyMutation, EmptySubscription>;
