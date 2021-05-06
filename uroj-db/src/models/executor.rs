use crate::schema::executors;
use crate::schema::executors::dsl::*;
use diesel::prelude::*;
use diesel::PgConnection;
use serde::{Deserialize, Serialize};

#[derive(
    Queryable, Debug, Serialize, Deserialize, AsChangeset, Clone, Identifiable, Associations,
)]
pub struct Executor {
    pub id: i32,
    pub addr: String,
}

impl Executor {
    pub fn find_one(eid: i32, conn: &PgConnection) -> QueryResult<Executor> {
        executors.find(eid).first(conn)
    }

    pub fn list_all(conn: &PgConnection) -> QueryResult<Vec<Self>> {
        executors.load(conn)
    }
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[table_name = "executors"]
pub struct NewExecutor {
    pub addr: String,
}

impl NewExecutor {
    pub fn create(&self, conn: &PgConnection) -> QueryResult<Executor> {
        diesel::insert_into(executors::table)
            .values(self)
            .get_result(conn)
    }
}
