use uroj_core::instance::InstancePool;

// fn init_db_pool() -> Option<DbPool> {}

fn init_instance_pool() -> Option<InstancePool> {}

mod routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use actix_web::{App, HttpServer};

    HttpServer::new(|| {
        App::new()
            .service(routes::index::index)
            .service(routes::index::hello)
            .service(routes::instance::ws_index)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
