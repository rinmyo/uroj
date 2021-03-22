use actix_web::{HttpResponse, web};

use crate::{PgPool, db::db_pool::PgPooledConnection};

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
