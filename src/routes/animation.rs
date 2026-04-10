use std::borrow::Cow;

use axum::{
    extract::State,
    http::header::CONTENT_TYPE,
    response::{IntoResponse, Response},
};
use takumi::{
    layout::Viewport,
    layout::style::StyleSheet,
    rendering::{
        AnimatedGifOptions, AnimatedPngOptions, AnimatedWebpOptions,
        AnimationFrame as TakumiAnimationFrame, RenderOptions, SequentialScene,
        encode_animated_gif, encode_animated_png, encode_animated_webp,
        render as takumi_render, render_sequence_animation,
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

    let context = state.context.read().await;

    let viewport = Viewport::new((request.options.width, request.options.height))
        .with_device_pixel_ratio(request.options.device_pixel_ratio);

    let frames = if has_scenes {
        let fps = request
            .options
            .fps
            .ok_or_else(|| ApiError::BadRequest("fps is required when using scenes".into()))?;

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

        let scenes: Vec<SequentialScene<'_>> = request
            .scenes
            .into_iter()
            .map(|scene| {
                let options = RenderOptions::builder()
                    .viewport(viewport)
                    .node(scene.node)
                    .global(&context)
                    .draw_debug_border(request.options.draw_debug_border)
                    .stylesheet(stylesheet.clone())
                    .build();

                SequentialScene::builder()
                    .options(options)
                    .duration_ms(scene.duration_ms)
                    .build()
            })
            .collect();

        render_sequence_animation(&scenes, fps)?
    } else {
        let mut frames = Vec::with_capacity(request.frames.len());

        for frame in request.frames {
            let options = RenderOptions::builder()
                .viewport(viewport)
                .node(frame.node)
                .global(&context)
                .draw_debug_border(request.options.draw_debug_border)
                .build();

            let image = takumi_render(options)?;
            frames.push(TakumiAnimationFrame::new(image, frame.duration_ms));
        }

        frames
    };

    drop(context);

    let content_type = match request.options.format {
        AnimationFormat::Webp => "image/webp",
        AnimationFormat::Apng => "image/png",
        AnimationFormat::Gif => "image/gif",
    };

    let format = request.options.format;
    let loop_count = request.options.loop_count;
    let quality = request.options.quality;

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
                encode_animated_webp(Cow::Borrowed(&frames), &mut buf, opts)?;
            }
            AnimationFormat::Apng => {
                let mut opts = AnimatedPngOptions::default();
                if let Some(lc) = loop_count {
                    opts.loop_count = Some(lc);
                }
                encode_animated_png(&frames, &mut buf, opts)?;
            }
            AnimationFormat::Gif => {
                let mut opts = AnimatedGifOptions::default();
                opts.loop_count = loop_count.or(Some(0));
                encode_animated_gif(Cow::Borrowed(&frames), &mut buf, opts)?;
            }
        }

        Ok(buf)
    })
    .await
    .map_err(|e| ApiError::Internal(format!("Animation render task panicked: {e}")))??;

    Ok(([(CONTENT_TYPE, content_type)], buffer).into_response())
}
