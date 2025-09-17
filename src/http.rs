#![forbid(unsafe_code)]
#![deny(warnings)]
#![warn(clippy::pedantic)]

use actix_web::{get, post, web, App, HttpResponse, HttpServer};
//
use serde_json::json;
use tracing::{error, info, warn};

use crate::domain::{AppState, StartRequest};
use crate::metrics::Metrics;
use crate::service::ExperimentRunner;
// validation performed by service

#[post("/experiments")]
pub async fn start(payload: web::Json<StartRequest>, data: web::Data<AppState>) -> HttpResponse {
    let req = payload.into_inner();
    let runner = ExperimentRunner::new(data.ctrl.clone(), data.metrics.clone());
    info!(experiment=%req.experiment_id, kind=%req.kind, duration=req.duration_seconds, "start experiment request");
    if let Some(running_id) = runner.running_id() {
        return json_error(
            actix_web::http::StatusCode::CONFLICT,
            &format!("another experiment running: {running_id}"),
        );
    }
    if let Err(e) = runner.validate_request(&req) {
        return json_error(actix_web::http::StatusCode::BAD_REQUEST, &format!("{e:#}"));
    }
    let now = chrono::Utc::now().timestamp();
    let exp = match runner.create_experiment(&req, now) {
        Ok(e) => e,
        Err(e) => return json_error(actix_web::http::StatusCode::BAD_REQUEST, &format!("{e:#}")),
    };
    runner.begin(&exp);
    let runner_clone = runner.clone();
    tokio::spawn(async move {
        runner_clone.run_to_completion(exp).await;
    });
    HttpResponse::Accepted().json(json!({"status":"ok"}))
}

#[post("/experiments/{id}/stop")]
pub async fn stop(path: web::Path<String>, data: web::Data<AppState>) -> HttpResponse {
    let id = path.into_inner();
    let runner = ExperimentRunner::new(data.ctrl.clone(), data.metrics.clone());
    if runner.stop(&id) {
        info!(experiment=%id, "stop experiment request");
        HttpResponse::Ok().json(json!({"status":"ok"}))
    } else {
        warn!(experiment=%id, "stop: not found");
        json_error(
            actix_web::http::StatusCode::NOT_FOUND,
            "experiment not found",
        )
    }
}

#[get("/healthz")]
pub async fn healthz() -> HttpResponse {
    HttpResponse::Ok().json(json!({"status":"ok"}))
}

#[get("/experiments/{id}/status")]
pub async fn status(path: web::Path<String>, data: web::Data<AppState>) -> HttpResponse {
    let id = path.into_inner();
    let runner = ExperimentRunner::new(data.ctrl.clone(), data.metrics.clone());
    match runner.status(&id) {
        Some(st) => HttpResponse::Ok().json(st),
        None => json_error(
            actix_web::http::StatusCode::NOT_FOUND,
            "experiment not found",
        ),
    }
}

#[get("/metrics")]
pub async fn scrape_metrics(data: web::Data<AppState>) -> HttpResponse {
    let runner = ExperimentRunner::new(data.ctrl.clone(), data.metrics.clone());
    match runner.encode_metrics() {
        Ok(buf) => HttpResponse::Ok()
            .content_type("text/plain; version=0.0.4")
            .body(buf),
        Err(e) => {
            error!(error=%format!("{e:#}"), "encode metrics failed");
            HttpResponse::InternalServerError().body("encode metrics failed")
        }
    }
}

// no-op: logic moved to service::ExperimentRunner

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
