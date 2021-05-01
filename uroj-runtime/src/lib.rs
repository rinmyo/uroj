use actix_web::{guard, web};
use async_graphql::{Context, Schema};
use models::{AppSchema, Mutation, Query, Subscription};
use std::{str::FromStr};
use tokio::sync::{Mutex as TokioMutex, MutexGuard as TokioMutexGuard};

use uroj_common::utils::{Claims, Role as AuthRole};
use uroj_db::connection::{Conn, PgPool};

pub fn configure_service(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/")
            .route(web::post().to(handlers::index))
            .route(
                web::get()
                    .guard(guard::Header("upgrade", "websocket"))
                    .to(handlers::index_ws),
            )
            .route(web::get().to(handlers::index_playground)),
    );
}

pub fn create_schema_with_context(db_pool: PgPool, ins_pool: InstancePool) -> AppSchema {
    let mux_ins_pool = TokioMutex::new(ins_pool);

    Schema::build(Query, Mutation, Subscription)
        // limits are commented out, because otherwise introspection query won't work
        // .limit_depth(3)
        // .limit_complexity(15)
        .data(db_pool)
        .data(mux_ins_pool)
        .finish()
}

pub(crate) fn get_conn_from_ctx(ctx: &Context<'_>) -> Conn {
    ctx.data::<PgPool>()
        .expect("Can't get pool")
        .get()
        .expect("Can't get DB connection")
}

pub(crate) fn get_id_from_ctx(ctx: &Context<'_>) -> Result<String, String> {
    ctx.data_opt::<Claims>()
        .map(|c| c.sub.clone())
        .ok_or("not found login user".to_string())
}

pub(crate) fn get_role_from_ctx(ctx: &Context<'_>) -> Option<AuthRole> {
    ctx.data_opt::<Claims>()
        .map(|c| AuthRole::from_str(&c.role).expect("Cannot parse authrole"))
}

pub(crate) async fn get_instance_pool_from_ctx<'ctx>(
    ctx: &Context<'ctx>,
) -> TokioMutexGuard<'ctx, InstancePool> {
    ctx.data_unchecked::<TokioMutex<InstancePool>>()
        .lock()
        .await
}


mod handlers;
mod instance;
mod models;
mod raw_station;

pub type InstancePool = instance::InstancePool;
