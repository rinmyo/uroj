use actix_web::{get, web::Query, HttpRequest, Result};
use serde::Deserialize;

#[derive(Deserialize)]
struct Info {
    usr: String,
}

#[get("/index")]
async fn index(req: HttpRequest, form: Query<Info>) -> Result<String> {
    println!("{:?}", req);
    Ok(format!("welcome: {}", form.usr))
}

#[get("/hello")]
async fn hello(req_body: String) -> Result<String> {
    Ok(format!("hello:{}", req_body))
}
