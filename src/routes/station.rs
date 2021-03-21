use actix_web::web::Path;
use uroj_models::station::Station;

#[get("/station/{id}")]
async fn get_one_station(Path(id): Path<String>) -> Result<Station> {
    Ok(format!("hello:{}", req_body))
}

#[post("/station/new")]
async fn post_new_station() -> Result<StationData> {}

#[delete("/station/{id}")]
async fn delete_station() -> Result<StationData> {}

#[get("/station/all")]
async fn get_all_station() -> Result {}
