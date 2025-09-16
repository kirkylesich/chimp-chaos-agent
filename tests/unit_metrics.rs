#![forbid(unsafe_code)]
#![deny(warnings)]
#![warn(clippy::pedantic)]

use chimp_chaos_agent::metrics::Metrics;

#[test]
fn create_and_encode() {
    let m = Metrics::new().expect("metrics");
    let buf = m.encode_text().expect("encode");
    assert!(!buf.is_empty());
}

