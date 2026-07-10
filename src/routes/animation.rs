use std::borrow::Cow;

use axum::{
    extract::State,
    http::header::CONTENT_TYPE,
    response::{IntoResponse, Response},
};
use takumi::{
    prelude::{
        AnimatedGifOptions, AnimatedPngOptions, AnimatedWebpOptions,
        AnimationFormat as TakumiAnimationFormat, AnimationFrame as TakumiAnimationFrame,
        ImageSource, RenderOptions, SequentialScene, StyleSheet, Viewport,
    },
    render as takumi_render, write_animated_gif, write_animated_png, write_animated_webp,
    write_animation,
};
use tokio::task::spawn_blocking;

use crate::{
    dto::animation::{AnimationFormat, AnimationRequest},
    error::{ApiError, ApiResult},
    extractors::json_or_form::JsonOrMultipart,
    resources::{dithering, font_families, lang, resolve_fonts, resolve_images, resolve_node},
    state::SharedState,
};

pub async fn render_animation(
    State(state): State<SharedState>,
    payload: JsonOrMultipart<AnimationRequest>,
) -> ApiResult<Response> {
    let request = payload.data;
    let uploaded_files = payload.files;

    let has_frames = !request.frames.is_empty();
    let has_scenes = !request.scenes.is_empty();

    if !has_frames && !has_scenes {
        return Err(ApiError::BadRequest(
            "either frames or scenes must be provided".into(),
        ));
    }

    if has_frames && has_scenes {
        return Err(ApiError::BadRequest(
            "frames and scenes are mutually exclusive".into(),
        ));
    }

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
    let mut images = state.images.read().await.clone();

    for resource in request.fetched_resources {
        let data =
            base64::Engine::decode(&base64::engine::general_purpose::STANDARD, resource.data)
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

    let viewport = Viewport::new((request.options.width, request.options.height))
        .with_device_pixel_ratio(request.options.device_pixel_ratio);

    let mut stylesheet = if !request.stylesheets.is_empty() {
        StyleSheet::parse_list(&request.stylesheets)
            .map_err(|e| ApiError::BadRequest(format!("Invalid stylesheet: {e}")))?
    } else {
        StyleSheet::default()
    };
    if let Some(keyframes) = request.keyframes {
        stylesheet.extend_keyframes(keyframes);
    }

    let content_type = match request.options.format {
        AnimationFormat::Webp => "image/webp",
        AnimationFormat::Apng => "image/png",
        AnimationFormat::Gif => "image/gif",
    };
    let loop_count = request.options.loop_count;
    let quality = request.options.quality;

    if has_scenes {
        let fps = request
            .options
            .fps
            .ok_or_else(|| ApiError::BadRequest("fps is required when using scenes".into()))?;

        let mut scenes = Vec::with_capacity(request.scenes.len());
        for scene in request.scenes {
            let node = resolve_node(scene.node, scene.html)?;
            images = resolve_images(
                &state,
                &node,
                images,
                request.options.fetch_images,
                timeout_ms,
                request.options.fetch_cache,
            )
            .await?;
            let options = RenderOptions::builder()
                .viewport(viewport)
                .node(node)
                .fonts(&fonts)
                .images(images.clone())
                .draw_debug_border(request.options.draw_debug_border)
                .stylesheet(stylesheet.clone())
                .dithering(dithering(request.options.dithering))
                .font_families(font_families.clone())
                .lang(lang)
                .build();
            scenes.push(
                SequentialScene::builder()
                    .options(options)
                    .duration_ms(scene.duration_ms)
                    .build(),
            );
        }

        let format = match request.options.format {
            AnimationFormat::Webp => {
                let mut options = AnimatedWebpOptions::default();
                options.loop_count = loop_count;
                if let Some(quality) = quality {
                    options.quality = quality;
                    options.lossless = false;
                }
                TakumiAnimationFormat::WebP(options)
            }
            AnimationFormat::Apng => {
                let mut options = AnimatedPngOptions::default();
                options.loop_count = loop_count;
                TakumiAnimationFormat::Apng(options)
            }
            AnimationFormat::Gif => {
                let mut options = AnimatedGifOptions::default();
                options.loop_count = loop_count.or(Some(0));
                TakumiAnimationFormat::Gif(options)
            }
        };
        let mut buffer = Vec::new();
        write_animation(&scenes, fps, format, &mut buffer)?;
        return Ok(([(CONTENT_TYPE, content_type)], buffer).into_response());
    }

    let mut frames = Vec::with_capacity(request.frames.len());
    for frame in request.frames {
        let node = resolve_node(frame.node, frame.html)?;
        images = resolve_images(
            &state,
            &node,
            images,
            request.options.fetch_images,
            timeout_ms,
            request.options.fetch_cache,
        )
        .await?;
        let options = RenderOptions::builder()
            .viewport(viewport)
            .node(node)
            .fonts(&fonts)
            .images(images.clone())
            .draw_debug_border(request.options.draw_debug_border)
            .stylesheet(stylesheet.clone())
            .dithering(dithering(request.options.dithering))
            .font_families(font_families.clone())
            .lang(lang)
            .build();
        frames.push(TakumiAnimationFrame::new(
            takumi_render(options)?,
            frame.duration_ms,
        ));
    }

    let format = request.options.format;

    let buffer = spawn_blocking(move || -> ApiResult<Vec<u8>> {
        let mut buf = Vec::new();

        match format {
            AnimationFormat::Webp => {
                let mut opts = AnimatedWebpOptions::default();
                if let Some(lc) = loop_count {
                    opts.loop_count = Some(lc);
                }
                if let Some(q) = quality {
                    opts.quality = q;
                }
                write_animated_webp(Cow::Borrowed(&frames), &mut buf, opts)?;
            }
            AnimationFormat::Apng => {
                let mut opts = AnimatedPngOptions::default();
                if let Some(lc) = loop_count {
                    opts.loop_count = Some(lc);
                }
                write_animated_png(&frames, &mut buf, opts)?;
            }
            AnimationFormat::Gif => {
                let mut opts = AnimatedGifOptions::default();
                opts.loop_count = loop_count.or(Some(0));
                write_animated_gif(Cow::Borrowed(&frames), &mut buf, opts)?;
            }
        }

        Ok(buf)
    })
    .await
    .map_err(|e| ApiError::Internal(format!("Animation render task panicked: {e}")))??;

    Ok(([(CONTENT_TYPE, content_type)], buffer).into_response())
}
