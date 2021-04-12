pub struct QueryRoot;

use std::str::FromStr;

use async_graphql::{async_trait, guard::Guard, Context, Object, Result};
use async_graphql::{EmptySubscription, Schema};
use chrono::Utc;
use uroj_common::utils::{Claims, Role as AuthRole};
use uroj_db::get_conn_from_ctx;
use uroj_db::models::class::Class as ClassData;
use uroj_db::models::station::{NewStation as NewStationData, Station as StationData};
use uroj_db::models::user::User as UserData;

use class::Class;
use user::User;

use station::Station;

use self::station::StationInput;

pub type AppSchema = Schema<Query, Mutation, EmptySubscription>;

pub struct Query;

#[Object]
impl Query {
    #[graphql(guard(LoginGaurd()))]
    async fn class(&self, ctx: &Context<'_>, id: i32) -> Result<Class> {
        let ref class = ClassData::find(id, &get_conn_from_ctx(ctx))?;
        Ok(class.into())
    }

    #[graphql(guard(RoleGuard(role = "AuthRole::Admin")))]
    async fn classes(&self, ctx: &Context<'_>) -> Result<Vec<Class>> {
        Ok(ClassData::list_all(&get_conn_from_ctx(ctx))?
            .iter()
            .map(|c| c.into())
            .collect())
    }

    #[graphql(guard(LoginGaurd()))]
    async fn user(&self, ctx: &Context<'_>, id: i32) -> Result<User> {
        let ref user = UserData::get(id, &get_conn_from_ctx(ctx))?;
        Ok(user.into())
    }

    #[graphql(guard(LoginGaurd()))]
    async fn user_by_username(&self, ctx: &Context<'_>, username: String) -> Result<User> {
        let ref user = UserData::get_by_username(&username, &get_conn_from_ctx(ctx))?;
        Ok(user.into())
    }

    #[graphql(guard(LoginGaurd()))]
    async fn users(&self, ctx: &Context<'_>) -> Result<Vec<User>> {
        Ok(UserData::list_all(&get_conn_from_ctx(ctx))?
            .iter()
            .map(|p| p.into())
            .collect())
    }

    #[graphql(guard(LoginGaurd()))]
    async fn station(&self, ctx: &Context<'_>, id: i32) -> Result<Station> {
        let ref station = StationData::find(id, &get_conn_from_ctx(ctx))?;
        Ok(station.into())
    }
}

pub struct Mutation;

#[Object]
impl Mutation {
    #[graphql(guard(RoleGuard(role = "AuthRole::Admin")))]
    async fn create_station(&self, ctx: &Context<'_>, input: StationInput) -> Result<Station> {
        let name = get_username_from_ctx(ctx).ok_or("Invalid token")?;
        let conn = get_conn_from_ctx(ctx);
        let user = UserData::get_by_username(&name, &conn)?;
        let new_station = NewStationData {
            title: input.title,
            description: input.description,
            created: Utc::now().naive_utc(),
            updated: Utc::now().naive_utc(),
            draft: input.draft,
            author_id: Some(user.id),
            yaml: input.yaml,
        };

        let created_station = &new_station.create(&conn)?;
        Ok(created_station.into())
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

pub(crate) struct LoginGaurd;

#[async_trait::async_trait]
impl Guard for LoginGaurd {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        get_username_from_ctx(ctx)
            .ok_or("Not Login".into())
            .map(|_| ())
    }
}

fn get_username_from_ctx(ctx: &Context<'_>) -> Option<String> {
    ctx.data_opt::<Claims>().map(|c| c.sub.clone())
}

fn get_role_from_ctx(ctx: &Context<'_>) -> Option<AuthRole> {
    ctx.data_opt::<Claims>()
        .map(|c| AuthRole::from_str(&c.role).expect("Cannot parse authrole"))
}

pub mod class;
pub mod station;
pub mod user;
