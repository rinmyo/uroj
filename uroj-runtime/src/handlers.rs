use actix_web::{get, web, HttpRequest, HttpResponse, Result};
use async_graphql::{Schema, http::{playground_source, GraphQLPlaygroundConfig}};
use async_graphql_actix_web::WSSubscription;

use crate::models::AppSchema;

pub(crate) async fn index_ws(
    schema: web::Data<AppSchema>,
    req: HttpRequest,
    payload: web::Payload,
) -> Result<HttpResponse> {
    WSSubscription::start(Schema::clone(&schema), &req, payload)
}

#[get("/")]
pub(crate) async fn index_playground() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(playground_source(
            GraphQLPlaygroundConfig::new("/").subscription_endpoint("/"),
        ))
}
