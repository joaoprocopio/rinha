#[global_allocator]
static ALLOCATOR: jemallocator::Jemalloc = jemallocator::Jemalloc;

use pingora::prelude::*;
use tokio::sync::broadcast;

mod rinha_domain;
mod rinha_http;
mod rinha_worker;

fn main() {
    let mut server = Server::new(None).unwrap();
    server.bootstrap();

    // NOTA: a solução com broadcast channels funciona por agora, mais se colocar mais de um worker, vai fuder o esquema.
    // todos mundo que tiver ouvindo nesse receiver vai receber a struct,
    // isso vai fazer a mesma struct ser processada N — sendo N o número de workers
    let (sender, receiver) = broadcast::channel::<Payment>(size_of::<Payment>() * 100);

    let mut rinha = rinha_service(sender);
    rinha.add_tcp("0.0.0.0:9999");

    let rinha_worker = rinha_worker_service(receiver);

    server.add_service(rinha);
    server.add_service(rinha_worker);

    server.run_forever();
}
