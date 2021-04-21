use std::{net::{IpAddr, Ipv6Addr}, str::FromStr, sync::{Arc, Mutex, MutexGuard}};

use actix_web::{guard, web};
use async_graphql::{Context, EmptySubscription, Schema};
use game::instance::{Instance, InstancePool};
use models::{Mutation, Query, };

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
) -> Schema<Query, Mutation, EmptySubscription> {
    Schema::build(Query, Mutation, EmptySubscription)
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

pub fn get_instance_pool_from_ctx<'ctx>(ctx: &Context<'ctx>) -> MutexGuard<'ctx, InstancePool> {
    ctx.data_unchecked::<Arc<Mutex<InstancePool>>>()
        .lock()
        .unwrap()
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

mod game;
mod handlers;
mod models;
mod rpc;
