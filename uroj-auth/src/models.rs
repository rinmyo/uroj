use std::str::FromStr;

use async_graphql::validators::Email;
use async_graphql::{guard::Guard, *};

use serde::{Deserialize, Serialize};

use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use strum_macros::*;
use uroj_common::utils::{create_token, Role as AuthRole};
use uroj_db::models::user::{NewUser as NewUserData, User as UserData};

use crate::{get_conn_from_ctx, get_id_from_ctx, get_role_from_ctx};

#[derive(SimpleObject)]
pub(crate) struct User {
    id: String,
    email: String,
    role: Role,
    is_active: bool,
}

impl From<&UserData> for User {
    fn from(user: &UserData) -> Self {
        User {
            id: user.id.clone(),
            role: Role::from_str(&user.user_role)
                .expect(&format!("cannot convert {} to Role", &user.user_role)),
            email: user.email.clone(),
            is_active: user.is_active,
        }
    }
}

#[derive(InputObject)]
pub(crate) struct UserInput {
    pub(crate) id: String,
    #[graphql(validator(Email))]
    pub(crate) email: String,
    pub(crate) class_id: Option<i32>,
    pub(crate) password: String,
    pub(crate) role: Role,
}

#[derive(InputObject)]
pub(crate) struct BanUserInput {
    pub(crate) id: String,
}

#[derive(InputObject)]
pub(crate) struct SignUpInput {
    pub(crate) id: String,
    #[graphql(validator(Email))]
    pub(crate) email: String,
    pub(crate) password: String,
}

#[derive(InputObject)]
pub(crate) struct SignInInput {
    pub(crate) id: String,
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

pub type AppSchema = Schema<Query, Mutation, EmptySubscription>;
pub struct Query;

#[Object]
impl Query {
    #[graphql(guard(or(
        RoleGuard(role = "AuthRole::Admin"),
        RoleGuard(role = "AuthRole::User")
    )))]
    async fn get_users(&self, ctx: &Context<'_>) -> Result<Vec<User>> {
        Ok(UserData::list_all(&get_conn_from_ctx(ctx))?
            .iter()
            .map(|p| User::from(p))
            .collect())
    }
}

pub struct Mutation;

#[Object]
impl Mutation {
    // #[graphql(guard(RoleGuard(role = "AuthRole::Admin")))]
    async fn create_user(&self, ctx: &Context<'_>, input: UserInput) -> Result<User> {
        let new_user = NewUserData {
            id: input.id,
            hash_pwd: hash(&input.password, DEFAULT_COST)?,
            email: input.email,
            class_id: input.class_id,
            user_role: input.role.to_string(),
        };

        let created_user = &new_user.create(&get_conn_from_ctx(ctx))?;

        Ok(created_user.into())
    }

    #[graphql(guard(RoleGuard(role = "AuthRole::Admin")))]
    async fn ban_user(&self, ctx: &Context<'_>, input: BanUserInput) -> Result<bool> {
        let conn = get_conn_from_ctx(ctx);
        let user = UserData::get(&input.id, &conn)?;
        user.update_active(false, &conn)
            .map_err(|e| e.into())
            .map(|_| true)
    }

    async fn sign_up(&self, ctx: &Context<'_>, input: SignUpInput) -> Result<User> {
        let new_user = NewUserData {
            id: input.id,
            hash_pwd: hash(&input.password, DEFAULT_COST)?,
            email: input.email,
            class_id: None,
            user_role: Role::User.to_string(),
        };

        let created_user = &new_user.create(&get_conn_from_ctx(ctx))?;

        Ok(created_user.into())
    }

    async fn sign_in(&self, ctx: &Context<'_>, input: SignInInput) -> Result<String> {
        let conn = get_conn_from_ctx(ctx);
        let user = UserData::get(&input.id, &conn)?;
        if verify(&input.password, &user.hash_pwd)? {
            if !user.is_active {
                return Err("User is not available".into());
            }
            let role = AuthRole::from_str(&user.user_role).expect("Can't convert &str to AuthRole");
            user.update_last_login(Utc::now().naive_utc(), &conn)?;
            Ok(create_token(user.id, role))
        } else {
            Err("Password Error".into())
        }
    }

    async fn update_pwd(&self, ctx: &Context<'_>, input: UpdatePwdInput) -> Result<bool> {
        let id = get_id_from_ctx(ctx).ok_or("Not Login")?;
        let conn = get_conn_from_ctx(ctx);
        let user = UserData::get(&id, &conn)?;
        if verify(&input.old_pwd, &user.hash_pwd)? {
            user.update_password_hash(hash(&input.new_pwd, DEFAULT_COST)?, &conn)
                .map_err(|e| e.into())
                .map(|_| true)
        } else {
            Err("Wrong pwd".into())
        }
    }
}

pub(crate) struct RoleGuard {
    role: AuthRole,
}

#[async_trait::async_trait]
impl Guard for RoleGuard {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        match get_role_from_ctx(ctx) {
            Some(role) => {
                if role == self.role {
                    Ok(())
                } else {
                    Err("Forbiden".into())
                }
            }
            None => Err("Not Login".into()),
        }
    }
}
