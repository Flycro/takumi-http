use std::collections::HashMap;

use axum::body::Bytes;
use serde::Deserialize;
use takumi::prelude::Node;

use crate::dto::{
    render::FetchedResource,
    resources::{FontInput, default_true},
};
use crate::{error::ApiError, extractors::json_or_form::MultipartParseable};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MeasureOptions {
    pub width: Option<u32>,
    pub height: Option<u32>,
    #[serde(default = "default_dpr")]
    pub device_pixel_ratio: f32,
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

impl Default for MeasureOptions {
    fn default() -> Self {
        Self {
            width: None,
            height: None,
            device_pixel_ratio: default_dpr(),
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
pub struct MeasureRequest {
    pub node: Option<Node>,
    pub html: Option<String>,
    #[serde(default)]
    pub fetched_resources: Vec<FetchedResource>,
    #[serde(default)]
    pub options: MeasureOptions,
}

impl MultipartParseable for MeasureRequest {
    fn from_multipart_fields(
        fields: HashMap<String, String>,
        _files: &[(String, Bytes)],
    ) -> Result<Self, ApiError> {
        let node = fields
            .get("node")
            .map(|value| serde_json::from_str(value).map_err(ApiError::JsonError))
            .transpose()?;
        let html = fields.get("html").cloned();
        if node.is_none() && html.is_none() {
            return Err(ApiError::BadRequest(
                "Missing 'node' or 'html' field".into(),
            ));
        }

        let options: MeasureOptions = if let Some(options_json) = fields.get("options") {
            serde_json::from_str(options_json).map_err(ApiError::JsonError)?
        } else {
            MeasureOptions::default()
        };

        Ok(MeasureRequest {
            node,
            html,
            fetched_resources: Vec::new(),
            options,
        })
    }
}
