pub mod user;

use std::str::FromStr;

use async_graphql::{
    async_trait, guard::Guard, Context, EmptySubscription, Object, Result, Schema,
};

use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use uroj_common::utils::{create_token, Claims, Role as AuthRole};
use uroj_db::{
    get_conn_from_ctx,
    models::user::{NewUser as NewUserData, User as UserData},
};
use user::*;

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
    #[graphql(guard(RoleGuard(role = "AuthRole::Admin")))]
    async fn create_user(&self, ctx: &Context<'_>, user: UserInput) -> Result<User> {
        let new_user = NewUserData {
            username: user.username,
            hash_pwd: hash(&user.password, DEFAULT_COST)?,
            email: user.email,
            class_id: user.class_id,
            user_role: user.role.to_string(),
            is_active: true,
            date_joined: Utc::now().naive_utc(),
        };

        let created_user = &new_user.create(&get_conn_from_ctx(ctx))?;

        Ok(created_user.into())
    }

    #[graphql(guard(RoleGuard(role = "AuthRole::Admin")))]
    async fn ban_user(&self, ctx: &Context<'_>, input: BanUserInput) -> Result<bool> {
        let conn = get_conn_from_ctx(ctx);
        let user = UserData::get(input.id, &conn)?;
        user.update_active(false, &conn)
            .map_err(|e| e.into())
            .map(|_| true)
    }

    async fn sign_up(&self, ctx: &Context<'_>, user: SignUpInput) -> Result<User> {
        let new_user = NewUserData {
            username: user.username,
            hash_pwd: hash(&user.password, DEFAULT_COST)?,
            email: user.email,
            class_id: None,
            user_role: Role::User.to_string(),
            is_active: true,
            date_joined: Utc::now().naive_utc(),
        };

        let created_user = &new_user.create(&get_conn_from_ctx(ctx))?;

        Ok(created_user.into())
    }

    async fn sign_in(&self, ctx: &Context<'_>, input: SignInInput) -> Result<String> {
        let conn = get_conn_from_ctx(ctx);
        let user = UserData::get_by_username(&input.username, &conn)?;
        if verify(&input.password, &user.hash_pwd)? {
            if user.is_active {
                return Err("User is not available".into());
            }
            let role = AuthRole::from_str(&user.user_role).expect("Can't convert &str to AuthRole");
            user.update_last_login(Utc::now().naive_utc(), &conn)?;
            Ok(create_token(user.username, role))
        } else {
            Err("Password Error".into())
        }
    }

    async fn update_pwd(&self, ctx: &Context<'_>, input: UpdatePwdInput) -> Result<bool> {
        let name = get_username_from_ctx(ctx).ok_or("Not Login")?;
        let conn = get_conn_from_ctx(ctx);
        let user = UserData::get_by_username(&name, &conn)?;
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

pub fn get_username_from_ctx(ctx: &Context<'_>) -> Option<String> {
    ctx.data_opt::<Claims>().map(|c| c.sub.clone())
}

pub fn get_role_from_ctx(ctx: &Context<'_>) -> Option<AuthRole> {
    ctx.data_opt::<Claims>()
        .map(|c| AuthRole::from_str(&c.role).expect("Cannot parse authrole"))
}
