use async_graphql::*;
use chrono::NaiveDateTime;
use uroj_db::models::instance::Instance as InstanceData;
use uroj_db::models::station::Station as StationData;

use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum_macros::*;

use crate::get_conn_from_ctx;

use super::station::Station;

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct Instance {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    pub creator: Option<String>,
    pub player: String,
    pub station_id: i32,
    pub curr_state: InstanceStatus,
    pub begin_at: NaiveDateTime,
    pub executor_id: i32,
    pub token: String, //给别人以访问
}


#[ComplexObject]
impl Instance {
    async fn station(&self, ctx: &Context<'_>) -> Station {
        let data = &StationData::find(self.station_id, &get_conn_from_ctx(ctx))
            .expect("cannot query stations");
        data.into()
    }
}

impl From<&InstanceData> for Instance {
    fn from(data: &InstanceData) -> Self {
        Instance {
            id: data.id.to_string(),
            title: data.title.clone(),
            description: data.description.clone(),
            created_at: data.created_at,
            creator: data.creator_id.clone(),
            player: data.player_id.clone(),
            station_id: data.station_id,
            curr_state: InstanceStatus::from_str(&data.curr_state)
                .expect(&format!("cannot convert {} to Status", &data.curr_state)),
            begin_at: data.begin_at,
            executor_id: data.executor_id,
            token: data.token.clone(),
        }
    }
}

#[derive(InputObject)]
pub(crate) struct InstanceInput {
    pub title: String,
    pub description: Option<String>,
    pub player: String,
    pub station_id: i32,  //指定station的副本
    pub executor_id: i32, //指定
}

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Enum, Display, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum InstanceStatus {
    Prestart, //启动前
    Playing,  //使用中
    Finished, //已结束
}

impl Default for InstanceStatus {
    fn default() -> Self {
        InstanceStatus::Prestart
    }
}
