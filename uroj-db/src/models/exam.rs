use crate::schema::exams;
use crate::schema::exams::dsl::*;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::PgConnection;
use serde::{Deserialize, Serialize};

use super::question::Question;

#[derive(
    Queryable, Debug, Serialize, Deserialize, AsChangeset, Clone, Identifiable, Associations,
)]
pub struct Exam {
    pub id: i32,
    pub title: String,
    pub finish_at: NaiveDateTime,
}

impl Exam {
    pub fn find_one(eid: i32, conn: &PgConnection) -> QueryResult<Self> {
        exams.find(eid).first(conn)
    }

    pub fn list_all(conn: &PgConnection) -> QueryResult<Vec<Self>> {
        exams.load(conn)
    }

    pub fn get_questions(&self, conn: &PgConnection) -> QueryResult<Vec<Question>> {
        Question::belonging_to(self).load(conn)
    }
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[table_name = "exams"]
pub struct NewExam {
    pub id: i32,
    pub title: String,
    pub finish_at: NaiveDateTime,
}

impl NewExam {
    pub fn create(exam: &Self, conn: &PgConnection) -> QueryResult<Exam> {
        diesel::insert_into(exams::table)
            .values(exam)
            .get_result(conn)
    }
}
