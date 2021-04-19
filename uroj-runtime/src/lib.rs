use std::{
    net::{IpAddr, Ipv6Addr},
    str::FromStr,
    sync::{Arc, Mutex},
};

use actix_web::{guard, web};
use async_graphql::{Context, Schema};
use futures::{future, prelude::*};
use game::instance::{Instance, InstancePool};
use models::{Mutation, Query, Subscription};
use tarpc::server::{self, Channel, Incoming};
use tokio_serde::formats::Json;
use uroj_common::utils::{Claims, Role as AuthRole};

pub fn configure_service(cfg: &mut web::ServiceConfig) {
    cfg.service(handlers::index_playground).service(
        web::resource("/").route(
            web::get()
                .guard(guard::Header("upgrade", "websocket"))
                .to(handlers::index_ws),
        ),
    );
}

pub fn create_schema_with_context(
    arc_pool: Arc<Mutex<InstancePool>>,
) -> Schema<Query, Mutation, Subscription> {
    Schema::build(Query, Mutation, Subscription)
        // limits are commented out, because otherwise introspection query won't work
        // .limit_depth(3)
        // .limit_complexity(15)
        .data(arc_pool)
        .finish()
}

pub fn get_id_from_ctx(ctx: &Context<'_>) -> Result<String, String> {
    ctx.data_opt::<Claims>()
        .map(|c| c.sub.clone())
        .ok_or("not found login user".to_string())
}

pub fn get_role_from_ctx(ctx: &Context<'_>) -> Option<AuthRole> {
    ctx.data_opt::<Claims>()
        .map(|c| AuthRole::from_str(&c.role).expect("Cannot parse authrole"))
}

pub fn borrow_instance_from_ctx<'ctx>(
    ctx: &Context<'ctx>,
    id: &str,
) -> Result<&'ctx Instance, String> {
    ctx.data_unchecked::<Arc<Mutex<InstancePool>>>()
        .lock()
        .unwrap()
        .get(id)
        .ok_or("err".to_string())
}

async fn run_rpc_server(port: u16, arc_pool: Arc<Mutex<InstancePool>>) {
    let server_addr = (IpAddr::V6(Ipv6Addr::LOCALHOST), port);

    let mut listener = tarpc::serde_transport::tcp::listen(&server_addr, Json::default)
        .await
        .unwrap();
    listener.config_mut().max_frame_length(usize::MAX);
    listener
        .filter_map(|r| future::ready(r.ok()))
        .map(server::BaseChannel::with_defaults)
        // Limit channels to 1 per IP.
        .max_channels_per_key(1, |t| t.transport().peer_addr().unwrap().ip())
        .map(|channel| {
            let server = rpc::RunnerServer {
                instance_pool: arc_pool,
            };
            channel.execute(server.serve())
        })
        // Max 10 channels.
        .buffer_unordered(10)
        .for_each(|_| async {})
        .await;
}

mod game;
mod handlers;
mod models;
mod rpc;
