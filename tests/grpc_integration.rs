#![forbid(unsafe_code)]
#![deny(warnings)]
#![warn(clippy::pedantic)]

use chimp_chaos_agent::pb::{agent_client::AgentClient, StartRequest, StopRequest};
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn start_then_stop() {
    // Spin up server in background on random port
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        let svc = chimp_chaos_agent::AgentService::default();
        tonic::transport::Server::builder()
            .add_service(chimp_chaos_agent::pb::agent_server::AgentServer::new(svc))
            .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener))
            .await
            .unwrap();
    });

    let mut client = AgentClient::connect(format!("http://{}", addr)).await.unwrap();
    client.start(StartRequest { cpu_percent: 10, memory_mb: 0, duration_seconds: 1, kind: chimp_chaos_agent::pb::Kind::Cpu as i32 }).await.unwrap();
    sleep(Duration::from_millis(100)).await;
    client.stop(StopRequest {}).await.unwrap();
}

