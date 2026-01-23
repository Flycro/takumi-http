use std::collections::HashMap;

use axum::body::Bytes;
use serde::Deserialize;
use takumi::layout::node::NodeKind;

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
    pub node: NodeKind,
    #[serde(default)]
    pub options: RenderOptions,
    #[serde(default)]
    pub fetched_resources: Vec<FetchedResource>,
}

impl MultipartParseable for RenderRequest {
    fn from_multipart_fields(
        fields: HashMap<String, String>,
        _files: &[(String, Bytes)],
    ) -> Result<Self, ApiError> {
        let node_json = fields
            .get("node")
            .ok_or_else(|| ApiError::BadRequest("Missing 'node' field".into()))?;

        let node: NodeKind = serde_json::from_str(node_json).map_err(ApiError::JsonError)?;

        let options: RenderOptions = if let Some(options_json) = fields.get("options") {
            serde_json::from_str(options_json).map_err(ApiError::JsonError)?
        } else {
            RenderOptions::default()
        };

        Ok(RenderRequest {
            node,
            options,
            fetched_resources: Vec::new(),
        })
    }
}
