use std::collections::HashMap;

use axum::body::Bytes;
use serde::Deserialize;
use takumi::layout::node::NodeKind;

use crate::{error::ApiError, extractors::json_or_form::MultipartParseable};

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MeasureOptions {
    pub width: Option<u32>,
    pub height: Option<u32>,
    #[serde(default = "default_dpr")]
    pub device_pixel_ratio: f32,
}

fn default_dpr() -> f32 {
    1.0
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MeasureRequest {
    pub node: NodeKind,
    #[serde(default)]
    pub options: MeasureOptions,
}

impl MultipartParseable for MeasureRequest {
    fn from_multipart_fields(
        fields: HashMap<String, String>,
        _files: &[(String, Bytes)],
    ) -> Result<Self, ApiError> {
        let node_json = fields
            .get("node")
            .ok_or_else(|| ApiError::BadRequest("Missing 'node' field".into()))?;

        let node: NodeKind = serde_json::from_str(node_json).map_err(ApiError::JsonError)?;

        let options: MeasureOptions = if let Some(options_json) = fields.get("options") {
            serde_json::from_str(options_json).map_err(ApiError::JsonError)?
        } else {
            MeasureOptions::default()
        };

        Ok(MeasureRequest { node, options })
    }
}
