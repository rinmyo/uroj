use super::class::Class;
use crate::schema::users;
use crate::schema::users::dsl::*;
use chrono::prelude::*;
use diesel::{dsl::any, prelude::*};
use diesel::PgConnection;
use serde::{Deserialize, Serialize};

#[derive(
    Queryable, Debug, Serialize, Deserialize, AsChangeset, Clone, Identifiable, Associations,
)]
#[belongs_to(Class)]
pub struct User {
    pub id: String,
    pub hash_pwd: String,
    pub email: String,
    pub class_id: Option<i32>,
    pub user_role: String,
    pub is_active: bool,
    pub joined_at: NaiveDateTime,
    pub last_login_at: Option<NaiveDateTime>,
}

impl User {
    pub fn list_all(conn: &PgConnection) -> QueryResult<Vec<Self>> {
        users.load(conn)
    }

    pub fn get(uid: &str, conn: &PgConnection) -> QueryResult<Self> {
        users.find(uid).first(conn)
    }

    pub fn find_many(ids: &[String], conn: &PgConnection) -> QueryResult<Vec<Self>> {
        users.filter(id.eq(any(ids))).load(conn)
    }

    pub fn get_by_class_id(cid: i32, conn: &PgConnection) -> QueryResult<Vec<Self>> {
        users.filter(class_id.eq(cid)).load(conn)
    }

    pub fn update_password_hash(&self, new_hash: String, conn: &PgConnection) -> QueryResult<()> {
        diesel::update(self)
            .set(hash_pwd.eq(new_hash))
            .execute(conn)?;
        Ok(())
    }

    pub fn update_active(&self, active: bool, conn: &PgConnection) -> QueryResult<()> {
        diesel::update(self)
            .set(is_active.eq(active))
            .execute(conn)?;
        Ok(())
    }

    pub fn update_last_login(&self, time: NaiveDateTime, conn: &PgConnection) -> QueryResult<()> {
        diesel::update(self)
            .set(last_login_at.eq(time))
            .execute(conn)?;
        Ok(())
    }
}

#[derive(Insertable, Serialize, Deserialize, Debug, AsChangeset, Associations)]
#[table_name = "users"]
pub struct NewUser {
    pub id: String,
    pub hash_pwd: String,
    pub email: String,
    pub class_id: Option<i32>,
    pub user_role: String,
}

impl NewUser {
    pub fn create(&self, conn: &PgConnection) -> QueryResult<User> {
        diesel::insert_into(users::table)
            .values(self)
            .get_result(conn)
    }
}
