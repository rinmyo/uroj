use async_graphql::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::{collections::HashMap, str::FromStr};

use uroj_db::models::instance_question::InstanceQuestion;
use uroj_db::models::question::Question as QuestionModel;

use crate::{get_conn_from_ctx, instance::fsm::NodeID};

use super::{FrameSender, GameFrame};

type QID = i32;
#[derive(Deserialize, Serialize, Debug, Clone, SimpleObject)]
pub(crate) struct Question {
    id: QID,
    title: String,
    #[graphql(skip)]
    from: NodeID,
    #[graphql(skip)]
    to: NodeID,
    #[graphql(skip)]
    err_node: Vec<NodeID>,
    #[graphql(skip)]
    err_sgn: bool,
    score: i32,
}

impl Question {
    fn new(qid: QID, qm: &QuestionModel) -> Self {
        Question {
            id: qid,
            title: qm.title.clone(),
            from: qm.from_node as NodeID,
            to: qm.to_node as NodeID,
            err_node: qm.err_node.iter().map(|n| *n as NodeID).collect(),
            err_sgn: qm.err_sgn,
            score: qm.score,
        }
    }
}

#[derive(Clone, SimpleObject, Serialize, Deserialize)]
pub(crate) struct UpdateQuestion {
    id: i32,
    state: QuestionStatus,
}

#[derive(Deserialize, Serialize, Clone, SimpleObject)]
pub(crate) struct QuestionsData {
    questions: Vec<Question>,
}

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize, Enum, Copy)]
pub(crate) enum QuestionStatus {
    Expired,
    Completed,
    Skip,
}

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct ExamManager {
    pub(crate) question: Vec<Question>,
    pub(crate) score: HashMap<QID, i32>,
}

impl ExamManager {
    pub(crate) fn new(question: &HashMap<QID, QuestionModel>) -> Self {
        ExamManager {
            question: question
                .iter()
                .map(|(id, q)| Question::new(*id, q))
                .collect(),
            score: HashMap::new(),
        }
    }

    pub(crate) fn get_questions(&self) -> QuestionsData {
        QuestionsData {
            questions: self.question.clone(),
        }
    }

    pub(crate) fn sync_score_to_db(&self, iid: String, ctx: &Context<'_>) -> Result<(), String> {
        let conn = get_conn_from_ctx(ctx);
        let uuid = Uuid::from_str(&iid).map_err(|_|"myname")?;

        for (iqid, score) in &self.score {
            let iq = InstanceQuestion::find_one(uuid, *iqid, &conn)
                .map_err(|_| "cannot find a question")?;

            iq.update_score(*score, &conn)
                .map_err::<String, _>(|_| "cannot update score".into())?;
        }

        Ok(())
    }

    pub(crate) async fn update_state(
        &mut self,
        iqid: QID,
        state: QuestionStatus,
        sender: &FrameSender,
    ) {
        let mut score = 0;

        if state == QuestionStatus::Completed {
            for q in &self.question {
                if iqid == q.id {
                    score = q.score
                }
            }
        }

        self.score.insert(iqid, score);

        GameFrame::UpdateQuestion(UpdateQuestion {
            id: iqid,
            state: state,
        })
        .send_via(sender)
        .await;
    }
}
