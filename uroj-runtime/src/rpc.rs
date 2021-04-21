use std::{collections::HashMap, net::{IpAddr, Ipv6Addr, SocketAddr}, sync::{Arc, Mutex}};
use futures::{future, prelude::*};

use tarpc::context;
use tarpc::server::{self, Channel, Incoming};
use tokio_serde::formats::Json;
use uroj_common::rpc::{InstanceConfig, Runner};

use crate::game::instance::{Instance, InstancePool};

#[derive(Clone)]
pub(crate) struct RunnerServer {
    instance_pool: Arc<Mutex<InstancePool>>,
}

#[tarpc::server]
impl Runner for RunnerServer {
    async fn run_instance(
        self,
        _: context::Context,
        cfg: InstanceConfig,
    ) -> Result<String, String> {
        let id = cfg.id.clone();
        let mut pool = self.instance_pool.lock().unwrap();
        if pool.contains_key(&id) {
            return Err(format!("instance {} is already running", id));
        }
        let instance = Instance::new(&cfg)?;
        pool.insert(id.clone(), instance);
        Ok(id)
    }
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
            let server = RunnerServer {
                instance_pool: arc_pool.clone(),
            };
            channel.execute(server.serve())
        })
        // Max 10 channels.
        .buffer_unordered(10)
        .for_each(|_| async {})
        .await;
}
