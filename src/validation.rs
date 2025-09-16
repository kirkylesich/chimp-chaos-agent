#![forbid(unsafe_code)]
#![deny(warnings)]
#![warn(clippy::pedantic)]

use anyhow::{bail, Result as AnyResult};
use crate::domain::{StartRequest, ExperimentKind, StartParams};
use std::str::FromStr;

pub fn validate_start(req: &StartRequest) -> AnyResult<()> {
    if req.experiment_id.trim().is_empty() { bail!("experiment_id is empty"); }
    if req.duration_seconds == 0 { bail!("duration_seconds must be > 0"); }
    let kind = ExperimentKind::from_str(&req.kind)?;
    match (kind, &req.params) {
        (ExperimentKind::CPU, StartParams::Cpu { duty_percent }) => {
            if *duty_percent == 0 || *duty_percent > 100 { bail!("duty_percent must be 1..=100"); }
        }
        (ExperimentKind::MEMORY, StartParams::Memory { memory_mb: _ }) => {}
        _ => bail!("kind and params mismatch"),
    }
    Ok(())
}

