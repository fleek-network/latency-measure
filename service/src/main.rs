mod types;

use axum::{routing::post, Json, Router};
use measure::MeasureRequest;
use tokio::task;
use ttfb::ttfb;
use types::{MeasureError, MeasureResponse};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", post(measure));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind to port 3000");

    println!("Listening on 3000");

    let _ = axum::serve(listener, app).await;
}

async fn measure(Json(target): Json<MeasureRequest>) -> Result<Json<MeasureResponse>, MeasureError> {
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
