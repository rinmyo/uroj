use actix_web::{HttpRequest, HttpResponse, Result, get, post, web};
use async_graphql::{Schema, http::{playground_source, GraphQLPlaygroundConfig}};
use async_graphql_actix_web::{Request, Response, WSSubscription};
use uroj_common::utils::get_claims;

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

#[post("/")]
pub(crate) async fn index(schema: web::Data<AppSchema>, http_req: HttpRequest, req: Request) -> Response {
    let mut query = req.into_inner();

    let maybe_claims = get_claims(http_req);
    if let Some(claims) = maybe_claims {
        query = query.data(claims);
    }

    schema.execute(query).await.into()
}
