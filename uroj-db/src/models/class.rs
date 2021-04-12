use crate::schema::classes;
use crate::schema::classes::dsl::*;
use diesel::prelude::*;
use diesel::PgConnection;
use serde::{Deserialize, Serialize};

#[derive(
    Queryable, Debug, Serialize, Deserialize, AsChangeset, Clone, Identifiable, Associations,
)]
#[table_name = "classes"]
pub struct Class {
    pub id: i32,
    pub class_name: String,
}

impl Class {
    pub fn find(cid: i32, conn: &PgConnection) -> QueryResult<Class> {
        classes.find(cid).first(conn)
    }

    pub fn list_all(conn: &PgConnection) -> QueryResult<Vec<Self>> {
        classes.load(conn)
    }

    pub fn update_class_name(&self, new_name: &str, conn: &PgConnection) -> QueryResult<()> {
        diesel::update(self)
            .set(class_name.eq(new_name))
            .execute(conn)?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[table_name = "classes"]
pub struct NewClass {
    pub class_name: String,
}

impl NewClass {
    pub fn create(class: &Self, conn: &PgConnection) -> QueryResult<Class> {
        diesel::insert_into(classes::table)
            .values(class)
            .get_result(conn)
    }
}
