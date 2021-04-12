use std::str::FromStr;

use async_graphql::validators::Email;
use async_graphql::*;

use serde::{Deserialize, Serialize};

use strum_macros::*;

use uroj_db::models::user::User as UserData;

#[derive(SimpleObject)]
pub(crate) struct User {
    id: i32,
    username: String,
    email: String,
    role: Role,
    is_active: bool,
}

impl From<&UserData> for User {
    fn from(user: &UserData) -> Self {
        User {
            username: user.username.clone(),
            role: Role::from_str(&user.user_role)
                .expect(&format!("cannot convert {} to Role", &user.user_role)),
            id: user.id,
            email: user.email.clone(),
            is_active: user.is_active,
        }
    }
}

#[derive(InputObject)]
pub(crate) struct UserInput {
    pub(crate) username: String,
    #[graphql(validator(Email))]
    pub(crate) email: String,
    pub(crate) class_id: Option<i32>,
    pub(crate) password: String,
    pub(crate) role: Role,
}

#[derive(InputObject)]
pub(crate) struct BanUserInput {
    pub(crate) id: i32,
}

#[derive(InputObject)]
pub(crate) struct SignUpInput {
    pub(crate) username: String,
    #[graphql(validator(Email))]
    pub(crate) email: String,
    pub(crate) password: String,
}

#[derive(InputObject)]
pub(crate) struct SignInInput {
    pub(crate) username: String,
    pub(crate) password: String,
}

#[derive(InputObject)]
pub(crate) struct UpdatePwdInput {
    pub(crate) old_pwd: String,
    pub(crate) new_pwd: String,
}

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Enum, Display, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum Role {
    Admin,
    User,
}
