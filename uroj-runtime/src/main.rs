#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let schema = create_schema_with_context(pool);

    HttpServer::new(move || App::new().configure(configure_service).data(schema.clone()))
        .bind("0.0.0.0:8003")?
        .run()
        .await
}
