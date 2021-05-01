use super::{instance::Instance, question::Question};
use crate::schema::instance_questions;
use crate::schema::instance_questions::dsl::*;
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Debug, Identifiable, Associations, Queryable, AsChangeset)]
#[belongs_to(Instance)]
pub struct InstanceQuestion {
    pub id: i32,
    pub instance_id: Uuid,
    pub question_id: i32,
    pub score: Option<i32>,
}

impl InstanceQuestion {
    pub fn find_one(sid: i32, conn: &PgConnection) -> QueryResult<Self> {
        instance_questions.find(sid).first(conn)
    }

    pub fn get_by_question(qid: i32, conn: &PgConnection) -> QueryResult<Vec<Self>> {
        instance_questions.filter(question_id.eq(qid)).load(conn)
    }

    pub fn get_question(&self, conn: &PgConnection) -> QueryResult<Question> {
        Question::find_one(self.question_id, conn)
    }

    pub fn update_score(&self, sre: i32, conn: &PgConnection) -> QueryResult<()> {
        diesel::update(self)
            .set(score.eq(sre))
            .execute(conn)?;
        Ok(())
    }
}

#[derive(Insertable, Debug, AsChangeset, Associations)]
#[table_name = "instance_questions"]
pub struct NewInstanceQuestion {
    pub instance_id: Uuid,
    pub question_id: i32,
    pub score: Option<i32>,
}

impl NewInstanceQuestion {
    pub fn create(&self, conn: &PgConnection) -> QueryResult<InstanceQuestion> {
        diesel::insert_into(instance_questions::table)
            .values(self)
            .get_result(conn)
    }
}
