use crate::schema::instance_configs;
use crate::schema::instance_configs::dsl::*;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Insertable, Serialize, Deserialize, Debug, AsChangeset, Associations)]
#[table_name = "instance_configs"]
pub struct NewInstanceConfig {
    pub title: String,
    pub description: String,
    pub creator: String,
    pub player: String,
    pub yaml: String,
    pub begin_at: NaiveDateTime,
    pub token: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Identifiable, Associations, Queryable, AsChangeset)]
pub struct InstanceConfig {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub created_at: NaiveDateTime,
    pub creator: String,
    pub player: String,
    pub yaml: String,
    pub begin_at: NaiveDateTime,
    pub token: Uuid, //给别人以访问
}
