use axum::{
    extract::State,
    http::header::CONTENT_TYPE,
    response::{IntoResponse, Response},
};
use takumi::{
    layout::Viewport,
    rendering::{
        AnimationFrame as TakumiAnimationFrame, RenderOptionsBuilder, encode_animated_png,
        encode_animated_webp, render as takumi_render,
    },
};
use tokio::task::spawn_blocking;

use crate::{
    dto::animation::{AnimationFormat, AnimationRequest},
    error::{ApiError, ApiResult},
    extractors::json_or_form::JsonOrMultipart,
    state::SharedState,
};

pub async fn render_animation(
    State(state): State<SharedState>,
    payload: JsonOrMultipart<AnimationRequest>,
) -> ApiResult<Response> {
    let request = payload.data;

    if request.frames.is_empty() {
        return Err(ApiError::BadRequest("frames array cannot be empty".into()));
    }

    let context = state.context.read().await;

    let mut viewport = Viewport::new(request.options.width, request.options.height);
    viewport.device_pixel_ratio = request.options.device_pixel_ratio;

    let mut frames = Vec::with_capacity(request.frames.len());

    for frame in request.frames {
        let options = RenderOptionsBuilder::default()
            .viewport(viewport)
            .node(frame.node)
            .global(&context)
            .draw_debug_border(request.options.draw_debug_border)
            .build()
            .map_err(|e| ApiError::Internal(format!("Failed to build render options: {e}")))?;

        let image = takumi_render(options)?;
        frames.push(TakumiAnimationFrame::new(image, frame.duration_ms));
    }

    drop(context);

    let content_type = match request.options.format {
        AnimationFormat::Webp => "image/webp",
        AnimationFormat::Apng => "image/png",
    };

    let format = request.options.format;
    let buffer = spawn_blocking(move || -> ApiResult<Vec<u8>> {
        let mut buf = Vec::new();

        match format {
            AnimationFormat::Webp => {
                encode_animated_webp(&frames, &mut buf, false, false, None)?;
            }
            AnimationFormat::Apng => {
                encode_animated_png(&frames, &mut buf, None)?;
            }
        }

        Ok(buf)
    })
    .await
    .map_err(|e| ApiError::Internal(format!("Animation render task panicked: {e}")))??;

    Ok(([(CONTENT_TYPE, content_type)], buffer).into_response())
}
