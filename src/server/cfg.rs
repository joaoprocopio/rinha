use crate::{error::Result, shared::env::env_or};
use axum::body::Bytes;
use std::{
    ops::{Deref, DerefMut},
    path::PathBuf,
    sync::Arc,
};
use tokio::{
    runtime::Handle,
    sync::{Mutex, mpsc},
};

#[derive(Debug, Clone)]
pub struct Server(Arc<ServerInner>);

impl Deref for Server {
    type Target = Arc<ServerInner>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Server {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug)]
pub struct ServerInner {
    pub env: ServerEnv,
    pub handle: Handle,
    pub sender: mpsc::Sender<Bytes>,
    pub receiver: Mutex<mpsc::Receiver<Bytes>>,
}

#[derive(Debug)]
pub struct ServerEnv {
    pub addr: String,
    pub uds_path: PathBuf,
}

impl Server {
    pub async fn new(
        handle: Handle,
        sender: mpsc::Sender<Bytes>,
        receiver: mpsc::Receiver<Bytes>,
    ) -> Result<Self> {
        let env = ServerEnv::env_or_default()?;

        Ok(Self(Arc::new(ServerInner {
            env: env,
            handle: handle,
            sender: sender,
            receiver: Mutex::new(receiver),
        })))
    }
}

impl ServerEnv {
    fn env_or_default() -> Result<Self> {
        Ok(Self {
            addr: env_or("RINHA_SERVER_ADDR", "0.0.0.0:8000".into()),
            uds_path: env_or("RINHA_UDS_PATH", "/tmp/rinha/rinha.sock".into()),
        })
    }
}
