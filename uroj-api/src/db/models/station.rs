use super::user::User;
use crate::db::schema::stations;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Identifiable, Associations)]
#[belongs_to(User, foreign_key = "author_id")]
pub struct Station {
    pub id: i32,
    pub title: String,
    pub discription: Option<String>,
    pub created: NaiveDateTime,
    pub updated: NaiveDateTime,
    pub draft: bool,
    pub author_id: Option<i32>,
}
