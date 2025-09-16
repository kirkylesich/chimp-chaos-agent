#![forbid(unsafe_code)]
#![deny(warnings)]
#![warn(clippy::pedantic)]

use anyhow::{bail, Result as AnyResult};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ExperimentState {
    pub running: bool,
    pub kind: String,
    pub total_duration_seconds: u32,
    pub remaining_seconds: u32,
    pub started_ts_seconds: i64,
    pub ends_ts_seconds: i64,
}

#[derive(Clone, Default)]
pub struct LoadController {
    pub state: Arc<Mutex<HashMap<String, ExperimentState>>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StartRequest {
    pub experiment_id: String,
    pub kind: String,
    pub duration_seconds: u32,
    pub params: StartParams,
}

#[derive(Clone)]
pub struct AppState {
    pub ctrl: LoadController,
    pub metrics: crate::metrics::Metrics,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExperimentKind {
    CPU,
    MEMORY,
}

impl std::fmt::Display for ExperimentKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExperimentKind::CPU => f.write_str("CPU"),
            ExperimentKind::MEMORY => f.write_str("MEMORY"),
        }
    }
}

impl FromStr for ExperimentKind {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> AnyResult<Self> {
        match s {
            "CPU" => Ok(Self::CPU),
            "MEMORY" => Ok(Self::MEMORY),
            other => bail!(format!("unsupported kind: {other}")),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Experiment {
    pub id: String,
    pub kind: ExperimentKind,
    pub params: ExperimentParams,
    pub duration_seconds: u32,
    pub started_ts_seconds: i64,
    pub ends_ts_seconds: i64,
}

impl Experiment {
    pub fn new(
        id: String,
        kind: ExperimentKind,
        params: ExperimentParams,
        duration_seconds: u32,
        started_ts_seconds: i64,
    ) -> AnyResult<Self> {
        if id.trim().is_empty() {
            bail!("experiment_id is empty");
        }
        if duration_seconds == 0 {
            bail!("duration_seconds must be > 0");
        }
        // validate params against kind
        match (&kind, &params) {
            (ExperimentKind::CPU, ExperimentParams::Cpu { duty_percent }) => {
                if !(1..=100).contains(duty_percent) {
                    bail!("cpu duty_percent must be 1..=100");
                }
            }
            (ExperimentKind::MEMORY, ExperimentParams::Memory { memory_mb: _ }) => {}
            _ => bail!("kind and params mismatch"),
        }
        let ends_ts_seconds = started_ts_seconds + duration_seconds as i64;
        Ok(Self {
            id,
            kind,
            params,
            duration_seconds,
            started_ts_seconds,
            ends_ts_seconds,
        })
    }

    pub fn remaining_seconds(&self, now_ts: i64) -> u32 {
        if now_ts >= self.ends_ts_seconds {
            0
        } else {
            (self.ends_ts_seconds - now_ts) as u32
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StartParams {
    Cpu { duty_percent: u32 },
    Memory { memory_mb: u32 },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ExperimentParams {
    Cpu { duty_percent: u32 },
    Memory { memory_mb: u32 },
}

pub fn build_experiment(req: &StartRequest, now_ts: i64) -> AnyResult<Experiment> {
    let kind = ExperimentKind::from_str(&req.kind)?;
    let params = match (&kind, &req.params) {
        (ExperimentKind::CPU, StartParams::Cpu { duty_percent }) => ExperimentParams::Cpu {
            duty_percent: *duty_percent,
        },
        (ExperimentKind::MEMORY, StartParams::Memory { memory_mb }) => ExperimentParams::Memory {
            memory_mb: *memory_mb,
        },
        _ => bail!("kind and params mismatch"),
    };
    Experiment::new(
        req.experiment_id.clone(),
        kind,
        params,
        req.duration_seconds,
        now_ts,
    )
}
