use axum::{Json, extract::State};
use base64::{Engine, engine::general_purpose::STANDARD};
use takumi::{
    measure as takumi_measure,
    prelude::{ImageSource, MeasuredNode, RenderOptions, Viewport},
};

use crate::{
    dto::measure::MeasureRequest,
    error::{ApiError, ApiResult},
    extractors::json_or_form::JsonOrMultipart,
    resources::{font_families, lang, resolve_fonts, resolve_images, resolve_node},
    state::SharedState,
};

pub async fn measure(
    State(state): State<SharedState>,
    payload: JsonOrMultipart<MeasureRequest>,
) -> ApiResult<Json<MeasuredNode>> {
    let request = payload.data;
    let uploaded_files = payload.files;
    let timeout_ms = request.options.fetch_timeout_ms.unwrap_or(10_000);
    let fonts = resolve_fonts(
        &state,
        request.options.fonts,
        timeout_ms,
        request.options.fetch_cache,
    )
    .await?;
    let node = resolve_node(request.node, request.html)?;
    let mut images = state.images.read().await.clone();
    for resource in request.fetched_resources {
        let data = STANDARD
            .decode(resource.data)
            .map_err(|error| ApiError::BadRequest(format!("Invalid base64: {error}")))?;
        let image = ImageSource::from_bytes(&data)
            .map_err(|error| ApiError::ImageDecodeError(format!("{error:?}")))?;
        images.insert(resource.src.into(), image);
    }
    for (name, data) in uploaded_files {
        let image = ImageSource::from_bytes(&data)
            .map_err(|error| ApiError::ImageDecodeError(format!("{error:?}")))?;
        images.insert(name.into(), image);
    }
    let images = resolve_images(
        &state,
        &node,
        images,
        request.options.fetch_images,
        timeout_ms,
        request.options.fetch_cache,
    )
    .await?;

    let viewport = Viewport::new((request.options.width, request.options.height))
        .with_device_pixel_ratio(request.options.device_pixel_ratio);

    let options = RenderOptions::builder()
        .viewport(viewport)
        .node(node)
        .fonts(&fonts)
        .images(images)
        .font_families(font_families(request.options.font_families))
        .lang(lang(request.options.lang)?)
        .build();

    let measured = takumi_measure(options)?;

    Ok(Json(measured))
}
