#![forbid(unsafe_code)]
#![deny(warnings)]
#![warn(clippy::pedantic)]

use tonic::transport::Server;
use tracing::info;
pub use chimp_chaos_agent::{pb, AgentService};

fn init_tracing() {
    let fmt = tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::from_default_env());
    fmt.json().init();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();
    let addr = "0.0.0.0:50051".parse()?;
    let svc = AgentService::default();
    info!(?addr, "starting agent");
    Server::builder()
        .add_service(pb::agent_server::AgentServer::new(svc))
        .serve(addr)
        .await?;
    Ok(())
}

