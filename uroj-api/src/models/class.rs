use async_graphql::*;
use uroj_db::{
    models::{class::Class as ClassData, user::User as UserData},
};

use crate::get_conn_from_ctx;

use super::user::User;

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct Class {
    id: i32,
    name: String,
}

#[ComplexObject]
impl Class {
    async fn users(&self, ctx: &Context<'_>) -> Vec<User> {
        //Userにはデータベースからしか転換できないので、
        //もしエラーになったら、必ずデータベースのせいでユーザーの操作に関係ないのだ
        UserData::get_by_class_id(self.id, &get_conn_from_ctx(ctx))
            .expect("cannot query users")
            .iter()
            .map(|u| u.into())
            .collect()
    }
}

impl From<&ClassData> for Class {
    fn from(class_data: &ClassData) -> Self {
        Class {
            id: class_data.id,
            name: class_data.class_name.clone(),
        }
    }
}
