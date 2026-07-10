use std::collections::HashMap;

use axum::body::Bytes;
use serde::Deserialize;
use takumi::prelude::{KeyframesRule, Node};

use crate::{
    dto::{
        render::FetchedResource,
        resources::{Dithering, FontInput, default_true},
    },
    error::ApiError,
    extractors::json_or_form::MultipartParseable,
};

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum AnimationFormat {
    #[default]
    Webp,
    Apng,
    Gif,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimationFrame {
    pub node: Option<Node>,
    pub html: Option<String>,
    pub duration_ms: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimationScene {
    pub node: Option<Node>,
    pub html: Option<String>,
    pub duration_ms: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimationOptions {
    pub width: Option<u32>,
    pub height: Option<u32>,
    #[serde(default)]
    pub format: AnimationFormat,
    #[serde(default = "default_dpr")]
    pub device_pixel_ratio: f32,
    #[serde(default)]
    pub draw_debug_border: bool,
    pub fps: Option<u32>,
    pub quality: Option<u8>,
    pub loop_count: Option<u16>,
    #[serde(default)]
    pub dithering: Dithering,
    #[serde(default)]
    pub fonts: Vec<FontInput>,
    pub font_families: Option<Vec<String>>,
    pub lang: Option<String>,
    #[serde(default)]
    pub fetch_images: bool,
    pub fetch_timeout_ms: Option<u64>,
    #[serde(default = "default_true")]
    pub fetch_cache: bool,
}

fn default_dpr() -> f32 {
    1.0
}

impl Default for AnimationOptions {
    fn default() -> Self {
        Self {
            width: None,
            height: None,
            format: AnimationFormat::default(),
            device_pixel_ratio: default_dpr(),
            draw_debug_border: false,
            fps: None,
            quality: None,
            loop_count: None,
            dithering: Dithering::default(),
            fonts: Vec::new(),
            font_families: None,
            lang: None,
            fetch_images: false,
            fetch_timeout_ms: None,
            fetch_cache: true,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimationRequest {
    #[serde(default)]
    pub frames: Vec<AnimationFrame>,
    #[serde(default)]
    pub scenes: Vec<AnimationScene>,
    #[serde(default)]
    pub options: AnimationOptions,
    #[serde(
        default,
        deserialize_with = "takumi::unstable::base::keyframes::deserialize_optional_keyframes"
    )]
    pub keyframes: Option<Vec<KeyframesRule>>,
    #[serde(default)]
    pub stylesheets: Vec<String>,
    #[serde(default)]
    pub fetched_resources: Vec<FetchedResource>,
}

impl MultipartParseable for AnimationRequest {
    fn from_multipart_fields(
        fields: HashMap<String, String>,
        _files: &[(String, Bytes)],
    ) -> Result<Self, ApiError> {
        let frames: Vec<AnimationFrame> = if let Some(frames_json) = fields.get("frames") {
            serde_json::from_str(frames_json).map_err(ApiError::JsonError)?
        } else {
            Vec::new()
        };

        let scenes: Vec<AnimationScene> = if let Some(scenes_json) = fields.get("scenes") {
            serde_json::from_str(scenes_json).map_err(ApiError::JsonError)?
        } else {
            Vec::new()
        };

        let options: AnimationOptions = if let Some(options_json) = fields.get("options") {
            serde_json::from_str(options_json).map_err(ApiError::JsonError)?
        } else {
            AnimationOptions::default()
        };

        let keyframes = if let Some(keyframes_json) = fields.get("keyframes") {
            Some(serde_json::from_str(keyframes_json).map_err(ApiError::JsonError)?)
        } else {
            None
        };

        let stylesheets: Vec<String> = if let Some(stylesheets_json) = fields.get("stylesheets") {
            serde_json::from_str(stylesheets_json).map_err(ApiError::JsonError)?
        } else {
            Vec::new()
        };

        Ok(AnimationRequest {
            frames,
            scenes,
            options,
            keyframes,
            stylesheets,
            fetched_resources: Vec::new(),
        })
    }
}
