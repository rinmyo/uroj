use super::user::User;
use crate::schema::stations;
use crate::schema::stations::dsl::*;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Identifiable, Associations, Queryable, AsChangeset)]
#[belongs_to(User, foreign_key = "author_id")]
pub struct Station {
    pub id: i32,
    pub title: String,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub draft: bool,
    pub author_id: Option<String>,
    pub yaml: String,
}

impl Station {
    pub fn find(sid: i32, conn: &PgConnection) -> QueryResult<Self> {
        stations.find(sid).first(conn)
    }

    pub fn find_by_title(ttl: &str, conn: &PgConnection) -> QueryResult<Self> {
        stations.filter(title.eq(ttl)).first(conn)
    }

    pub fn find_by_author_id(aid: &str, conn: &PgConnection) -> QueryResult<Vec<Self>> {
        stations.filter(author_id.eq(aid)).load(conn)
    }

    pub fn find_by_author(a: &User, conn: &PgConnection) -> QueryResult<Vec<Self>> {
        Station::belonging_to(a).load(conn)
    }

    pub fn list_all(conn: &PgConnection) -> QueryResult<Vec<Self>> {
        stations.load(conn)
    }
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[table_name = "stations"]
pub struct NewStation {
    pub title: String,
    pub description: Option<String>,
    pub draft: bool,
    pub author_id: Option<String>,
    pub yaml: String,
}

impl NewStation {
    pub fn create(&self, conn: &PgConnection) -> QueryResult<Station> {
        diesel::insert_into(stations::table)
            .values(self)
            .get_result(conn)
    }
}
