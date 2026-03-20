use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};
use takumi::resources::image::ImageSource;

use crate::{
    error::{ApiError, ApiResult},
    state::SharedState,
};

#[derive(Debug, Deserialize)]
pub struct AddImageRequest {
    pub src: String,
    pub data: String,
}

#[derive(Serialize)]
pub struct AddImageResponse {
    pub src: String,
    pub message: &'static str,
}

#[derive(Serialize)]
pub struct ClearImagesResponse {
    pub message: &'static str,
}

pub async fn add_image(
    State(state): State<SharedState>,
    Json(request): Json<AddImageRequest>,
) -> ApiResult<impl IntoResponse> {
    if !state.config.enable_cache {
        return Err(ApiError::CacheDisabled);
    }

    let data = STANDARD
        .decode(&request.data)
        .map_err(|e| ApiError::BadRequest(format!("Invalid base64: {e}")))?;

    let image_source = ImageSource::from_bytes(&data)
        .map_err(|e| ApiError::ImageDecodeError(format!("{e:?}")))?;

    let mut context = state.context.write().await;
    context
        .persistent_image_store_mut()
        .insert(request.src.clone(), image_source);

    Ok((
        StatusCode::CREATED,
        Json(AddImageResponse {
            src: request.src,
            message: "Image added to cache",
        }),
    ))
}

pub async fn clear_images(
    State(state): State<SharedState>,
) -> ApiResult<Json<ClearImagesResponse>> {
    let mut context = state.context.write().await;
    context.persistent_image_store_mut().clear();

    Ok(Json(ClearImagesResponse {
        message: "Image cache cleared",
    }))
}
