use crate::schema::instances;
use crate::schema::instances::dsl::*;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use super::user::User;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Insertable, Serialize, Deserialize, Debug, AsChangeset, Associations)]
#[table_name = "instances"]
pub struct NewInstance {
    pub title: String,
    pub description: String,
    pub creator: String,
    pub player: String,
    pub yaml: String,
    pub begin_at: NaiveDateTime,
    pub executor_id: i32,  //指定
}

#[derive(Debug, Serialize, Deserialize, Identifiable, Associations, Queryable, AsChangeset)]
#[belongs_to(User, foreign_key="player")]
pub struct Instance {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub created_at: NaiveDateTime,
    pub creator: String,
    pub player: String,
    pub yaml: String,
    pub begin_at: NaiveDateTime,
    pub token: Uuid, //给别人以访问
    pub executor_id: i32,
}
