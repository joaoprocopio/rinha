use crate::rinha_balancer::upstream::{Balancer, Upstream};
use crate::{rinha_conf, rinha_core::Result, rinha_net::resolve_socket_addr};
use std::hash::Hash;
use std::sync::Arc;
use std::{collections::BTreeSet, time::Duration};
use tokio::sync::OnceCell;
use tokio::time::interval;

mod upstream;

static BALANCER: OnceCell<Arc<Balancer>> = OnceCell::const_new();

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Processor {
    Default,
    Fallback,
}

async fn health_check(upstream: &Upstream) -> Result<()> {
    Ok(())
}

pub async fn bootstrap() -> Result<()> {
    let (default_upstream, fallback_upstream) = tokio::try_join!(
        resolve_socket_addr(rinha_conf::RINHA_DEFAULT_UPSTREAM_ADDR.as_str()),
        resolve_socket_addr(rinha_conf::RINHA_FALLBACK_UPSTREAM_ADDR.as_str()),
    )?;

    let (mut default_upstream, mut fallback_upstream) = (
        Upstream::new(default_upstream),
        Upstream::new(fallback_upstream),
    );
    default_upstream.ext.insert(Processor::Default);
    fallback_upstream.ext.insert(Processor::Fallback);

    let balancer = Arc::new(Balancer::new(
        BTreeSet::from([default_upstream, fallback_upstream]),
        Box::new(|upstream: &Upstream| Box::pin(health_check(upstream))),
    ));

    BALANCER.set(balancer)?;

    Ok(())
}

pub fn get_balancer() -> Result<Arc<Balancer>> {
    let balancer = BALANCER.get().ok_or("Failed")?;

    Ok(balancer.clone())
}

pub async fn task() -> Result<()> {
    let mut inter = interval(Duration::from_secs(5));
    let balancer = get_balancer()?;

    loop {
        tokio::select! {
            _ = inter.tick() => {
                if let Err(err) = balancer.check().await {
                    tracing::error!(?err)
                }
            }
        }
    }
}
