#![forbid(unsafe_code)]
#![deny(warnings)]
#![warn(clippy::pedantic)]

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use parking_lot::Mutex;
use prometheus::{Encoder, IntCounter, IntGauge, Opts, Registry, TextEncoder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[derive(Clone)]
pub struct AppState {
    pub ctrl: LoadController,
    pub metrics: Metrics,
}

#[derive(Clone, Default)]
pub struct LoadController {
    state: Arc<Mutex<HashMap<String, ExperimentState>>>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ExperimentState {
    pub running: bool,
    pub kind: String,
}

#[derive(Clone)]
pub struct Metrics {
    pub registry: Registry,
    pub cpu_hog_active: IntGauge,
    pub cpu_hog_duty_percent: IntGauge,
    pub cpu_seconds_total: IntCounter,
}

impl Metrics {
    pub fn new() -> Self {
        let registry = Registry::new();
        let cpu_hog_active = IntGauge::with_opts(Opts::new("agent_cpu_hog_active", "active flag")).unwrap();
        let cpu_hog_duty_percent = IntGauge::with_opts(Opts::new("agent_cpu_hog_duty_percent", "duty percent")).unwrap();
        let cpu_seconds_total = IntCounter::with_opts(Opts::new("agent_cpu_seconds_total", "cpu seconds" )).unwrap();
        registry.register(Box::new(cpu_hog_active.clone())).ok();
        registry.register(Box::new(cpu_hog_duty_percent.clone())).ok();
        registry.register(Box::new(cpu_seconds_total.clone())).ok();
        Self { registry, cpu_hog_active, cpu_hog_duty_percent, cpu_seconds_total }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartRequest { pub experiment_id: String, pub kind: String, pub cpu_percent: Option<u32>, pub memory_mb: Option<u32>, pub duration_seconds: u32 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Empty {}

#[post("/experiments")]
pub async fn start(payload: web::Json<StartRequest>, data: web::Data<AppState>) -> impl Responder {
    let req = payload.into_inner();
    let id = req.experiment_id.clone();
    {
        let mut map = data.ctrl.state.lock();
        map.insert(id.clone(), ExperimentState { running: true, kind: req.kind.clone() });
    }
    let mclone = data.metrics.clone();
    tokio::spawn(async move {
        match req.kind.as_str() {
            "CPU" => cpu_load(id.clone(), req.cpu_percent.unwrap_or(50), req.duration_seconds, mclone).await,
            "MEMORY" => memory_load(id.clone(), req.memory_mb.unwrap_or(50), req.duration_seconds).await,
            _ => cpu_load(id.clone(), req.cpu_percent.unwrap_or(50), req.duration_seconds, mclone).await,
        }
    });
    HttpResponse::Ok().finish()
}

#[post("/experiments/{id}/stop")]
pub async fn stop(path: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    let id = path.into_inner();
    let mut map = data.ctrl.state.lock();
    if let Some(st) = map.get_mut(&id) { st.running = false; }
    HttpResponse::Ok().finish()
}

#[get("/healthz")]
pub async fn healthz() -> impl Responder { HttpResponse::Ok().body("ok") }

#[get("/experiments/{id}/status")]
pub async fn status(path: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    let id = path.into_inner();
    let map = data.ctrl.state.lock();
    let st = map.get(&id).cloned().unwrap_or_default();
    HttpResponse::Ok().json(st)
}

#[get("/metrics")]
pub async fn scrape_metrics(data: web::Data<AppState>) -> impl Responder {
    let mut buf = Vec::new();
    let encoder = TextEncoder::new();
    let mf = data.metrics.registry.gather();
    encoder.encode(&mf, &mut buf).ok();
    HttpResponse::Ok().content_type("text/plain; version=0.0.4").body(buf)
}

async fn cpu_load(_experiment_id: String, cpu_percent: u32, duration_seconds: u32, mtr: Metrics) {
    let cpu_percent = cpu_percent.max(1).min(100);
    mtr.cpu_hog_active.set(1);
    mtr.cpu_hog_duty_percent.set(cpu_percent as i64);
    let on = Duration::from_millis(cpu_percent as u64);
    let off = Duration::from_millis(100 - cpu_percent as u64);
    let end = tokio::time::Instant::now() + Duration::from_secs(u64::from(duration_seconds));
    let mut _secs = 0u64;
    while tokio::time::Instant::now() < end {
        let spin_until = tokio::time::Instant::now() + on;
        while tokio::time::Instant::now() < spin_until { std::hint::spin_loop(); }
        sleep(off).await;
        _secs += 1;
        mtr.cpu_seconds_total.inc();
    }
    mtr.cpu_hog_active.set(0);
}

async fn memory_load(_experiment_id: String, memory_mb: u32, duration_seconds: u32) {
    let bytes = (memory_mb as usize).saturating_mul(1024 * 1024);
    let mut buf = Vec::<u8>::new();
    if bytes > 0 { buf.resize(bytes, 0u8); }
    let end = tokio::time::Instant::now() + Duration::from_secs(u64::from(duration_seconds));
    while tokio::time::Instant::now() < end {
        if !buf.is_empty() { buf[0] = buf[0].wrapping_add(1); }
        sleep(Duration::from_millis(50)).await;
    }
}

pub async fn serve(bind: &str) -> std::io::Result<()> {
    let state = AppState { ctrl: LoadController::default(), metrics: Metrics::new() };
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(healthz)
            .service(start)
            .service(stop)
            .service(status)
            .service(scrape_metrics)
    })
    .bind(bind)?
    .run()
    .await
}


