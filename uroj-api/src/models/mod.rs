pub struct QueryRoot;

use std::{collections::HashMap, str::FromStr, sync::Arc};

use async_graphql::{async_trait, dataloader::Loader, guard::Guard, Context, Object, Result};
use async_graphql::{EmptySubscription, Error, Schema};
use uroj_common::utils::{Claims, Role as AuthRole};
use uroj_db::connection::PgPool;
use uroj_db::models::class::Class as ClassData;
use uroj_db::models::executor::{Executor as ExecutorData, NewExecutor as NewExecutorData};
use uroj_db::models::instance::{Instance as InstanceData, NewInstance as NewInstanceData};
use uroj_db::models::station::{NewStation as NewStationData, Station as StationData};
use uroj_db::models::user::User as UserData;

use class::Class;
use user::User;

use station::Station;
use uuid::Uuid;

use crate::{get_conn_from_ctx, get_random_token};

use self::{
    executor::Executor,
    instance::{Instance, InstanceInput, InstanceStatus},
    station::StationInput,
};

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
    async fn executors(&self, ctx: &Context<'_>) -> Result<Vec<Executor>> {
        Ok(ExecutorData::list_all(&get_conn_from_ctx(ctx))?
            .iter()
            .map(|c| c.into())
            .collect())
    }

    #[graphql(guard(LoginGaurd()))]
    async fn user(&self, ctx: &Context<'_>, id: String) -> Result<User> {
        let ref user = UserData::get(&id, &get_conn_from_ctx(ctx))?;
        Ok(user.into())
    }

    #[graphql(guard(LoginGaurd()))]
    async fn me(&self, ctx: &Context<'_>) -> Result<User> {
        let uid = get_id_from_ctx(ctx).unwrap();
        let ref user = UserData::get(&uid, &get_conn_from_ctx(ctx))?;
        Ok(user.into())
    }

    // #[graphql(guard(LoginGaurd()))]
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

    #[graphql(guard(LoginGaurd()))]
    async fn stations(&self, ctx: &Context<'_>) -> Result<Vec<Station>> {
        Ok(StationData::list_all(&get_conn_from_ctx(ctx))?
            .iter()
            .map(|p| p.into())
            .collect())
    }

    #[graphql(guard(LoginGaurd()))]
    async fn instance(&self, ctx: &Context<'_>, uuid: String) -> Result<Instance> {
        let uuid = Uuid::from_str(&uuid)?;
        let ref instance = InstanceData::find_one(uuid, &get_conn_from_ctx(ctx))?;
        Ok(instance.into())
    }

    #[graphql(guard(LoginGaurd()))]
    async fn instances(&self, ctx: &Context<'_>) -> Result<Vec<Instance>> {
        Ok(InstanceData::list_all(&get_conn_from_ctx(ctx))?
            .iter()
            .map(|p| p.into())
            .collect())
    }
}

pub struct Mutation;

#[Object]
impl Mutation {
    #[graphql(guard(RoleGuard(role = "AuthRole::Admin")))]
    async fn create_station(&self, ctx: &Context<'_>, input: StationInput) -> Result<Station> {
        let id = get_id_from_ctx(ctx).ok_or("Invalid token")?;
        let conn = get_conn_from_ctx(ctx);
        let user = UserData::get(&id, &conn)?;
        let new_station = NewStationData {
            title: input.title,
            description: input.description,
            draft: input.draft,
            author_id: Some(user.id),
            yaml: input.yaml,
        };

        let created_station = &new_station.create(&conn)?;
        Ok(created_station.into())
    }

    #[graphql(guard(RoleGuard(role = "AuthRole::Admin")))]
    async fn create_executor(&self, ctx: &Context<'_>, url: String) -> Result<Executor> {
        let conn = get_conn_from_ctx(ctx);
        let new_executor = NewExecutorData { addr: url };
        let created_executor = &new_executor.create(&conn)?;
        Ok(created_executor.into())
    }

    async fn create_instance(&self, ctx: &Context<'_>, input: InstanceInput) -> Result<Instance> {
        let id = get_id_from_ctx(ctx).ok_or("Invalid token")?;
        let conn = get_conn_from_ctx(ctx);
        let user = UserData::get(&id, &conn)?;
        let station = StationData::find(input.station_id, &conn)?;

        let new_instance = NewInstanceData {
            title: input.title,
            description: input.description,
            creator_id: user.id,
            player_id: input.player,
            station_id: station.id,
            curr_state: InstanceStatus::default().to_string(),
            executor_id: input.executor_id,
            token: get_random_token(),
        };

        let created_instance = &new_instance.create(&conn)?;
        Ok(created_instance.into())
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
        get_id_from_ctx(ctx).ok_or("Not Login".into()).map(|_| ())
    }
}

fn get_id_from_ctx(ctx: &Context<'_>) -> Option<String> {
    ctx.data_opt::<Claims>().map(|c| c.sub.clone())
}

fn get_role_from_ctx(ctx: &Context<'_>) -> Option<AuthRole> {
    ctx.data_opt::<Claims>()
        .map(|c| AuthRole::from_str(&c.role).expect("Cannot parse authrole"))
}

pub struct UserLoader {
    pub pool: Arc<PgPool>,
}

#[async_trait::async_trait]
impl Loader<String> for UserLoader {
    type Value = User;
    type Error = Error;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        let conn = self.pool.get().expect("Can't get DB connection");
        let users = UserData::find_many(keys, &conn).expect("Can't get users' details");
        Ok(users.iter().map(|u| (u.id.clone(), u.into())).collect())
    }
}

pub mod class;
pub mod executor;
pub mod instance;
pub mod station;
pub mod user;
