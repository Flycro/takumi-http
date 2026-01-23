use axum::{Json, extract::State};
use takumi::{
    layout::Viewport,
    rendering::{MeasuredNode, RenderOptionsBuilder, measure_layout},
};

use crate::{
    dto::measure::MeasureRequest,
    error::{ApiError, ApiResult},
    extractors::json_or_form::JsonOrMultipart,
    state::SharedState,
};

pub async fn measure(
    State(state): State<SharedState>,
    payload: JsonOrMultipart<MeasureRequest>,
) -> ApiResult<Json<MeasuredNode>> {
    let request = payload.data;
    let context = state.context.read().await;

    let mut viewport = Viewport::new(request.options.width, request.options.height);
    viewport.device_pixel_ratio = request.options.device_pixel_ratio;

    let options = RenderOptionsBuilder::default()
        .viewport(viewport)
        .node(request.node)
        .global(&context)
        .build()
        .map_err(|e| ApiError::Internal(format!("Failed to build render options: {e}")))?;

    let measured = measure_layout(options)?;

    Ok(Json(measured))
}
