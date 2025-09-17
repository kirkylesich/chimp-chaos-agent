#![forbid(unsafe_code)]
#![deny(warnings)]
#![warn(clippy::pedantic)]

use anyhow::Result as AnyResult;

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
}
