use actix_web::{web, App, HttpServer};

use routes::station;
use uroj_runtime::instance::pool::InstancePool;
use uuid::Uuid;

// fn init_db_pool() -> Result<InstancePool, String> {}

// fn init_instance_pool() -> Result<InstancePool, String> {
//     let pool = InstancePool::new(|| Uuid::new_v4().to_string());
//     Ok(pool)
// }

mod routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // let _instance_pool = init_instance_pool().unwrap();
    // let db_pool = init_db_pool();

    HttpServer::new(|| {
        App::new().service(
            web::scope("/api/v1")
                .service(routes::index::index)
                .service(routes::index::hello)
                .service(routes::instance::ws_index)
                .service(routes::station::get_station_by_id),
        )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
