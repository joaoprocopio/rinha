use crate::{error::Result, shared::env::Env};
use axum::body::Bytes;
use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};
use tokio::{runtime::Handle, sync::mpsc};

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
    pub env: Env,
    pub handle: Handle,
    pub sender: mpsc::Sender<Bytes>,
}

impl App {
    pub async fn new(handle: Handle, sender: mpsc::Sender<Bytes>) -> Result<Self> {
        let env = Env::env_or_default()?;

        Ok(Self {
            inner: Arc::new(AppInner {
                env: env,
                handle: handle,
                sender: sender,
            }),
        })
    }
}
