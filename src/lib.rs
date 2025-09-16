#![forbid(unsafe_code)]
#![deny(warnings)]
#![warn(clippy::pedantic)]

use parking_lot::Mutex;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tonic::{Request, Response, Status};

pub mod pb {
    tonic::include_proto!("chimp.agent.v1");
}

#[derive(Default, Clone)]
pub struct LoadController {
    state: Arc<Mutex<LoadState>>,
}

#[derive(Default)]
pub struct LoadState {
    running: bool,
}

#[derive(Default, Clone)]
pub struct AgentService {
    ctrl: LoadController,
}

#[tonic::async_trait]
impl pb::agent_server::Agent for AgentService {
    async fn start(&self, request: Request<pb::StartRequest>) -> Result<Response<pb::Empty>, Status> {
        let args = request.into_inner();
        let ctrl = self.ctrl.clone();
        {
            let mut st = ctrl.state.lock();
            st.running = true;
        }
        tokio::spawn(async move {
            match args.kind {
                x if x == pb::Kind::Cpu as i32 => cpu_load(ctrl.clone(), args.cpu_percent, args.duration_seconds).await,
                x if x == pb::Kind::Memory as i32 => memory_load(ctrl.clone(), args.memory_mb, args.duration_seconds).await,
                _ => cpu_load(ctrl.clone(), args.cpu_percent, args.duration_seconds).await,
            }
        });
        Ok(Response::new(pb::Empty {}))
    }

    async fn stop(&self, _request: Request<pb::StopRequest>) -> Result<Response<pb::Empty>, Status> {
        let mut st = self.ctrl.state.lock();
        st.running = false;
        Ok(Response::new(pb::Empty {}))
    }
}

async fn cpu_load(ctrl: LoadController, cpu_percent: u32, duration_seconds: u32) {
    let cpu_percent = cpu_percent.max(1).min(100);
    let on = Duration::from_millis(cpu_percent as u64);
    let off = Duration::from_millis(100 - cpu_percent as u64);
    let end = tokio::time::Instant::now() + Duration::from_secs(u64::from(duration_seconds));
    while tokio::time::Instant::now() < end {
        if !ctrl.state.lock().running { break; }
        let spin_until = tokio::time::Instant::now() + on;
        while tokio::time::Instant::now() < spin_until { std::hint::spin_loop(); }
        sleep(off).await;
    }
}

async fn memory_load(ctrl: LoadController, memory_mb: u32, duration_seconds: u32) {
    let bytes = (memory_mb as usize).saturating_mul(1024 * 1024);
    let mut buf = Vec::<u8>::new();
    if bytes > 0 { buf.resize(bytes, 0u8); }
    let end = tokio::time::Instant::now() + Duration::from_secs(u64::from(duration_seconds));
    while tokio::time::Instant::now() < end {
        if !ctrl.state.lock().running { break; }
        // Touch memory to keep it resident
        if !buf.is_empty() { buf[0] = buf[0].wrapping_add(1); }
        sleep(Duration::from_millis(50)).await;
    }
}


