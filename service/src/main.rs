mod types;

use std::time::{Duration, Instant};

use axum::{routing::post, Json, Router};
use measure::{
    MeasureDurationRequest, MeasureError, MeasureRequest, MeasureResponse,
};
use reqwest::{Client, Method};
use tokio::task;
use ttfb::ttfb;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/ttfb", post(measure_ttfb))
        .route("/duration", post(measure_duration));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind to port 3000");

    println!("Listening on 3000");

    let _ = axum::serve(listener, app).await;
}

async fn measure_ttfb(
    Json(target): Json<MeasureRequest>,
) -> Result<Json<MeasureResponse>, MeasureError> {
    let target = target.target;
    println!("target_request_url: {:?}", target);

    let handle = task::spawn_blocking(move || {
        ttfb(&target, true).map(|outcome| {
            let response: MeasureResponse = outcome.into();
            Json(response)
        })
    });

    match handle.await {
        Ok(result) => result.map_err(MeasureError::from),
        Err(e) => Err(MeasureError::from(e)),
    }
}

async fn measure_duration(
    Json(target): Json<MeasureDurationRequest>,
) -> Result<Json<MeasureResponse>, MeasureError> {
    let client = Client::new();

    let method: Method = match target.method.to_uppercase().as_str() {
        "GET" => Method::GET,
        "POST" => Method::POST,
        "PUT" => Method::PUT,
        "DELETE" => Method::DELETE,
        _ => Method::GET,
    };

    let mut request_builder = client.request(method, &target.target);

    if let Some(headers) = target.headers {
        for (key, value) in headers {
            request_builder = request_builder.header(key, value);
        }
    }

    if let Some(body) = target.body {
        request_builder = request_builder.body(body);
    }

    let start = Instant::now();
    let response = request_builder.send().await;
    let duration = start.elapsed();

    match response {
        Ok(response) => {
            if !response.status().is_success() {
                return Err(MeasureError::HttpError(response.status()));
            }

            Ok(Json(MeasureResponse {
                ip: "".to_string(),
                dns_lookup_duration: None,
                tcp_connect_duration: Duration::from_secs(0),
                http_get_send_duration: Duration::from_secs(0),
                ttfb_duration: Duration::from_secs(0),
                tls_handshake_duration: None,
                overall_duration: Some(duration),
            }))
        }
        Err(e) => Err(MeasureError::from(e)),
    }
}
