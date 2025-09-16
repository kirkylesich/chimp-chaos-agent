#![forbid(unsafe_code)]
#![deny(warnings)]
#![warn(clippy::pedantic)]

use anyhow::{Context, Result as AnyResult};
use prometheus::{Encoder, IntCounter, IntGauge, Opts, Registry, TextEncoder};

#[derive(Clone)]
pub struct Metrics {
    pub registry: Registry,
    pub cpu_hog_active: IntGauge,
    pub cpu_hog_duty_percent: IntGauge,
    pub cpu_seconds_total: IntCounter,
    pub experiment_active: IntGauge,
    pub experiment_total_seconds: IntGauge,
    pub experiment_remaining_seconds: IntGauge,
}

impl Metrics {
    pub fn new() -> AnyResult<Self> {
        let registry = Registry::new();
        let cpu_hog_active = IntGauge::with_opts(Opts::new("agent_cpu_hog_active", "active flag"))
            .context("create cpu_hog_active")?;
        let cpu_hog_duty_percent =
            IntGauge::with_opts(Opts::new("agent_cpu_hog_duty_percent", "duty percent"))
                .context("create cpu_hog_duty_percent")?;
        let cpu_seconds_total =
            IntCounter::with_opts(Opts::new("agent_cpu_seconds_total", "cpu seconds"))
                .context("create cpu_seconds_total")?;
        registry
            .register(Box::new(cpu_hog_active.clone()))
            .context("register cpu_hog_active")?;
        registry
            .register(Box::new(cpu_hog_duty_percent.clone()))
            .context("register cpu_hog_duty_percent")?;
        registry
            .register(Box::new(cpu_seconds_total.clone()))
            .context("register cpu_seconds_total")?;
        let experiment_active = IntGauge::with_opts(Opts::new(
            "agent_experiment_active",
            "1 if an experiment is running",
        ))
        .context("create experiment_active")?;
        let experiment_total_seconds = IntGauge::with_opts(Opts::new(
            "agent_experiment_total_seconds",
            "configured total seconds",
        ))
        .context("create experiment_total_seconds")?;
        let experiment_remaining_seconds = IntGauge::with_opts(Opts::new(
            "agent_experiment_remaining_seconds",
            "remaining seconds",
        ))
        .context("create experiment_remaining_seconds")?;
        registry
            .register(Box::new(experiment_active.clone()))
            .context("register experiment_active")?;
        registry
            .register(Box::new(experiment_total_seconds.clone()))
            .context("register experiment_total_seconds")?;
        registry
            .register(Box::new(experiment_remaining_seconds.clone()))
            .context("register experiment_remaining_seconds")?;
        Ok(Self {
            registry,
            cpu_hog_active,
            cpu_hog_duty_percent,
            cpu_seconds_total,
            experiment_active,
            experiment_total_seconds,
            experiment_remaining_seconds,
        })
    }

    pub fn encode_text(&self) -> AnyResult<Vec<u8>> {
        let mut buf = Vec::new();
        let encoder = TextEncoder::new();
        let mf = self.registry.gather();
        encoder.encode(&mf, &mut buf).context("encode metrics")?;
        Ok(buf)
    }
}
