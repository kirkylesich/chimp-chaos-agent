#![forbid(unsafe_code)]
#![deny(warnings)]
#![warn(clippy::pedantic)]

use anyhow::{anyhow, Result as AnyResult};
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

impl LoadController {
    pub fn get_running_id(&self) -> Option<String> {
        self.state
            .lock()
            .iter()
            .find(|(_, st)| st.running)
            .map(|(k, _)| k.clone())
    }

    pub fn start(&self, id: &str, exp: &Experiment) {
        let mut map = self.state.lock();
        map.insert(
            id.to_string(),
            ExperimentState {
                running: true,
                kind: match exp.kind {
                    ExperimentKind::CPU => "CPU".into(),
                    ExperimentKind::MEMORY => "MEMORY".into(),
                },
                total_duration_seconds: exp.duration_seconds,
                remaining_seconds: exp.duration_seconds,
                started_ts_seconds: exp.started_ts_seconds,
                ends_ts_seconds: exp.ends_ts_seconds,
            },
        );
    }

    pub fn finish(&self, id: &str) {
        let mut map = self.state.lock();
        if let Some(st) = map.get_mut(id) {
            st.running = false;
            st.remaining_seconds = 0;
        }
    }
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
    fn from_str(s: &str) -> Result<Self, anyhow::Error> {
        match s {
            "CPU" => Ok(Self::CPU),
            "MEMORY" => Ok(Self::MEMORY),
            other => Err(anyhow::anyhow!("unsupported kind: {other}")),
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
    ) -> Self {
        let ends_ts_seconds = started_ts_seconds + duration_seconds as i64;
        Self {
            id,
            kind,
            params,
            duration_seconds,
            started_ts_seconds,
            ends_ts_seconds,
        }
    }

    pub fn remaining_seconds(&self, now_ts: i64) -> u32 {
        if now_ts >= self.ends_ts_seconds {
            0
        } else {
            (self.ends_ts_seconds - now_ts) as u32
        }
    }

    pub fn kind_label(&self) -> String {
        self.kind.to_string()
    }

    pub fn params_label(&self) -> String {
        match &self.params {
            ExperimentParams::Cpu { duty_percent } => format!("duty_percent={}", duty_percent),
            ExperimentParams::Memory { memory_mb } => format!("memory_mb={}", memory_mb),
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
        _ => return Err(anyhow!("kind and params mismatch")),
    };
    Ok(Experiment::new(
        req.experiment_id.clone(),
        kind,
        params,
        req.duration_seconds,
        now_ts,
    ))
}
