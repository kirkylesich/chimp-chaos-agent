#![forbid(unsafe_code)]
#![deny(warnings)]
#![warn(clippy::pedantic)]

use anyhow::{bail, Result as AnyResult};
use crate::domain::StartRequest;

pub fn validate_start(req: &StartRequest) -> AnyResult<()> {
    if req.experiment_id.trim().is_empty() { bail!("experiment_id is empty"); }
    if req.duration_seconds == 0 { bail!("duration_seconds must be > 0"); }
    match req.kind.as_str() {
        "CPU" => {
            let p = req.cpu_percent.unwrap_or(50);
            if p == 0 || p > 100 { bail!("cpu_percent must be 1..=100"); }
        }
        "MEMORY" => {
            let _m = req.memory_mb.unwrap_or(50);
        }
        other => bail!(format!("unsupported kind: {other}")),
    }
    Ok(())
}

