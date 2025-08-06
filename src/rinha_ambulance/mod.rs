use crate::rinha_domain::Health;
use crate::{rinha_conf, rinha_core::Result, rinha_net::resolve_socket_addr};
use derivative::Derivative;
use http::{Extensions, Method, Request, Uri, header};
use http_body_util::{BodyExt, Empty};
use hyper::body::{Buf, Bytes};
use hyper::client::conn::http1;
use hyper_util::rt::TokioIo;
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::{Arc, LazyLock};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::{OnceCell, RwLock};
use tokio::time::interval;

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

async fn try_check(upstream: &Upstream) -> Result<(&Upstream, Health)> {
    let stream = TcpStream::connect(upstream.addr).await?;

    let io = TokioIo::new(stream);
    let (mut sender, conn) = http1::handshake(io).await?;

    tokio::spawn(async move {
        if let Err(err) = conn.await {
            tracing::error!(?err);
        }
    });

    let uri = format!("http://{}/payments/service-health", upstream.addr);
    let uri = Uri::from_str(uri.as_str())?;
    let authority = uri.authority().ok_or_else(|| "Unable to get authority")?;

    let req = Request::builder()
        .method(Method::GET)
        .header(header::HOST, authority.as_str())
        .uri(uri)
        .body(Empty::<Bytes>::new())?;

    let res = sender.send_request(req).await?;
    let body = res.collect().await?.aggregate();
    let health: Health = serde_json::from_reader(body.reader())?;

    Ok((upstream, health))
}

async fn check() -> Result<()> {
    let (default_upstream, fallback_upstream) =
        get_upstreams().ok_or_else(|| "Failed to get upstreams")?;
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
    let upstreams = get_upstreams()?;
    let health_map = get_health_map();
    let health_map = health_map.read().await;

    if *health_map
        .get(&upstreams.0.hash_addr())
        .unwrap_or_else(|| &false)
    {
        return Some(upstreams.0);
    } else if *health_map
        .get(&upstreams.1.hash_addr())
        .unwrap_or_else(|| &false)
    {
        return Some(upstreams.1);
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

pub async fn bootstrap() -> Result<()> {
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

pub async fn task() -> Result<()> {
    let mut ticker = interval(Duration::from_secs(5));

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                if let Err(err) = check().await {
                    tracing::error!(?err)
                }
            }
        }
    }
}
