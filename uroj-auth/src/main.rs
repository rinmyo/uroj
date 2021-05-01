use actix_web::{App, HttpServer};
use dotenv::dotenv;
use uroj_auth::{configure_service, create_schema_with_context};
use uroj_db::connection::create_connection_pool;
use uroj_db::run_migrations;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let pool = create_connection_pool();
    run_migrations(&pool);

    let schema = create_schema_with_context(pool);

    HttpServer::new(move || App::new().configure(configure_service).data(schema.clone()))
        .bind("0.0.0.0:8002")?
        .run()
        .await
}
