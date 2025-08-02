use crate::{rinha_chan, rinha_conf, rinha_storage};

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T, E = BoxError> = std::result::Result<T, E>;

pub fn bootstrap() {
    rinha_chan::bootstrap();
    rinha_conf::bootstrap();
    rinha_storage::bootstrap();
}
