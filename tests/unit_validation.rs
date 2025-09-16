#![forbid(unsafe_code)]
#![deny(warnings)]
#![warn(clippy::pedantic)]

use chimp_chaos_agent::validation::validate_start;
use chimp_chaos_agent::domain::StartRequest;

#[test]
fn ok_cpu_defaults() {
    let r = StartRequest { experiment_id: "e1".into(), kind: "CPU".into(), cpu_percent: Some(50), memory_mb: None, duration_seconds: 1 };
    assert!(validate_start(&r).is_ok());
}

#[test]
fn ok_memory_defaults() {
    let r = StartRequest { experiment_id: "e1".into(), kind: "MEMORY".into(), cpu_percent: None, memory_mb: Some(10), duration_seconds: 1 };
    assert!(validate_start(&r).is_ok());
}

#[test]
fn err_empty_experiment() {
    let r = StartRequest { experiment_id: " ".into(), kind: "CPU".into(), cpu_percent: Some(10), memory_mb: None, duration_seconds: 1 };
    assert!(validate_start(&r).is_err());
}

#[test]
fn err_zero_duration() {
    let r = StartRequest { experiment_id: "e".into(), kind: "CPU".into(), cpu_percent: Some(10), memory_mb: None, duration_seconds: 0 };
    assert!(validate_start(&r).is_err());
}

#[test]
fn err_cpu_percent_range() {
    let r1 = StartRequest { experiment_id: "e".into(), kind: "CPU".into(), cpu_percent: Some(0), memory_mb: None, duration_seconds: 1 };
    assert!(validate_start(&r1).is_err());
    let r2 = StartRequest { experiment_id: "e".into(), kind: "CPU".into(), cpu_percent: Some(101), memory_mb: None, duration_seconds: 1 };
    assert!(validate_start(&r2).is_err());
}

#[test]
fn err_kind_unsupported() {
    let r = StartRequest { experiment_id: "e".into(), kind: "NET".into(), cpu_percent: None, memory_mb: None, duration_seconds: 1 };
    assert!(validate_start(&r).is_err());
}

