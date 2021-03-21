use crate::models::station::*;
use actix_web::{
    delete, get, post,
    web::{Json, Path},
    Result,
};

#[get("/station/{id}")]
async fn get_one_station(Path(id): Path<String>) -> Result<Json<Station>> {
    todo!()
}

#[post("/station/new")]
async fn post_new_station() -> Result<String> {
    todo!()
}

#[delete("/station/{id}")]
async fn delete_station() -> Result<String> {
    todo!()
}

#[get("/station/all")]
async fn get_all_station() -> Result<Json<Station>> {
    todo!()
}
