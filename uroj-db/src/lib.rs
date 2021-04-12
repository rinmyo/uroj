use std::sync::Arc;

use async_graphql::Context;
use connection::{Conn, PgPool};

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
extern crate dotenv;

embed_migrations!();

pub fn run_migrations(pool: &PgPool) {
    let conn = pool.get().expect("Can't get DB connection");
    embedded_migrations::run(&conn).expect("Failed to run database migrations");
    // if environment variable is set (in case of production environment), then update users' hash
    // if let Ok(hash) = std::env::var("SECURED_USER_PASSWORD_HASH") {
    //     repository::update_password_hash(hash, &conn);
    // };
}

pub fn get_conn_from_ctx(ctx: &Context<'_>) -> Conn {
    ctx.data::<Arc<PgPool>>().expect("Can't get pool").get().expect("Can't get DB connection")
}

pub mod connection;
pub mod models;
pub mod schema;
