#![forbid(unsafe_code)]
#![deny(warnings)]
#![warn(clippy::pedantic)]

use anyhow::Result as AnyResult;
use serde::Serialize;

use crate::domain::{Experiment, ExperimentParams, ExperimentState, LoadController, StartRequest};
use crate::metrics::Metrics;
use crate::validation::validate_start;

#[derive(Clone)]
pub struct ExperimentRunner {
    ctrl: LoadController,
    metrics: Metrics,
}

impl ExperimentRunner {
    pub fn new(ctrl: LoadController, metrics: Metrics) -> Self {
        Self { ctrl, metrics }
    }

    pub fn running_id(&self) -> Option<String> {
        self.ctrl.get_running_id()
    }

    pub fn validate_request(&self, req: &StartRequest) -> AnyResult<()> {
        validate_start(req).map_err(|e| anyhow::anyhow!(e))
    }

    pub fn create_experiment(&self, req: &StartRequest, now_ts: i64) -> AnyResult<Experiment> {
        Experiment::new_from_start_request(req, now_ts)
    }

    pub fn begin(&self, exp: &Experiment) {
        self.ctrl.start(&exp.id, exp);
        self.metrics.mark_experiment_started(exp.duration_seconds);
        self.metrics.set_running_info(
            &exp.id,
            &exp.kind_label(),
            &exp.params_label(),
            exp.duration_seconds,
        );
    }

    pub fn finish(&self, exp: &Experiment) {
        self.ctrl.finish(&exp.id);
        self.metrics.clear_running_info(
            &exp.id,
            &exp.kind_label(),
            &exp.params_label(),
            exp.duration_seconds,
        );
        self.metrics.mark_experiment_finished();
    }

    pub async fn run_to_completion(self, exp: Experiment) {
        match exp.params {
            ExperimentParams::Cpu { duty_percent } => {
                let _ = crate::lib_cpu::cpu_load(
                    exp.id.clone(),
                    duty_percent,
                    exp.duration_seconds,
                    self.metrics.clone(),
                )
                .await;
            }
            ExperimentParams::Memory { memory_mb } => {
                let _ =
                    crate::lib_mem::memory_load(exp.id.clone(), memory_mb, exp.duration_seconds)
                        .await;
            }
        }
        self.finish(&exp);
    }

    pub fn stop(&self, id: &str) -> bool {
        let mut map = self.ctrl.state.lock();
        if let Some(st) = map.get_mut(id) {
            st.running = false;
            true
        } else {
            false
        }
    }

    pub fn status(&self, id: &str) -> Option<ExperimentState> {
        let map = self.ctrl.state.lock();
        map.get(id).cloned()
    }

    pub fn encode_metrics(&self) -> AnyResult<Vec<u8>> {
        self.metrics.encode_text()
    }

    pub fn health(&self) -> HealthReport {
        let map = self.ctrl.state.lock();
        let running_entry = map.iter().find(|(_, st)| st.running);
        let running = running_entry.is_some();
        let running_id = running_entry.map(|(k, _)| k.clone());
        let invariants_ok = map.values().all(|st| {
            let duration = i64::from(st.total_duration_seconds);
            let diff = st.ends_ts_seconds - st.started_ts_seconds;
            diff == duration
                && st.ends_ts_seconds >= st.started_ts_seconds
                && st.remaining_seconds <= st.total_duration_seconds
        });
        let metrics_ok = self.metrics.encode_text().is_ok();
        let registry_metrics = self.metrics.registry.gather().len();
        let status = if metrics_ok && invariants_ok {
            "ok"
        } else {
            "degraded"
        };
        HealthReport {
            status: status.to_string(),
            running,
            running_id,
            metrics_ok,
            registry_metrics,
            invariants_ok,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct HealthReport {
    pub status: String,
    pub running: bool,
    pub running_id: Option<String>,
    pub metrics_ok: bool,
    pub registry_metrics: usize,
    pub invariants_ok: bool,
}
