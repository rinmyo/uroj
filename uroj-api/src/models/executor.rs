use async_graphql::*;
use uroj_db::models::executor::Executor as ExecutorData;

#[derive(SimpleObject)]
pub struct Executor {
    id: i32,
    addr: String,
}

impl From<&ExecutorData> for Executor {
    fn from(executor_data: &ExecutorData) -> Self {
        Executor {
            id: executor_data.id,
            addr: executor_data.addr.clone(),
        }
    }
}
