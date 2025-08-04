use crate::rinha_balancer::upstream::{Balancer, Upstream};
use crate::{rinha_conf, rinha_core::Result, rinha_net::resolve_socket_addr};
use std::hash::Hash;
use std::{collections::BTreeSet, time::Duration};
use tokio::time::interval;

pub mod upstream;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Processor {
    Default,
    Fallback,
}

async fn health_check(upstream: &Upstream) -> Result<()> {
    Ok(())
}

pub async fn bootstrap() -> Result<Balancer> {
    let (default_upstream, fallback_upstream) = tokio::try_join!(
        resolve_socket_addr(rinha_conf::RINHA_DEFAULT_UPSTREAM_ADDR.as_str()),
        resolve_socket_addr(rinha_conf::RINHA_FALLBACK_UPSTREAM_ADDR.as_str()),
    )?;

    let mut default_upstream = Upstream::new(default_upstream);
    let mut fallback_upstream = Upstream::new(fallback_upstream);
    default_upstream.ext.insert(Processor::Default);
    fallback_upstream.ext.insert(Processor::Fallback);

    dbg!(default_upstream.hash_addr());

    let balancer = Balancer::new(
        BTreeSet::from([default_upstream, fallback_upstream]),
        health_check,
    );

    Ok(balancer)
}

pub async fn task() {
    let mut ttt = interval(Duration::from_secs(5));

    loop {
        tokio::select! {
            _ = ttt.tick() => {
                dbg!("abc");
            }
        }
    }
}
