use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::Duration};
use thiserror::Error;
use ttfb::{TtfbError, TtfbOutcome};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeasureRequest {
    pub target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeasureDurationRequest {
    pub target: String,
    pub method: String,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeasureResponse {
    pub ip: String,
    pub dns_lookup_duration: Option<Duration>,
    pub tcp_connect_duration: Duration,
    pub http_get_send_duration: Duration,
    pub ttfb_duration: Duration,
    pub tls_handshake_duration: Option<Duration>,
    pub overall_duration: Option<Duration>,
}

#[derive(Error, Debug)]
pub enum MeasureError {
    #[error("TTFB error: {0}")]
    Ttfb(#[from] TtfbError),
    #[error("Blocking task spawn error: {0}")]
    BlockingTaskSpawn(#[from] tokio::task::JoinError),
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[allow(dead_code)]
    #[error("HTTP error: {0}")]
    HttpError(reqwest::StatusCode),
}

impl From<TtfbOutcome> for MeasureResponse {
    fn from(outcome: TtfbOutcome) -> Self {
        MeasureResponse {
            ip: outcome.ip_addr().to_string(),
            dns_lookup_duration: outcome.dns_lookup_duration().map(|d| d.relative()),
            tcp_connect_duration: outcome.tcp_connect_duration().relative(),
            http_get_send_duration: outcome.http_get_send_duration().relative(),
            ttfb_duration: outcome.ttfb_duration().relative(),
            tls_handshake_duration: outcome.tls_handshake_duration().map(|d| d.relative()),
            overall_duration: None,
        }
    }
}

impl IntoResponse for MeasureError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}
