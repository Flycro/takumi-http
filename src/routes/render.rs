use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::State,
    http::{HeaderMap, header::CONTENT_TYPE},
    response::{IntoResponse, Response},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use takumi::{
    prelude::{
        ImageSource, OutputFormat as TakumiOutputFormat, Quality, RenderOptions, StyleSheet,
        SvgOptions, Viewport,
    },
    render as takumi_render, render_svg, write_image,
};
use tokio::task::spawn_blocking;

use crate::{
    dto::render::{OutputFormat, RenderRequest},
    error::{ApiError, ApiResult},
    extractors::json_or_form::JsonOrMultipart,
    resources::{dithering, font_families, lang, resolve_fonts, resolve_images, resolve_node},
    state::SharedState,
};

fn convert_format(
    format: &OutputFormat,
    quality: Option<u8>,
    lossless: bool,
) -> TakumiOutputFormat {
    let quality = Quality::new(quality.unwrap_or(75));
    match format {
        OutputFormat::Png => TakumiOutputFormat::Png,
        OutputFormat::Jpeg => TakumiOutputFormat::Jpeg { quality },
        OutputFormat::Webp if lossless => TakumiOutputFormat::WebPLossless,
        OutputFormat::Webp => TakumiOutputFormat::WebP { quality },
        OutputFormat::Ico => TakumiOutputFormat::Ico,
        OutputFormat::Svg => unreachable!("SVG uses the vector backend"),
    }
}

pub async fn render(
    State(state): State<SharedState>,
    _headers: HeaderMap,
    payload: JsonOrMultipart<RenderRequest>,
) -> ApiResult<Response> {
    let request = payload.data;
    let uploaded_files = payload.files;

    let is_svg = matches!(request.options.format, OutputFormat::Svg);
    let format = (!is_svg).then(|| {
        convert_format(
            &request.options.format,
            request.options.quality,
            request.options.lossless,
        )
    });
    let time_ms = request.options.time_ms;
    let timeout_ms = request.options.fetch_timeout_ms.unwrap_or(10_000);
    let fonts = resolve_fonts(
        &state,
        request.options.fonts,
        timeout_ms,
        request.options.fetch_cache,
    )
    .await?;
    let font_families = font_families(request.options.font_families);
    let lang = lang(request.options.lang)?;
    let node = resolve_node(request.node, request.html)?;

    let mut fetched_resources: HashMap<Arc<str>, ImageSource> = HashMap::new();

    fetched_resources.extend(state.images.read().await.clone());

    // Add resources from JSON (base64 encoded)
    for resource in request.fetched_resources {
        let data = STANDARD.decode(&resource.data).map_err(|e| {
            ApiError::BadRequest(format!("Invalid base64 in fetchedResources: {e}"))
        })?;

        let image_source = ImageSource::from_bytes(&data)
            .map_err(|e| ApiError::ImageDecodeError(format!("{e:?}")))?;

        fetched_resources.insert(Arc::from(resource.src), image_source);
    }

    // Add resources from multipart file uploads
    for (name, data) in uploaded_files {
        let image_source = ImageSource::from_bytes(&data)
            .map_err(|e| ApiError::ImageDecodeError(format!("Failed to decode {name}: {e:?}")))?;

        fetched_resources.insert(Arc::from(name), image_source);
    }

    let fetched_resources = resolve_images(
        &state,
        &node,
        fetched_resources,
        request.options.fetch_images,
        timeout_ms,
        request.options.fetch_cache,
    )
    .await?;

    // Build stylesheet from CSS strings and/or structured keyframes
    let mut stylesheet = if !request.stylesheets.is_empty() {
        StyleSheet::parse_list(&request.stylesheets)
            .map_err(|e| ApiError::BadRequest(format!("Invalid stylesheet: {e}")))?
    } else {
        StyleSheet::default()
    };

    if let Some(keyframes) = request.keyframes {
        stylesheet.extend_keyframes(keyframes);
    }

    let viewport = Viewport::new((request.options.width, request.options.height))
        .with_device_pixel_ratio(request.options.device_pixel_ratio);

    let draw_debug_border = request.options.draw_debug_border;

    if is_svg {
        let svg = render_svg(
            SvgOptions::builder()
                .viewport(viewport)
                .node(node)
                .fonts(&fonts)
                .images(fetched_resources)
                .stylesheet(stylesheet)
                .time_ms(time_ms)
                .font_families(font_families)
                .lang(lang)
                .build(),
        )?;
        return Ok(([(CONTENT_TYPE, "image/svg+xml")], svg).into_response());
    }

    let options = RenderOptions::builder()
        .viewport(viewport)
        .node(node)
        .fonts(&fonts)
        .draw_debug_border(draw_debug_border)
        .images(fetched_resources)
        .stylesheet(stylesheet)
        .time_ms(time_ms)
        .dithering(dithering(request.options.dithering))
        .font_families(font_families)
        .lang(lang)
        .build();

    let image = takumi_render(options)?;

    let buffer = spawn_blocking(move || -> ApiResult<Vec<u8>> {
        let mut buf = Vec::new();
        write_image(&image, &mut buf, format.expect("raster format checked"))?;
        Ok(buf)
    })
    .await
    .map_err(|e| ApiError::Internal(format!("Render task panicked: {e}")))??;

    let content_type = format.expect("raster format checked").content_type();
    Ok(([(CONTENT_TYPE, content_type)], buffer).into_response())
}
