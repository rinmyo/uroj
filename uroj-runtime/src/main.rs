use actix_web::{App, HttpServer};
use dotenv::dotenv;
use uroj_runtime::{create_schema_with_context, run_rpc_server};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let pool = Arc::new(Mutex::new(InstancePool::new()));
    tokio::spawn(async { run_rpc_server(2334, Arc::clone(pool)).await });

    let schema = create_schema_with_context(Arc::clone(pool));

    HttpServer::new(move || App::new().configure(configure_service).data(schema.clone()))
        .bind("0.0.0.0:8003")?
        .run()
        .await
}
