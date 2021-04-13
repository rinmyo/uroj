use std::str::FromStr;

use async_graphql::*;

use serde::{Deserialize, Serialize};

use chrono::NaiveDateTime;

use strum_macros::*;

use uroj_db::models::class::Class as ClassData;
use uroj_db::models::station::Station as StationData;
use uroj_db::{models::user::User as UserData};

use crate::get_conn_from_ctx;

use super::{class::Class, station::Station};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct User {
    id: String,
    email: String,
    class_id: Option<i32>,
    role: Role,
    is_active: bool,
    date_joined: NaiveDateTime,
    last_login: Option<NaiveDateTime>,
}

#[ComplexObject]
impl User {
    async fn class(&self, ctx: &Context<'_>) -> Option<Class> {
        self.class_id.map(|cid| {
            //Classはデータベースからのみ転換されることができるので、
            //もしエラーになったら、必ずプログラムのせいでユーザーに関係ないのだ
            let ref class =
                ClassData::find(cid, &get_conn_from_ctx(ctx)).expect("cannot query class");
            class.into()
        })
    }

    async fn stations(&self, ctx: &Context<'_>) -> Vec<Station> {
        StationData::find_by_author_id(&self.id, &get_conn_from_ctx(ctx))
            .expect("cannot query stations")
            .iter()
            .map(|s| s.into())
            .collect()
    }
}

impl From<&UserData> for User {
    fn from(user: &UserData) -> Self {
        User {
            id: user.id.clone(),
            role: Role::from_str(&user.user_role)
                .expect(&format!("cannot convert {} to Role", &user.user_role)),
            email: user.email.clone(),
            class_id: user.class_id,
            is_active: user.is_active,
            date_joined: user.date_joined,
            last_login: user.last_login,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Enum, Display, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum Role {
    Admin,
    User,
}
