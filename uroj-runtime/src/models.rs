use crate::instance::{{Instance, InstanceConfig, InstanceStatus}, PathBtn, fsm::{GameFrame, GlobalStatus, NodeID}, station::{ButtonKind, StationData}};
use crate::raw_station::RawStation;
use crate::{
    get_conn_from_ctx, get_instance_pool_from_ctx, get_producer_from_ctx,
    kafka::{self},
};
use async_graphql::*;
use async_stream::stream;
use futures::Stream;
use futures_util::StreamExt;
use rdkafka::Message;
use std::{str::FromStr, sync::Mutex};
use uroj_db::models::instance::Instance as InstanceModel;

use uuid::Uuid;

#[Object]
impl Query {
    //获取车站布局
    async fn station_layout(&self, ctx: &Context<'_>, id: String) -> Result<StationData> {
        let pool = get_instance_pool_from_ctx(ctx).await;
        let instance = pool.get(&id).ok_or("no instance found")?;
        let data = instance.station.clone();
        Ok(data)
    }

    //获取全局状态
    async fn global_status(&self, ctx: &Context<'_>, id: String) -> Result<GlobalStatus> {
        todo!()
    }
}

#[Object]
impl Mutation {
    //运行
    async fn run(&self, ctx: &Context<'_>, id: String) -> Result<String> {
        let uuid = Uuid::from_str(&id)?;
        let conn = get_conn_from_ctx(ctx);
        let data = InstanceModel::find_one(uuid, &conn)?;
        let cfg = InstanceConfig {
            id: data.id.to_string(),
            title: data.title.clone(),
            player: data.player.clone(),
            token: data.token.clone(),
            station: RawStation::from_yaml(&data.yaml)?,
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

        let producer = get_producer_from_ctx(ctx);
        instance.create_path(start, end, producer).await?;

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

        let producer = get_producer_from_ctx(ctx);
        instance.cancel_path(start, producer).await?;
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

        let producer = get_producer_from_ctx(ctx);
        instance.cancel_path(start, producer).await?;
        Ok(id)
    }

    //區間故障解鎖
    async fn fault_unlock(&self, ctx: &Context<'_>, id: String, node: NodeID) -> Result<String> {
        let pool = get_instance_pool_from_ctx(ctx).await;
        let instance = pool.get(&id).ok_or("no instance found")?;
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

#[Subscription]
impl Subscription {
    async fn game_update<'ctx>(
        &self,
        ctx: &'ctx Context<'_>,
        id: String,
    ) -> Result<impl Stream<Item = GameFrame>> {
        let pool = get_instance_pool_from_ctx(ctx).await;
        let instance = pool.get(&id).ok_or("no instance found")?;
        let kafka_consumer_counter = ctx
            .data::<Mutex<i32>>()
            .expect("Can't get Kafka consumer counter");
        let consumer_group_id = kafka::get_kafka_consumer_group_id(kafka_consumer_counter);
        let consumer = kafka::create_consumer(consumer_group_id);

        Ok(stream! {
            let mut stream = consumer.stream();

            while let Some(value) = stream.next().await {
                yield match value {
                    Ok(message) => {
                        let payload = message.payload().expect("Kafka message should contain payload");
                        let message = String::from_utf8_lossy(payload).to_string();
                        serde_json::from_str(&message).expect("Can't deserialize a frame")
                    }
                    Err(e) => panic!("Error while Kafka message processing: {}", e)
                };
            }
        })
    }
}

pub struct Subscription;

pub struct Mutation;

pub struct Query;
pub type AppSchema = Schema<Query, Mutation, Subscription>;
