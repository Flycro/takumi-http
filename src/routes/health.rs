use axum::{Json, extract::State};
use serde::Serialize;

use crate::state::SharedState;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub version: &'static str,
    pub fonts_loaded: usize,
}

pub async fn health(State(state): State<SharedState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy",
        version: env!("CARGO_PKG_VERSION"),
        fonts_loaded: state.fonts_loaded,
    })
}
