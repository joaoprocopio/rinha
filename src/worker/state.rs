use crate::{error::Result, shared::env::Env};
use std::{ops::Deref, sync::Arc};
use tokio::runtime::Handle;

#[derive(Debug, Clone)]
pub struct Worker {
    inner: Arc<WorkerInner>,
}

impl Deref for Worker {
    type Target = Arc<WorkerInner>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Debug)]
pub struct WorkerInner {
    pub env: Env,
    pub handle: Handle,
}

impl Worker {
    pub async fn new(handle: Handle) -> Result<Self> {
        let env = Env::env_or_default()?;

        Ok(Self {
            inner: Arc::new(WorkerInner {
                env: env,
                handle: handle,
            }),
        })
    }
}
