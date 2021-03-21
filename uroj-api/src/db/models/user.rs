use super::class::Class;
use crate::db::schema::userinfo;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Queryable, Debug, Serialize, Deserialize, AsChangeset, Clone, Identifiable, Associations,
)]
#[table_name = "userinfo"]
#[belongs_to(Class)]
pub(crate) struct User {
    pub(crate) id: i32,
    pub(crate) username: String,
    pub(crate) password: String,
    pub(crate) email: String,
    pub(crate) class_id: Option<i32>,
    pub(crate) is_admin: bool,
    pub(crate) is_active: bool,
    pub(crate) date_joined: NaiveDateTime,
    pub(crate) last_login: Option<NaiveDateTime>,
}

#[derive(Insertable, Serialize, Deserialize, Debug, AsChangeset)]
#[table_name = "userinfo"]
pub(crate) struct NewUser {
    pub(crate) username: String,
    pub(crate) password: String,
    pub(crate) email: String,
    pub(crate) class_id: Option<i32>,
    pub(crate) is_admin: bool,
    pub(crate) is_active: bool,
    pub(crate) date_joined: NaiveDateTime,
}
