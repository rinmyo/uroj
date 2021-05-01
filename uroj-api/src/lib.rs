use std::sync::Arc;

use actix_web::web;
use async_graphql::{dataloader::DataLoader, Context, EmptySubscription, Schema};
use models::{AppSchema, Mutation, Query, UserLoader};

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

use uroj_db::connection::{Conn, PgPool};

pub fn configure_service(cfg: &mut web::ServiceConfig) {
    cfg.service(handlers::index_playground)
        .service(handlers::index);
}

pub fn create_schema_with_context(pool: PgPool) -> AppSchema {
    let arc_pool = Arc::new(pool);
    let details_data_loader = DataLoader::new(UserLoader {
        pool: arc_pool.clone(),
    })
    .max_batch_size(10);

    Schema::build(Query, Mutation, EmptySubscription)
        .enable_federation()
        .data(arc_pool)
        .data(details_data_loader)
        .finish()
}

pub fn get_conn_from_ctx(ctx: &Context<'_>) -> Conn {
    ctx.data::<Arc<PgPool>>()
        .expect("Can't get pool")
        .get()
        .expect("Can't get DB connection")
}

pub fn get_random_token() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect::<String>()
        .to_uppercase()
}

mod handlers;
pub mod models;
