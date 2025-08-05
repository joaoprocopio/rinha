use crate::{rinha_conf, rinha_core::Result, rinha_net::resolve_socket_addr};
use derivative::Derivative;
use http::Extensions;
use std::collections::{BTreeSet, HashMap};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::net::SocketAddr;
use std::sync::{Arc, LazyLock};
use std::time::Duration;
use tokio::sync::{OnceCell, RwLock};
use tokio::time::interval;

type Upstreams = BTreeSet<Upstream>;
type HealthMap = HashMap<u64, bool>;

static UPSTREAMS: OnceCell<Arc<Upstreams>> = OnceCell::const_new();
static HEALTH_MAP: LazyLock<Arc<RwLock<HealthMap>>> =
    LazyLock::new(|| Arc::new(RwLock::new(HashMap::new())));

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Processor {
    Default,
    Fallback,
}

#[derive(Derivative)]
#[derivative(Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Upstream {
    pub addr: SocketAddr,

    #[derivative(PartialEq = "ignore")]
    #[derivative(PartialOrd = "ignore")]
    #[derivative(Hash = "ignore")]
    #[derivative(Ord = "ignore")]
    pub ext: Extensions,
}

impl Upstream {
    pub fn hash_addr(&self) -> u64 {
        let mut hasher = DefaultHasher::default();
        self.hash(&mut hasher);
        hasher.finish()
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

async fn check() -> Result<()> {
    Ok(())
}

pub async fn select() -> Option<Upstream> {
    let upstreams = get_upstreams()?;
    let health_map = get_health_map();
    let health_map = health_map.read().await;

    for upstream in upstreams.iter() {
        let is_healthy = match health_map.get(&upstream.hash_addr()) {
            Some(is_healthy) => is_healthy.clone(),
            _ => continue,
        };

        if is_healthy {
            return Some(upstream.clone());
        }

        return None;
    }

    None
}

pub fn get_health_map() -> Arc<RwLock<HealthMap>> {
    HEALTH_MAP.clone()
}

pub fn get_upstreams() -> Option<Arc<Upstreams>> {
    Some(UPSTREAMS.get()?.clone())
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

    UPSTREAMS.set(Arc::new(BTreeSet::from([
        default_upstream,
        fallback_upstream,
    ])))?;

    Ok(())
}

pub async fn task() -> Result<()> {
    let mut inter = interval(Duration::from_secs(5));

    loop {
        tokio::select! {
            _ = inter.tick() => {
                if let Err(err) = check().await {
                    tracing::error!(?err)
                }
            }
        }
    }
}
