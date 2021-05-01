use async_graphql::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use uroj_db::models::instance_question::InstanceQuestion;
use uroj_db::models::question::Question as QuestionModel;

use crate::{get_conn_from_ctx, instance::fsm::NodeID};

type IQID = i32;
#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct Question {
    id: IQID,
    title: String,
    from: NodeID,
    to: NodeID,
    err_node: Vec<NodeID>,
    err_sgn: bool,
    score: i32,
}

impl Question {
    fn new(iqid: IQID, qm: &QuestionModel) -> Self {
        Question {
            id: iqid,
            title: qm.title.clone(),
            from: qm.from_node as NodeID,
            to: qm.to_node as NodeID,
            err_node: qm.err_node.iter().map(|n| *n as NodeID).collect(),
            err_sgn: qm.err_sgn,
            score: qm.score,
        }
    }

    fn sync_score_to_db(&self, score: i32, ctx: &Context<'_>) -> Result<(), String> {
        let conn = get_conn_from_ctx(ctx);
        let iq = InstanceQuestion::find_one(self.id, &conn)
            .map_err::<String, _>(|_| "cannot find a question".into())?;

        iq.update_score(score, &conn)
            .map_err::<String, _>(|_| "cannot update score".into())?;
        Ok(())
    }
}

#[derive(Clone, SimpleObject, Serialize, Deserialize)]
pub(crate) struct UpdateQuestion {
    id: i32,
    state: QuestionStatus,
}

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize, Enum, Copy)]
pub(crate) enum QuestionStatus {
    Expired,
    Completed,
    Skip
}

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct ExamManager {
    pub(crate) question: Vec<Question>,
    pub(crate) score: HashMap<IQID, i32>,
}

impl ExamManager {
    pub(crate) fn new(question: &HashMap<IQID, QuestionModel>) -> Self {
        ExamManager {
            question: question
                .iter()
                .map(|(id, q)| Question::new(*id, q))
                .collect(),
            score: HashMap::new(),
        }
    }

    pub(crate) fn sync_all_score_to_db(&self, ctx: &Context<'_>) -> Result<(), String> {
        for q in &self.question {
            let score = self.score.get(&q.id).ok_or::<String>("err".into())?;
            q.sync_score_to_db(*score, ctx);
        }

        Ok(())
    }
}
