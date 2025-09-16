#![forbid(unsafe_code)]
#![deny(warnings)]
#![warn(clippy::pedantic)]

use anyhow::Result as AnyResult;
use tokio::time::{sleep, Duration};

pub async fn cpu_load(
    _experiment_id: String,
    cpu_percent: u32,
    duration_seconds: u32,
    mtr: crate::metrics::Metrics,
) -> AnyResult<()> {
    let cpu_percent = cpu_percent.max(1).min(100);
    mtr.cpu_hog_active.set(1);
    mtr.cpu_hog_duty_percent.set(cpu_percent as i64);
    // Model duty cycle per second: busy for (cpu_percent)% of 1s, sleep for the rest.
    let on = Duration::from_millis((10 * cpu_percent) as u64); // scale to 1s window: 10ms * percent = X% of 1s
    let off = Duration::from_millis((1000 - (10 * cpu_percent)) as u64);
    let end = tokio::time::Instant::now() + Duration::from_secs(u64::from(duration_seconds));
    let mut last_seconds_inc = 0u64;
    while tokio::time::Instant::now() < end {
        let spin_until = tokio::time::Instant::now() + on;
        while tokio::time::Instant::now() < spin_until {
            std::hint::spin_loop();
        }
        sleep(off).await;
        // Increase cpu_seconds_total at 1 Hz
        last_seconds_inc += 1;
        if last_seconds_inc >= 1 {
            mtr.cpu_seconds_total.inc();
            last_seconds_inc = 0;
        }
    }
    mtr.cpu_hog_active.set(0);
    Ok(())
}
