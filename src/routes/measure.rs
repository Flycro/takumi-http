use axum::{Json, extract::State};
use takumi::{
    layout::Viewport,
    rendering::{MeasuredNode, RenderOptions, measure_layout},
};

use crate::{
    dto::measure::MeasureRequest,
    error::ApiResult,
    extractors::json_or_form::JsonOrMultipart,
    state::SharedState,
};

pub async fn measure(
    State(state): State<SharedState>,
    payload: JsonOrMultipart<MeasureRequest>,
) -> ApiResult<Json<MeasuredNode>> {
    let request = payload.data;
    let context = state.context.read().await;

    let viewport = Viewport::new((request.options.width, request.options.height))
        .with_device_pixel_ratio(request.options.device_pixel_ratio);

    let options = RenderOptions::builder()
        .viewport(viewport)
        .node(request.node)
        .global(&context)
        .build();

    let measured = measure_layout(options)?;

    Ok(Json(measured))
}
