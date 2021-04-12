use super::class::Class;
use crate::schema::users;
use crate::schema::users::dsl::*;
use chrono::prelude::*;
use diesel::prelude::*;
use diesel::PgConnection;
use serde::{Deserialize, Serialize};

#[derive(
    Queryable, Debug, Serialize, Deserialize, AsChangeset, Clone, Identifiable, Associations,
)]
#[belongs_to(Class)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub hash_pwd: String,
    pub email: String,
    pub class_id: Option<i32>,
    pub user_role: String,
    pub is_active: bool,
    pub date_joined: NaiveDateTime,
    pub last_login: Option<NaiveDateTime>,
}

impl User {
    pub fn list_all(conn: &PgConnection) -> QueryResult<Vec<Self>> {
        users.load(conn)
    }

    pub fn get_by_username(name: &str, conn: &PgConnection) -> QueryResult<Self> {
        users.filter(username.eq(name)).first(conn)
    }

    pub fn get(uid: i32, conn: &PgConnection) -> QueryResult<Self> {
        users.find(uid).first(conn)
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
            .set(last_login.eq(time))
            .execute(conn)?;
        Ok(())
    }
}

#[derive(Insertable, Serialize, Deserialize, Debug, AsChangeset, Associations)]
#[table_name = "users"]
pub struct NewUser {
    pub username: String,
    pub hash_pwd: String,
    pub email: String,
    pub class_id: Option<i32>,
    pub user_role: String,
    pub is_active: bool,
    pub date_joined: NaiveDateTime,
}

impl NewUser {
    pub fn create(&self, conn: &PgConnection) -> QueryResult<User> {
        diesel::insert_into(users::table)
            .values(self)
            .get_result(conn)
    }
}
