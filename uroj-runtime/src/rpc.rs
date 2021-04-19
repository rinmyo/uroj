use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use tarpc::context;
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
        let pool = self.instance_pool.lock().unwrap();
        if pool.contains_key(&cfg.id) {
            return Err(format!("instance {} is already running", cfg.id));
        }
        let instance = Instance::new(&cfg);
        pool.insert(cfg.id, instance);
        Ok(cfg.id)
    }
}
