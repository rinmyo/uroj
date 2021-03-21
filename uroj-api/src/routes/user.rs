use actix_web::{Result, get, web::Json};
use serde::*;

#[derive(Debug, Serialize, Deserialize)]
struct UserResponse {

}

#[get("/user")]
async fn user() -> Result<Json<UserResponse>> {
    todo!()
}
