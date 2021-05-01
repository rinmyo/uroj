use super::{exam::Exam, instance_question::InstanceQuestion};
use crate::schema::questions;
use crate::schema::questions::dsl::*;
use serde::{Deserialize, Serialize};
use diesel::prelude::*;

#[derive(Debug, Identifiable, Associations, Queryable, AsChangeset, Serialize, Deserialize)]
#[belongs_to(Exam)]
pub struct Question {
    pub id: i32,
    pub title: String,
    pub from_node: i32,
    pub to_node: i32,
    pub err_node: Vec<i32>,
    pub err_sgn: bool,
    pub exam_id: i32,
    pub station_id: i32,
    pub score: i32,
}

impl Question {
    pub fn find_one(qid: i32, conn: &PgConnection) -> QueryResult<Self> {
        questions.find(qid).first(conn)
    }

    pub fn get_scores(&self, conn: &PgConnection) -> QueryResult<Vec<InstanceQuestion>> {
        InstanceQuestion::get_by_question(self.id, conn)
    }
}

#[derive(Insertable, Debug, AsChangeset, Associations)]
#[table_name = "questions"]
pub struct NewQuestion {
    pub title: String,
    pub from_node: i32,
    pub to_node: i32,
    pub err_node: Vec<i32>,
    pub err_sgn: bool,
    pub exam_id: i32,
    pub station_id: i32,
    pub score: i32,
}

impl NewQuestion {
    pub fn create(&self, conn: &PgConnection) -> QueryResult<Question> {
        diesel::insert_into(questions::table)
            .values(self)
            .get_result(conn)
    }
}
