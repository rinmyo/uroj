use actix_web::web;
use async_graphql::{Context, EmptySubscription, Schema};
use models::{AppSchema, Mutation, Query};
use uroj_common::utils::Claims;
use uroj_db::connection::PgPool;

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

mod handlers;
pub mod models;
