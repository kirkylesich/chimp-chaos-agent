#![forbid(unsafe_code)]
#![deny(warnings)]
#![warn(clippy::pedantic)]

use actix_web::{test, App};
use chimp_chaos_agent::{
    healthz, scrape_metrics, start, status, stop, AppState, LoadController, Metrics,
};

#[actix_web::test]
async fn start_stop_and_metrics() {
    let state = AppState {
        ctrl: LoadController::default(),
        metrics: Metrics::new().unwrap(),
    };
    let app = test::init_service(
        App::new()
            .app_data(actix_web::web::Data::new(state))
            .service(healthz)
            .service(start)
            .service(stop)
            .service(status)
            .service(scrape_metrics),
    )
    .await;

    // healthz
    let req = test::TestRequest::get().uri("/healthz").to_request();
    let resp = test::call_service(&app, req).await;
    eprintln!("/metrics status: {}", resp.status());
    if !resp.status().is_success() {
        let body = test::read_body(resp).await;
        eprintln!("/metrics body: {}", String::from_utf8_lossy(&body));
        panic!("/metrics failed");
    }

    // start CPU
    let body = serde_json::json!({
        "experiment_id":"exp2",
        "kind":"CPU",
        "duration_seconds":1,
        "params": {"type":"CPU", "duty_percent":10}
    });
    let req = test::TestRequest::post()
        .uri("/experiments")
        .set_json(body)
        .to_request();
    let resp = test::call_service(&app, req).await;
    eprintln!("/experiments start status: {}", resp.status());
    if !resp.status().is_success() {
        let body = test::read_body(resp).await;
        eprintln!(
            "/experiments start body: {}",
            String::from_utf8_lossy(&body)
        );
        panic!("/experiments start failed");
    }

    // metrics scrape
    let req = test::TestRequest::get().uri("/metrics").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // stop
    let req = test::TestRequest::post()
        .uri("/experiments/exp2/stop")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}
