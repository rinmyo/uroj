use actix_web::{web, App, HttpServer};
use async_graphql::{EmptyMutation, EmptySubscription, Schema};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let pool = create_connection_pool();
    run_migrations(&pool);

    let schema = create_schema_with_context(pool);

    HttpServer::new(move || App::new().configure(configure_service).data(schema.clone()))
        .bind("0.0.0.0:8001")?
        .run()
        .await
}
