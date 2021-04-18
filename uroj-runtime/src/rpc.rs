use std::{collections::HashMap, net::SocketAddr};

use tarpc::{context};
use uroj_common::rpc::{NewInstanceConfig, World};

#[derive(Clone)]
struct RpcServer {
    redis: redis::Client
}

#[tarpc::server]
impl World for RpcServer {
    async fn create_instance(self, _: context::Context, req: NewInstanceConfig) -> Result<String, String> {
        let con = self.redis.get_tokio_connection_tokio().await?;
        con.set()
        todo!()
    }

    async fn delete_instance(self, _: context::Context, req: NewInstanceConfig) -> String {
        todo!()
    }
}