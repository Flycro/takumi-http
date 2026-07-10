use axum::Json;
use serde::{Deserialize, Serialize};
use takumi::prelude::Node;

use crate::error::ApiResult;

#[derive(Debug, Deserialize)]
pub struct ExtractUrlsRequest {
    pub node: Node,
}

#[derive(Serialize)]
pub struct ExtractUrlsResponse {
    pub urls: Vec<String>,
}

pub async fn extract_urls(
    Json(request): Json<ExtractUrlsRequest>,
) -> ApiResult<Json<ExtractUrlsResponse>> {
    let urls: Vec<String> = request.node.image_urls().map(|s| s.to_string()).collect();

    Ok(Json(ExtractUrlsResponse { urls }))
}
