use crate::raw_station::RawStation;
use crate::{get_conn_from_ctx, get_instance_pool_from_ctx};
use crate::{
    instance::{
        exam::QuestionsData,
        fsm::NodeID,
        station::{ButtonKind, LayoutData},
        GameFrame, PathBtn, {Instance, InstanceConfig, InstanceStatus},
    },
    raw_station::RawDirection,
};
use async_graphql::*;
use async_stream::stream;
use chrono::Utc;
use futures::Stream;
use std::{collections::HashMap, str::FromStr, time::Duration};
use tokio::time::sleep;
use uroj_db::models::instance::Instance as InstanceModel;

use uuid::Uuid;

#[Object]
impl Query {
    //获取车站布局
    async fn station_layout(&self, ctx: &Context<'_>, id: String) -> Result<LayoutData> {
        let pool = get_instance_pool_from_ctx(ctx).await;
        let instance = pool.get(&id).ok_or("no instance found")?;
        let data = instance.layout.clone();
        Ok(data)
    }

    //获取考题信息
    async fn questions(&self, ctx: &Context<'_>, id: String) -> Result<QuestionsData> {
        let pool = get_instance_pool_from_ctx(ctx).await;
        let instance = pool.get(&id).ok_or("no instance found")?;
        let data = instance.exam.as_ref().ok_or("not exam instance!")?;
        Ok(data.get_questions())
    }
}

#[Object]
impl Mutation {
    //运行
    async fn run(&self, ctx: &Context<'_>, id: String) -> Result<String> {
        let uuid = Uuid::from_str(&id)?;
        let conn = get_conn_from_ctx(ctx);
        let data = InstanceModel::find_one(uuid, &conn)?;
        //time bound
        if Utc::now().naive_local() < data.begin_at {
            return Err(format!("instance {} cannot be initialized yet", id).into());
        }

        let scores = data.get_scores(&conn)?;
        let mut questions = HashMap::new();
        for q in &scores {
            questions.insert(q.question_id, q.get_question(&conn)?);
        }

        let station_yaml = data.get_station(&conn)?.yaml;

        let cfg = InstanceConfig {
            id: data.id.to_string(),
            title: data.title.clone(),
            player: data.player_id.clone(),
            token: data.token.clone(),
            station: RawStation::from_yaml(&station_yaml)?,
            questions: questions,
        };

        let mut pool = get_instance_pool_from_ctx(ctx).await;
        if pool.contains_key(&cfg.id) {
            return Err(format!("instance {} is already running", cfg.id).into());
        }
        let instance = Instance::new(&cfg)?;
        pool.insert(cfg.id.clone(), instance);

        data.update_state(InstanceStatus::Playing.to_string(), &conn)?;

        Ok(cfg.id)
    }

    //结束
    async fn stop(&self, ctx: &Context<'_>, id: String) -> Result<String> {
        let mut instances = get_instance_pool_from_ctx(ctx).await;
        let ins = instances
            .remove(&id)
            .ok_or(format!("not found instance {}", id))?;
        let uuid = Uuid::from_str(&id)?;
        let conn = get_conn_from_ctx(ctx);
        InstanceModel::find_one(uuid, &conn)?
            .update_state(InstanceStatus::Finished.to_string(), &conn)?;
        Ok(id)
    }

    //創建進路
    async fn create_route(
        &self,
        ctx: &Context<'_>,
        id: String,
        input: CreateRouteInput,
    ) -> Result<String> {
        let pool = get_instance_pool_from_ctx(ctx).await;
        let instance = pool.get(&id).ok_or("no instance found")?;

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

        instance.create_path(start, end).await?;

        Ok(id)
    }

    //取消進路
    async fn cancel_route(
        &self,
        ctx: &Context<'_>,
        id: String,
        input: CancelRouteInput,
    ) -> Result<String> {
        let pool = get_instance_pool_from_ctx(ctx).await;
        let instance = pool.get(&id).ok_or("no instance found")?;

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
        let pool = get_instance_pool_from_ctx(ctx).await;
        let instance = pool.get(&id).ok_or("no instance found")?;

        let start = PathBtn {
            id: input.start_sgn,
            kind: input.start_btn,
        };

        instance.cancel_path(start).await?;
        Ok(id)
    }

    //區間故障解鎖
    async fn fault_unlock(&self, ctx: &Context<'_>, id: String, node: NodeID) -> Result<String> {
        let pool = get_instance_pool_from_ctx(ctx).await;
        let instance = pool.get(&id).ok_or("no instance found")?;
        // instance.create_path();

        Ok(id)
    }

    async fn spawn_train(&self, ctx: &Context<'_>, id: String, at: NodeID) -> Result<String> {
        let mut pool = get_instance_pool_from_ctx(ctx).await;
        let instance = pool.get_mut(&id).ok_or("no instance found")?;
        let mut fsm = instance.fsm.lock().await;
        let topo = instance.topo.clone();

        let arc_train = fsm.spawn_train(at).await;
        let next_node = instance
            .next_route_node(at, &RawDirection::Left)
            .await
            .or(instance.next_route_node(at, &RawDirection::Right).await);

        let arc_fsm = instance.fsm.clone();
        tokio::spawn(async move {
            sleep(Duration::from_millis(1)).await;
            if let Some(node) = next_node {
                let fsm = arc_fsm.lock().await;
                let mut train = arc_train.lock().await;

                train.try_move_to(node, &fsm, &topo);
            };
        });

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

#[Subscription]
impl Subscription {
    async fn game_update<'ctx>(
        &self,
        ctx: &'ctx Context<'_>,
        id: String,
    ) -> Result<impl Stream<Item = GameFrame>> {
        let pool = get_instance_pool_from_ctx(ctx).await;
        let instance = pool.get(&id).ok_or("no instance found")?;
        let mut stream = instance.tx.subscribe();

        Ok(stream! {
            while let Ok(value) = stream.recv().await {
                yield value;
            }
        })
    }
}

pub struct Subscription;

pub struct Mutation;

pub struct Query;
pub type AppSchema = Schema<Query, Mutation, Subscription>;
