use crate::{error::Result, shared::env::Env};
use axum::body::Bytes;
use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};
use tokio::{runtime::Handle, sync::mpsc};

pub mod http;
pub mod task;

#[derive(Debug, Clone)]
pub struct Server {
    inner: Arc<ServerInner>,
}

impl Deref for Server {
    type Target = Arc<ServerInner>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Server {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[derive(Debug)]
pub struct ServerInner {
    pub env: Env,
    pub handle: Handle,
    pub sender: mpsc::Sender<Bytes>,
}

impl Server {
    pub async fn new(handle: Handle, sender: mpsc::Sender<Bytes>) -> Result<Self> {
        let env = Env::env_or_default()?;

        Ok(Self {
            inner: Arc::new(ServerInner {
                env: env,
                handle: handle,
                sender: sender,
            }),
        })
    }
}
