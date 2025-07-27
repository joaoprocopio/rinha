use pingora::{
    http::ResponseHeader,
    lb::{Backend, health_check::HttpHealthCheck},
};

struct ValidationError;
fn validator(header: &ResponseHeader) -> Result<(), Box<pingora::Error>> {
    Ok(())
}

pub fn rinha_ambulance_service() {
    let mut hc = HttpHealthCheck::new("1.1.1.1", false);

    let v = Box::new(validator);

    hc.validator = Some(Box::new(&validator));

    let default_backend = Backend::new_with_weight("http://0.0.0.0:8001", 10).unwrap();
    let fallback_backend = Backend::new_with_weight("http://0.0.0.0:8002", 1).unwrap();
}
