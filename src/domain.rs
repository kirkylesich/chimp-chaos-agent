#![forbid(unsafe_code)]
#![deny(warnings)]
#![warn(clippy::pedantic)]

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ExperimentState {
    pub running: bool,
    pub kind: String,
}

#[derive(Clone, Default)]
pub struct LoadController {
    pub state: Arc<Mutex<HashMap<String, ExperimentState>>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StartRequest {
    pub experiment_id: String,
    pub kind: String,
    pub cpu_percent: Option<u32>,
    pub memory_mb: Option<u32>,
    pub duration_seconds: u32,
}

#[derive(Clone)]
pub struct AppState {
    pub ctrl: LoadController,
    pub metrics: crate::metrics::Metrics,
}

