use super::user::User;
use crate::schema::instances;
use crate::schema::instances::dsl::*;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Debug, Identifiable, Associations, Queryable, AsChangeset)]
#[belongs_to(User, foreign_key = "player")]
pub struct Instance {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    pub creator: Option<String>,
    pub player: String,
    pub yaml: String,
    pub curr_state: String,
    pub begin_at: NaiveDateTime,
    pub executor_id: i32,
    pub token: String, //给别人以访问
}

impl Instance {
    pub fn find_one(uid: Uuid, conn: &PgConnection) -> QueryResult<Self> {
        instances.find(uid).first(conn)
    }

    pub fn update_state(&self, new_state: String, conn: &PgConnection) -> QueryResult<()> {
        diesel::update(self)
            .set(curr_state.eq(new_state))
            .execute(conn)?;

        Ok(())
    }
}

#[derive(Insertable, Debug, AsChangeset, Associations)]
#[table_name = "instances"]
pub struct NewInstance {
    pub title: String,
    pub description: Option<String>,
    pub creator: Option<String>,
    pub player: String,
    pub yaml: String,
    pub curr_state: String,
    pub executor_id: i32, //指定
    pub token: String,
}

impl NewInstance {
    pub fn create(&self, conn: &PgConnection) -> QueryResult<Instance> {
        diesel::insert_into(instances::table)
            .values(self)
            .get_result(conn)
    }
}
