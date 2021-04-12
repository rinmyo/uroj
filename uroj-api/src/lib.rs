use std::sync::Arc;

use actix_web::web;
use async_graphql::{EmptySubscription, Schema, dataloader::DataLoader};
use models::{AppSchema, DetailsLoader, Mutation, Query};

use uroj_db::connection::PgPool;

pub fn configure_service(cfg: &mut web::ServiceConfig) {
    cfg.service(handlers::index_playground)
        .service(handlers::index);
}

pub fn create_schema_with_context(pool: PgPool) -> Schema<Query, Mutation, EmptySubscription> {
    let arc_pool = Arc::new(pool);
    let cloned_pool = Arc::clone(&arc_pool);
    let details_data_loader = DataLoader::new(DetailsLoader {
        pool: cloned_pool
    }).max_batch_size(10);

    let kafka_consumer_counter = Mutex::new(0);

    Schema::build(Query, Mutation, Subscription)
        // limits are commented out, because otherwise introspection query won't work
        // .limit_depth(3)
        // .limit_complexity(15)
        .data(arc_pool)
        .data(details_data_loader)
        .data(kafka::create_producer())
        .data(kafka_consumer_counter)
        .finish()
}

pub fn run_migrations(pool: &PgPool) {
    let conn = pool.get().expect("Can't get DB connection");
    embedded_migrations::run(&conn).expect("Failed to run database migrations");
}

mod handlers;
pub mod models;
