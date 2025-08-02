use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::net::SocketAddr;

use crate::rinha_core::Result;

pub type BoxBody = http_body_util::combinators::BoxBody<hyper::body::Bytes, hyper::Error>;

/// Cria um socket reusável.
pub fn create_tcp_socket(addr: SocketAddr) -> Result<Socket> {
    let domain = match addr {
        SocketAddr::V4(_) => Domain::IPV4,
        SocketAddr::V6(_) => Domain::IPV6,
    };
    let addr = SockAddr::from(addr);
    let socket = Socket::new(domain, Type::STREAM, Some(Protocol::TCP))?;
    let backlog = 4096;

    socket.set_tcp_nodelay(true)?;
    socket.set_nonblocking(true)?;
    socket.bind(&addr)?;
    socket.listen(backlog)?;

    Ok(socket)
}
