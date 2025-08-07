use crate::rinha_domain::Health;
use crate::rinha_net;
use crate::{rinha_conf, rinha_net::resolve_socket_addr};
use derivative::Derivative;
use http::{Extensions, Method, Request};
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::net::SocketAddr;
use std::sync::{Arc, LazyLock};
use tokio::sync::{OnceCell, RwLock};
use tokio::time::{Duration, interval};

type HealthMap = HashMap<u64, bool>;

static DEFAULT_UPSTREAM: OnceCell<Arc<Upstream>> = OnceCell::const_new();
static FALLBACK_UPSTREAM: OnceCell<Arc<Upstream>> = OnceCell::const_new();

static HEALTH_MAP: LazyLock<Arc<RwLock<HealthMap>>> =
    LazyLock::new(|| Arc::new(RwLock::new(HashMap::new())));

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum UpstreamType {
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

#[derive(thiserror::Error, Debug)]
pub enum TryCheckError {
    #[error("hyper")]
    Hyper(#[from] hyper::Error),
    #[error("http")]
    HTTP(#[from] http::Error),
    #[error("serde")]
    Serde(#[from] serde_json::Error),
    #[error("uri")]
    URI(#[from] http::uri::InvalidUri),
    #[error("client")]
    Client(#[from] hyper_util::client::legacy::Error),

    #[error("never")]
    Infallible(#[from] std::convert::Infallible),
}

async fn try_check(upstream: &Upstream) -> Result<(&Upstream, Health), TryCheckError> {
    let client = rinha_net::get_client();
    let uri = format!("http://{}/payments/service-health", upstream.addr);
    let req = Request::builder()
        .method(Method::GET)
        .uri(uri)
        .body(Full::new(Bytes::new()))?;
    let res = client.request(req).await?;
    let body = res.into_body().collect().await?.to_bytes();
    let health: Health = serde_json::from_slice(&body)?;

    Ok((upstream, health))
}

#[derive(thiserror::Error, Debug)]
pub enum CheckError {
    #[error("upstream failed")]
    UpstreamFailed,
    #[error("trial")]
    TrialError(#[from] TryCheckError),
}

async fn check() -> Result<(), CheckError> {
    let (default_upstream, fallback_upstream) =
        get_upstreams().ok_or_else(|| CheckError::UpstreamFailed)?;
    let ((default_upstream, default_stats), (fallback_upstream, fallback_stats)) = tokio::try_join!(
        try_check(default_upstream.as_ref()),  // default
        try_check(fallback_upstream.as_ref()), // fallback
    )?;

    let health_map = get_health_map();
    let mut health_map = health_map.write().await;

    health_map.insert(default_upstream.hash_addr(), !default_stats.failing);
    health_map.insert(fallback_upstream.hash_addr(), !fallback_stats.failing);

    Ok(())
}

pub async fn select() -> Option<Arc<Upstream>> {
    let (default_upstream, fallback_upstream) = get_upstreams()?;
    let health_map = get_health_map();
    let health_map = health_map.read().await;

    if *health_map
        .get(&default_upstream.hash_addr())
        .unwrap_or_else(|| &false)
    {
        return Some(default_upstream);
    } else if *health_map
        .get(&fallback_upstream.hash_addr())
        .unwrap_or_else(|| &false)
    {
        return Some(fallback_upstream);
    }

    None
}

pub fn get_health_map() -> Arc<RwLock<HealthMap>> {
    HEALTH_MAP.clone()
}

pub fn get_upstreams() -> Option<(Arc<Upstream>, Arc<Upstream>)> {
    Some((
        DEFAULT_UPSTREAM.get()?.clone(),
        FALLBACK_UPSTREAM.get()?.clone(),
    ))
}

#[derive(thiserror::Error, Debug)]
pub enum BootstrapError {
    #[error("sockaddr")]
    SockAddr(#[from] rinha_net::ResolveSocketAddrError),
    #[error("set error")]
    SetError(#[from] tokio::sync::SetError<Arc<Upstream>>),
}

pub async fn bootstrap() -> Result<(), BootstrapError> {
    let (default_upstream, fallback_upstream) = tokio::try_join!(
        resolve_socket_addr(rinha_conf::RINHA_DEFAULT_UPSTREAM_ADDR.as_str()),
        resolve_socket_addr(rinha_conf::RINHA_FALLBACK_UPSTREAM_ADDR.as_str()),
    )?;

    let (mut default_upstream, mut fallback_upstream) = (
        Upstream::new(default_upstream),
        Upstream::new(fallback_upstream),
    );

    default_upstream.ext.insert(UpstreamType::Default);
    fallback_upstream.ext.insert(UpstreamType::Fallback);

    DEFAULT_UPSTREAM.set(Arc::new(default_upstream))?;
    FALLBACK_UPSTREAM.set(Arc::new(fallback_upstream))?;

    Ok(())
}

pub async fn task() {
    let mut ticker = interval(Duration::from_secs(5));

    loop {
        ticker.tick().await;

        if let Err(err) = check().await {
            tracing::error!(?err, "ambulance task")
        }
    }
}
