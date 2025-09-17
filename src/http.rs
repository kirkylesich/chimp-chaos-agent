#![forbid(unsafe_code)]
#![deny(warnings)]
#![warn(clippy::pedantic)]

use actix_web::{get, post, web, App, HttpResponse, HttpServer};
use anyhow::Result as AnyResult;
use serde_json::json;
use tracing::{error, info, warn};

use crate::domain::build_experiment;
use crate::domain::{AppState, ExperimentParams, StartRequest};
use crate::metrics::Metrics;
use crate::validation::validate_start;

#[post("/experiments")]
pub async fn start(payload: web::Json<StartRequest>, data: web::Data<AppState>) -> HttpResponse {
    let req = payload.into_inner();
    let id = req.experiment_id.clone();
    info!(experiment=%id, kind=%req.kind, duration=req.duration_seconds, "start experiment request");
    if let Err(e) = validate_start(&req) {
        warn!(experiment=%id, error=%format!("{e:#}"), "invalid start request");
        return json_error(actix_web::http::StatusCode::BAD_REQUEST, &format!("{e:#}"));
    }
    if let Some(running_id) = data.ctrl.get_running_id() {
        warn!(experiment=%id, running=%running_id, "another experiment is in progress");
        return json_error(
            actix_web::http::StatusCode::CONFLICT,
            &format!("another experiment running: {running_id}"),
        );
    }
    let now = chrono::Utc::now().timestamp();
    let exp = match build_experiment(&req, now) {
        Ok(e) => e,
        Err(e) => return json_error(actix_web::http::StatusCode::BAD_REQUEST, &format!("{e:#}")),
    };
    data.ctrl.start(&id, &exp);
    data.metrics.mark_experiment_started(exp.duration_seconds);
    data.metrics.set_running_info(
        &exp.id,
        &exp.kind_label(),
        &exp.params_label(),
        exp.duration_seconds,
    );
    let mclone = data.metrics.clone();
    tokio::spawn(async move {
        match exp.params {
            ExperimentParams::Cpu { duty_percent } => {
                if let Err(e) =
                    cpu_load_task(id.clone(), duty_percent, exp.duration_seconds, mclone).await
                {
                    error!(experiment=%id, error=%format!("{e:#}"), "cpu load failed");
                }
            }
            ExperimentParams::Memory { memory_mb } => {
                if let Err(e) = memory_load_task(id.clone(), memory_mb, exp.duration_seconds).await
                {
                    error!(experiment=%id, error=%format!("{e:#}"), "memory load failed");
                }
            }
        }
        data.ctrl.finish(&id);
        data.metrics.clear_running_info(
            &exp.id,
            &exp.kind_label(),
            &exp.params_label(),
            exp.duration_seconds,
        );
        data.metrics.mark_experiment_finished();
        info!(experiment=%id, "experiment finished");
    });
    HttpResponse::Accepted().json(json!({"status":"ok"}))
}

#[post("/experiments/{id}/stop")]
pub async fn stop(path: web::Path<String>, data: web::Data<AppState>) -> HttpResponse {
    let id = path.into_inner();
    let mut map = data.ctrl.state.lock();
    match map.get_mut(&id) {
        Some(st) => {
            st.running = false;
            info!(experiment=%id, "stop experiment request");
            HttpResponse::Ok().json(json!({"status":"ok"}))
        }
        None => {
            warn!(experiment=%id, "stop: not found");
            json_error(
                actix_web::http::StatusCode::NOT_FOUND,
                "experiment not found",
            )
        }
    }
}

#[get("/healthz")]
pub async fn healthz() -> HttpResponse {
    HttpResponse::Ok().json(json!({"status":"ok"}))
}

#[get("/experiments/{id}/status")]
pub async fn status(path: web::Path<String>, data: web::Data<AppState>) -> HttpResponse {
    let id = path.into_inner();
    let map = data.ctrl.state.lock();
    match map.get(&id).cloned() {
        Some(st) => HttpResponse::Ok().json(st),
        None => json_error(
            actix_web::http::StatusCode::NOT_FOUND,
            "experiment not found",
        ),
    }
}

#[get("/metrics")]
pub async fn scrape_metrics(data: web::Data<AppState>) -> HttpResponse {
    match data.metrics.encode_text() {
        Ok(buf) => HttpResponse::Ok()
            .content_type("text/plain; version=0.0.4")
            .body(buf),
        Err(e) => {
            error!(error=%format!("{e:#}"), "encode metrics failed");
            HttpResponse::InternalServerError().body("encode metrics failed")
        }
    }
}

async fn cpu_load_task(
    _experiment_id: String,
    cpu_percent: u32,
    duration_seconds: u32,
    mtr: Metrics,
) -> AnyResult<()> {
    crate::lib_cpu::cpu_load(_experiment_id, cpu_percent, duration_seconds, mtr).await
}

async fn memory_load_task(
    _experiment_id: String,
    memory_mb: u32,
    duration_seconds: u32,
) -> AnyResult<()> {
    crate::lib_mem::memory_load(_experiment_id, memory_mb, duration_seconds).await
}

pub async fn serve(bind: &str) -> std::io::Result<()> {
    let metrics = Metrics::new().map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::Other, format!("metrics init: {e:#}"))
    })?;
    let state = AppState {
        ctrl: crate::domain::LoadController::default(),
        metrics,
    };
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

fn json_error(code: actix_web::http::StatusCode, reason: &str) -> HttpResponse {
    HttpResponse::build(code).json(json!({"status":"error","reason":reason}))
}
