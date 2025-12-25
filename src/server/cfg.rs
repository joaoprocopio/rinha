use crate::{error::Result, shared::env::env_or};
use std::{ops::Deref, sync::Arc};
use tokio::runtime::Handle;

#[derive(Debug, Clone)]
pub struct Server(Arc<ServerInner>);

impl Deref for Server {
    type Target = Arc<ServerInner>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct ServerInner {
    pub handle: Handle,
    pub env: ServerEnv,
}

#[derive(Debug)]
pub struct ServerEnv {
    pub addr: String,
}

impl Server {
    pub async fn new(handle: Handle) -> Result<Self> {
        let env = ServerEnv::from_env_or_default()?;

        Ok(Self(Arc::new(ServerInner {
            handle: handle,
            env: env,
        })))
    }
}

impl ServerEnv {
    fn from_env_or_default() -> Result<Self> {
        Ok(Self {
            addr: env_or("RINHA_ADDR", "0.0.0.0:8000".into()),
        })
    }
}
