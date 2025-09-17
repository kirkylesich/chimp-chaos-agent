#![forbid(unsafe_code)]
#![deny(warnings)]
#![warn(clippy::pedantic)]

pub mod domain;
pub mod http;
pub mod lib_cpu;
pub mod lib_mem;
pub mod metrics;
pub mod service;
pub mod validation;

pub use domain::{AppState, ExperimentState, LoadController, StartRequest};
pub use http::serve;
pub use http::{healthz, scrape_metrics, start, status, stop};
pub use metrics::Metrics;
pub use service::ExperimentRunner;
pub use validation::validate_start;
