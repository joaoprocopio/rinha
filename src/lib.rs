#![warn(clippy::all)]
#![warn(clippy::perf)]
#![warn(clippy::cargo)]

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

pub mod error;
pub mod ext;
pub mod server;
pub mod shared;
pub mod worker;
