use std::collections::HashMap;

use axum::body::Bytes;
use serde::Deserialize;
use takumi::layout::node::NodeKind;

use crate::{error::ApiError, extractors::json_or_form::MultipartParseable};

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum AnimationFormat {
    #[default]
    Webp,
    Apng,
}


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimationFrame {
    pub node: NodeKind,
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
}

fn default_dpr() -> f32 {
    1.0
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimationRequest {
    pub frames: Vec<AnimationFrame>,
    #[serde(default)]
    pub options: AnimationOptions,
}

impl MultipartParseable for AnimationRequest {
    fn from_multipart_fields(
        fields: HashMap<String, String>,
        _files: &[(String, Bytes)],
    ) -> Result<Self, ApiError> {
        let frames_json = fields
            .get("frames")
            .ok_or_else(|| ApiError::BadRequest("Missing 'frames' field".into()))?;

        let frames: Vec<AnimationFrame> =
            serde_json::from_str(frames_json).map_err(ApiError::JsonError)?;

        let options: AnimationOptions = if let Some(options_json) = fields.get("options") {
            serde_json::from_str(options_json).map_err(ApiError::JsonError)?
        } else {
            AnimationOptions::default()
        };

        Ok(AnimationRequest { frames, options })
    }
}
