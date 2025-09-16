#![forbid(unsafe_code)]
#![deny(warnings)]
#![warn(clippy::pedantic)]

use chimp_chaos_agent::domain::{Experiment, ExperimentKind, ExperimentParams};

#[test]
fn new_cpu_ok() {
    let e = Experiment::new(
        "e1".into(),
        ExperimentKind::CPU,
        ExperimentParams::Cpu { duty_percent: 50 },
        5,
        1000,
    ).expect("ok");
    assert_eq!(e.remaining_seconds(1000), 5);
}

#[test]
fn new_cpu_bad_percent() {
    let res = Experiment::new("e1".into(), ExperimentKind::CPU, ExperimentParams::Cpu { duty_percent: 0 }, 5, 1000);
    assert!(res.is_err());
}

#[test]
fn new_bad_duration() {
    let res = Experiment::new("e1".into(), ExperimentKind::MEMORY, ExperimentParams::Memory { memory_mb: 10 }, 0, 1000);
    assert!(res.is_err());
}

