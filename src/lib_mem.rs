#![forbid(unsafe_code)]
#![deny(warnings)]
#![warn(clippy::pedantic)]

use anyhow::Result as AnyResult;
use tokio::time::{sleep, Duration};

pub async fn memory_load(_experiment_id: String, memory_mb: u32, duration_seconds: u32) -> AnyResult<()> {
    let bytes = (memory_mb as usize).saturating_mul(1024 * 1024);
    let mut buf = Vec::<u8>::new();
    if bytes > 0 { buf.resize(bytes, 0u8); }
    let end = tokio::time::Instant::now() + Duration::from_secs(u64::from(duration_seconds));
    while tokio::time::Instant::now() < end {
        if !buf.is_empty() { buf[0] = buf[0].wrapping_add(1); }
        sleep(Duration::from_millis(50)).await;
    }
    Ok(())
}


