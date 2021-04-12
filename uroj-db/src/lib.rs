use connection::PgPool;

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

embed_migrations!();

pub fn run_migrations(pool: &PgPool) {
    let conn = pool.get().expect("Can't get DB connection");
    embedded_migrations::run(&conn).expect("Failed to run database migrations");
    // if environment variable is set (in case of production environment), then update users' hash
    // if let Ok(hash) = std::env::var("SECURED_USER_PASSWORD_HASH") {
    //     repository::update_password_hash(hash, &conn);
    // };
}

pub mod connection;
pub mod models;
pub mod schema;
