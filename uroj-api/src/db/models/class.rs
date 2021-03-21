use crate::db::schema::classes;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Debug, Serialize, Deserialize, AsChangeset, Clone, Identifiable, Associations)]
#[table_name = "classes"]
pub(crate) struct Class {
    pub(crate) id: i32,
    pub(crate) name: String,
}