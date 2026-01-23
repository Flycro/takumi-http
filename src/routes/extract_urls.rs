use axum::Json;
use serde::{Deserialize, Serialize};
use takumi::{
    layout::node::{Node, NodeKind},
    resources::task::FetchTaskCollection,
};

use crate::error::ApiResult;

#[derive(Debug, Deserialize)]
pub struct ExtractUrlsRequest {
    pub node: NodeKind,
}

#[derive(Serialize)]
pub struct ExtractUrlsResponse {
    pub urls: Vec<String>,
}

pub async fn extract_urls(
    Json(request): Json<ExtractUrlsRequest>,
) -> ApiResult<Json<ExtractUrlsResponse>> {
    let mut collection = FetchTaskCollection::default();
    request.node.collect_fetch_tasks(&mut collection);

    let urls: Vec<String> = collection
        .into_inner()
        .into_iter()
        .map(|arc| arc.to_string())
        .collect();

    Ok(Json(ExtractUrlsResponse { urls }))
}
