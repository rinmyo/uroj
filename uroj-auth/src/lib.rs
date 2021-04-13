use std::str::FromStr;

use actix_web::web;
use async_graphql::{Context, EmptySubscription, Schema};
use models::{AppSchema, Mutation, Query};

use uroj_common::utils::{Claims, Role as AuthRole};
use uroj_db::connection::{Conn, PgPool};

pub fn create_schema_with_context(pool: PgPool) -> AppSchema {
    Schema::build(Query, Mutation, EmptySubscription)
        .enable_federation()
        .data(pool)
        .finish()
}

pub fn configure_service(cfg: &mut web::ServiceConfig) {
    cfg.service(handlers::index_playground)
        .service(handlers::index);
}

pub fn get_conn_from_ctx(ctx: &Context<'_>) -> Conn {
    ctx.data::<PgPool>()
        .expect("Can't get pool")
        .get()
        .expect("Can't get DB connection")
}

pub fn get_id_from_ctx(ctx: &Context<'_>) -> Option<String> {
    ctx.data_opt::<Claims>().map(|c| c.sub.clone())
}

pub fn get_role_from_ctx(ctx: &Context<'_>) -> Option<AuthRole> {
    ctx.data_opt::<Claims>()
        .map(|c| AuthRole::from_str(&c.role).expect("Cannot parse authrole"))
}

mod handlers;
pub mod models;
