use crate::rinha_core::Result;
use http::Extensions;
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::{collections::BTreeSet, net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct Upstream {
    pub addr: SocketAddr,
    pub ext: Extensions,
}

impl Upstream {
    pub fn hash_addr(&self) -> u64 {
        let mut hasher = DefaultHasher::default();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

impl Hash for Upstream {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.addr.hash(state)
    }
}

impl PartialEq for Upstream {
    fn eq(&self, other: &Self) -> bool {
        self.addr == other.addr
    }
}

impl Eq for Upstream {}

impl PartialOrd for Upstream {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.addr.cmp(&other.addr))
    }
}

impl Ord for Upstream {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.addr.cmp(&other.addr)
    }
}

impl Upstream {
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            addr: addr,
            ext: Extensions::new(),
        }
    }
}

type Upstreams = BTreeSet<Upstream>;
type HealthMap = HashMap<u64, bool>;
type HealthCheck = (dyn Fn(&Upstream) -> dyn Future<Output = crate::rinha_core::Result<()>>);

pub struct Balancer<HC: AsyncFn(&Upstream) -> Result<()>> {
    upstreams: Arc<Upstreams>,
    health: Arc<RwLock<HealthMap>>,
    health_check: HC,
}

impl Balancer {
    pub fn new(upstreams: Upstreams, health_check: HealthCheck) -> Self {
        Self {
            health: Arc::new(RwLock::new(HashMap::with_capacity(upstreams.len()))),
            upstreams: Arc::new(upstreams),
            health_check: health_check,
        }
    }

    pub async fn select(&self) -> Option<Upstream> {
        let upstreams = self.get_backends();

        for upstream in upstreams.iter() {
            if self.is_healthy(&upstream).await {
                continue;
            }

            return Some(upstream.clone());
        }

        None
    }

    pub async fn check(&self) -> () {}

    fn get_backends(&self) -> Arc<Upstreams> {
        self.upstreams.clone()
    }

    fn get_health(&self) -> Arc<RwLock<HealthMap>> {
        self.health.clone()
    }

    pub async fn is_healthy(&self, backend: &Upstream) -> bool {
        let health_map = self.get_health();
        let health_map = health_map.read().await;

        return match health_map.get(&backend.hash_addr()) {
            Some(health) => health.clone(),
            _ => false,
        };
    }

    pub async fn set_health(&mut self, upstream: &Upstream, value: bool) -> Option<bool> {
        let health_map = self.get_health();
        let mut health_map = health_map.write().await;

        health_map.insert(upstream.hash_addr(), value)
    }
}
