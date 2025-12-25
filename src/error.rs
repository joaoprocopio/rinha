pub use anyhow::Error as AnyError;
pub use anyhow::anyhow as anyerror;

pub type Result<T, E = AnyError> = std::result::Result<T, E>;
