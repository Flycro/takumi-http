use std::collections::HashMap;

use axum::body::Bytes;
use serde::Deserialize;
use takumi::layout::node::Node;
use takumi::layout::style::KeyframesRule;

use crate::{error::ApiError, extractors::json_or_form::MultipartParseable};

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
    pub node: Node,
    pub duration_ms: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimationScene {
    pub node: Node,
    pub duration_ms: u32,
}

#[derive(Debug, Deserialize, Default)]
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
}

fn default_dpr() -> f32 {
    1.0
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
        deserialize_with = "takumi::keyframes::deserialize_optional_keyframes"
    )]
    pub keyframes: Option<Vec<KeyframesRule>>,
    #[serde(default)]
    pub stylesheets: Vec<String>,
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
        })
    }
}
