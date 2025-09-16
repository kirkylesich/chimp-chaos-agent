#![forbid(unsafe_code)]
#![deny(warnings)]
#![warn(clippy::pedantic)]

use chimp_chaos_agent::domain::{StartParams, StartRequest};
use chimp_chaos_agent::validation::validate_start;

#[test]
fn ok_cpu_defaults() {
    let r = StartRequest {
        experiment_id: "e1".into(),
        kind: "CPU".into(),
        duration_seconds: 1,
        params: StartParams::Cpu { duty_percent: 50 },
    };
    assert!(validate_start(&r).is_ok());
}

#[test]
fn ok_memory_defaults() {
    let r = StartRequest {
        experiment_id: "e1".into(),
        kind: "MEMORY".into(),
        duration_seconds: 1,
        params: StartParams::Memory { memory_mb: 10 },
    };
    assert!(validate_start(&r).is_ok());
}

#[test]
fn err_empty_experiment() {
    let r = StartRequest {
        experiment_id: " ".into(),
        kind: "CPU".into(),
        duration_seconds: 1,
        params: StartParams::Cpu { duty_percent: 10 },
    };
    assert!(validate_start(&r).is_err());
}

#[test]
fn err_zero_duration() {
    let r = StartRequest {
        experiment_id: "e".into(),
        kind: "CPU".into(),
        duration_seconds: 0,
        params: StartParams::Cpu { duty_percent: 10 },
    };
    assert!(validate_start(&r).is_err());
}

#[test]
fn err_cpu_percent_range() {
    let r1 = StartRequest {
        experiment_id: "e".into(),
        kind: "CPU".into(),
        duration_seconds: 1,
        params: StartParams::Cpu { duty_percent: 0 },
    };
    assert!(validate_start(&r1).is_err());
    let r2 = StartRequest {
        experiment_id: "e".into(),
        kind: "CPU".into(),
        duration_seconds: 1,
        params: StartParams::Cpu { duty_percent: 101 },
    };
    assert!(validate_start(&r2).is_err());
}

#[test]
fn err_kind_unsupported() {
    let r = StartRequest {
        experiment_id: "e".into(),
        kind: "NET".into(),
        duration_seconds: 1,
        params: StartParams::Cpu { duty_percent: 10 },
    };
    assert!(validate_start(&r).is_err());
}
