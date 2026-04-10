use std::collections::HashMap;

use axum::body::Bytes;
use serde::Deserialize;
use takumi::layout::node::Node;
use takumi::layout::style::KeyframesRule;

use crate::{error::ApiError, extractors::json_or_form::MultipartParseable};

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum OutputFormat {
    #[default]
    Png,
    Jpeg,
    Webp,
}


#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RenderOptions {
    pub width: Option<u32>,
    pub height: Option<u32>,
    #[serde(default)]
    pub format: OutputFormat,
    pub quality: Option<u8>,
    #[serde(default = "default_dpr")]
    pub device_pixel_ratio: f32,
    #[serde(default)]
    pub draw_debug_border: bool,
    #[serde(default)]
    pub time_ms: u64,
}

fn default_dpr() -> f32 {
    1.0
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FetchedResource {
    pub src: String,
    pub data: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderRequest {
    pub node: Node,
    #[serde(default)]
    pub options: RenderOptions,
    #[serde(default)]
    pub fetched_resources: Vec<FetchedResource>,
    #[serde(
        default,
        deserialize_with = "takumi::keyframes::deserialize_optional_keyframes"
    )]
    pub keyframes: Option<Vec<KeyframesRule>>,
    #[serde(default)]
    pub stylesheets: Vec<String>,
}

impl MultipartParseable for RenderRequest {
    fn from_multipart_fields(
        fields: HashMap<String, String>,
        _files: &[(String, Bytes)],
    ) -> Result<Self, ApiError> {
        let node_json = fields
            .get("node")
            .ok_or_else(|| ApiError::BadRequest("Missing 'node' field".into()))?;

        let node: Node = serde_json::from_str(node_json).map_err(ApiError::JsonError)?;

        let options: RenderOptions = if let Some(options_json) = fields.get("options") {
            serde_json::from_str(options_json).map_err(ApiError::JsonError)?
        } else {
            RenderOptions::default()
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

        Ok(RenderRequest {
            node,
            options,
            fetched_resources: Vec::new(),
            keyframes,
            stylesheets,
        })
    }
}
