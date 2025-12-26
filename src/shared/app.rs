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
pub struct App {
    inner: Arc<AppInner>,
}

impl Deref for App {
    type Target = Arc<AppInner>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for App {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[derive(Debug)]
pub struct AppInner {
    pub env: AppEnv,
    pub handle: Handle,
    pub sender: mpsc::Sender<Bytes>,
    pub receiver: Mutex<mpsc::Receiver<Bytes>>,
}

#[derive(Debug)]
pub struct AppEnv {
    pub addr: String,
    pub uds_path: PathBuf,
}

impl App {
    pub async fn new(
        handle: Handle,
        sender: mpsc::Sender<Bytes>,
        receiver: mpsc::Receiver<Bytes>,
    ) -> Result<Self> {
        let env = AppEnv::env_or_default()?;

        Ok(Self {
            inner: Arc::new(AppInner {
                env: env,
                handle: handle,
                sender: sender,
                receiver: Mutex::new(receiver),
            }),
        })
    }
}

impl AppEnv {
    fn env_or_default() -> Result<Self> {
        Ok(Self {
            addr: env_or("RINHA_SERVER_ADDR", "0.0.0.0:8000".into()),
            uds_path: env_or("RINHA_UDS_PATH", "/tmp/rinha/rinha.sock".into()),
        })
    }
}
