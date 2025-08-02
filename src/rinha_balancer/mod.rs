use crate::rinha_core::Result;
use std::time::Duration;
use tokio::time::interval;

pub async fn bootstrap() -> Result<()> {
    dbg!("discovering");

    Ok(())
}

pub async fn task() {
    let mut ttt = interval(Duration::from_secs(5));

    loop {
        tokio::select! {
            _ = ttt.tick() => {
                dbg!("abc");
            }
        }
    }
}
