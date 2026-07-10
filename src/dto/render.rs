use std::collections::HashMap;

use axum::body::Bytes;
use serde::Deserialize;
use takumi::prelude::{KeyframesRule, Node};

use crate::dto::resources::{Dithering, FontInput, default_true};
use crate::{error::ApiError, extractors::json_or_form::MultipartParseable};

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum OutputFormat {
    #[default]
    Png,
    Jpeg,
    Webp,
    Ico,
    Svg,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderOptions {
    pub width: Option<u32>,
    pub height: Option<u32>,
    #[serde(default)]
    pub format: OutputFormat,
    pub quality: Option<u8>,
    #[serde(default)]
    pub lossless: bool,
    #[serde(default = "default_dpr")]
    pub device_pixel_ratio: f32,
    #[serde(default)]
    pub draw_debug_border: bool,
    #[serde(default)]
    pub time_ms: u64,
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

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            width: None,
            height: None,
            format: OutputFormat::default(),
            quality: None,
            lossless: false,
            device_pixel_ratio: default_dpr(),
            draw_debug_border: false,
            time_ms: 0,
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

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FetchedResource {
    pub src: String,
    pub data: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderRequest {
    pub node: Option<Node>,
    pub html: Option<String>,
    #[serde(default)]
    pub options: RenderOptions,
    #[serde(default)]
    pub fetched_resources: Vec<FetchedResource>,
    #[serde(
        default,
        deserialize_with = "takumi::unstable::base::keyframes::deserialize_optional_keyframes"
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
            html,
            options,
            fetched_resources: Vec::new(),
            keyframes,
            stylesheets,
        })
    }
}
