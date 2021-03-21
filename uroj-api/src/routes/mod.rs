use actix_web::web;

use crate::PgPool;

pub mod index;
pub mod app;
pub mod user;
pub mod station;

fn pg_pool_handler(pool: web::Data<PgPool>) -> Result<PgPooledConnection, HttpResponse> {
    pool
    .get()
    .map_err(|e| {
        HttpResponse::InternalServerError().json(e.to_string())
    })
}
