use crate::{error::Result, shared::env::env_or};
use axum::body::Bytes;
use std::{ops::Deref, sync::Arc};
use tokio::{runtime::Handle, sync::mpsc::Sender};

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
    pub sender: Sender<Bytes>,
    pub env: ServerEnv,
}

#[derive(Debug)]
pub struct ServerEnv {
    pub addr: String,
}

impl Server {
    pub async fn new(handle: Handle, sender: Sender<Bytes>) -> Result<Self> {
        let env = ServerEnv::env_or_default()?;

        Ok(Self(Arc::new(ServerInner {
            handle: handle,
            sender: sender,
            env: env,
        })))
    }
}

impl ServerEnv {
    fn env_or_default() -> Result<Self> {
        Ok(Self {
            addr: env_or("RINHA_ADDR", "0.0.0.0:8000".into()),
        })
    }
}
