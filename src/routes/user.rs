#[get("/user")]
async fn hello(req_body: String) -> Result<UserData> {
    Ok(format!("hello:{}", req_body))
}
