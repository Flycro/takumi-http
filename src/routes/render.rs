use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::State,
    http::{HeaderMap, header::CONTENT_TYPE},
    response::{IntoResponse, Response},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use takumi::{
    layout::Viewport,
    rendering::{ImageOutputFormat, RenderOptionsBuilder, render as takumi_render, write_image},
    resources::image::{ImageSource, load_image_source_from_bytes},
};
use tokio::task::spawn_blocking;

use crate::{
    dto::render::{OutputFormat, RenderRequest},
    error::{ApiError, ApiResult},
    extractors::json_or_form::JsonOrMultipart,
    state::SharedState,
};

fn convert_format(format: &OutputFormat) -> ImageOutputFormat {
    match format {
        OutputFormat::Png => ImageOutputFormat::Png,
        OutputFormat::Jpeg => ImageOutputFormat::Jpeg,
        OutputFormat::Webp => ImageOutputFormat::WebP,
    }
}

pub async fn render(
    State(state): State<SharedState>,
    _headers: HeaderMap,
    payload: JsonOrMultipart<RenderRequest>,
) -> ApiResult<Response> {
    let request = payload.data;
    let uploaded_files = payload.files;

    let format = convert_format(&request.options.format);
    let quality = request.options.quality;

    let mut fetched_resources: HashMap<Arc<str>, Arc<ImageSource>> = HashMap::new();

    // Add resources from JSON (base64 encoded)
    for resource in request.fetched_resources {
        let data = STANDARD.decode(&resource.data).map_err(|e| {
            ApiError::BadRequest(format!("Invalid base64 in fetchedResources: {e}"))
        })?;

        let image_source = load_image_source_from_bytes(&data)
            .map_err(|e| ApiError::ImageDecodeError(format!("{e:?}")))?;

        fetched_resources.insert(Arc::from(resource.src), image_source);
    }

    // Add resources from multipart file uploads
    for (name, data) in uploaded_files {
        let image_source = load_image_source_from_bytes(&data)
            .map_err(|e| ApiError::ImageDecodeError(format!("Failed to decode {name}: {e:?}")))?;

        fetched_resources.insert(Arc::from(name), image_source);
    }

    let context = state.context.read().await;

    let mut viewport = Viewport::new(request.options.width, request.options.height);
    viewport.device_pixel_ratio = request.options.device_pixel_ratio;

    let node = request.node;
    let draw_debug_border = request.options.draw_debug_border;

    let options = RenderOptionsBuilder::default()
        .viewport(viewport)
        .node(node)
        .global(&context)
        .draw_debug_border(draw_debug_border)
        .fetched_resources(fetched_resources)
        .build()
        .map_err(|e| ApiError::Internal(format!("Failed to build render options: {e}")))?;

    let image = takumi_render(options)?;

    drop(context);

    let buffer = spawn_blocking(move || -> ApiResult<Vec<u8>> {
        let mut buf = Vec::new();
        write_image(&image, &mut buf, format, quality)?;
        Ok(buf)
    })
    .await
    .map_err(|e| ApiError::Internal(format!("Render task panicked: {e}")))??;

    Ok(([(CONTENT_TYPE, format.content_type())], buffer).into_response())
}
