#![forbid(unsafe_code)]
#![deny(warnings)]
#![warn(clippy::pedantic)]

#[tokio::test]
async fn cpu_runs() {
    let m = chimp_chaos_agent::metrics::Metrics::new().expect("metrics");
    chimp_chaos_agent::lib_cpu::cpu_load("e".into(), 10, 1, m)
        .await
        .expect("ok");
}

#[tokio::test]
async fn mem_runs() {
    chimp_chaos_agent::lib_mem::memory_load("e".into(), 1, 1)
        .await
        .expect("ok");
}
