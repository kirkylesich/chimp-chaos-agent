#![forbid(unsafe_code)]
#![deny(warnings)]
#![warn(clippy::pedantic)]

pub use chimp_chaos_agent::serve;
use tracing::info;

fn init_tracing() {
    let fmt = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env());
    fmt.json().init();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();
    let bind = "0.0.0.0:50051";
    info!(bind, "starting agent");
    serve(bind).await?;
    Ok(())
}
