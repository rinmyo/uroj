use actix_cors::Cors;
use actix_web::{App, HttpServer};
use dotenv::dotenv;
use uroj_db::{connection::create_connection_pool, run_migrations};
use uroj_runtime::{configure_service, create_schema_with_context, InstancePool};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let ins_pool = InstancePool::new();
    let db_pool = create_connection_pool();
    run_migrations(&db_pool);
    let schema = create_schema_with_context(db_pool, ins_pool);

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .configure(configure_service)
            .data(schema.clone())
    })
    .bind("0.0.0.0:8003")?
    .run()
    .await
}
