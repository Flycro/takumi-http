use std::{collections::HashMap, sync::Arc, time::Duration};

use base64::{Engine, engine::general_purpose::STANDARD};
use takumi::{
    from_html,
    prelude::{
        DitheringAlgorithm, FontFamily, FontResource, Fonts, FromHtmlOptions, ImageSource, Lang,
        Node,
    },
};

use crate::{
    dto::resources::{Dithering, FontInput},
    error::{ApiError, ApiResult},
    state::SharedState,
};

pub fn resolve_node(node: Option<Node>, html: Option<String>) -> ApiResult<Node> {
    match (node, html) {
        (Some(node), None) => Ok(node),
        (None, Some(html)) => from_html(&html, FromHtmlOptions::default())
            .map_err(|error| ApiError::BadRequest(format!("Invalid HTML: {error}"))),
        (Some(_), Some(_)) => Err(ApiError::BadRequest(
            "'node' and 'html' are mutually exclusive".into(),
        )),
        (None, None) => Err(ApiError::BadRequest(
            "either 'node' or 'html' must be provided".into(),
        )),
    }
}

async fn fetch_bytes(
    state: &SharedState,
    url: &str,
    timeout_ms: u64,
    cache: bool,
) -> ApiResult<Arc<[u8]>> {
    if cache && let Some(bytes) = state.fetched_bytes.read().await.get(url).cloned() {
        return Ok(bytes);
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(timeout_ms))
        .build()
        .map_err(|error| ApiError::BadRequest(format!("Invalid fetch options: {error}")))?;
    let response = client
        .get(url)
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(|error| ApiError::BadRequest(format!("Failed to fetch {url}: {error}")))?;
    let bytes: Arc<[u8]> = response
        .bytes()
        .await
        .map_err(|error| ApiError::BadRequest(format!("Failed to read {url}: {error}")))?
        .to_vec()
        .into();
    if bytes.len() > state.config.body_limit {
        return Err(ApiError::BadRequest(format!(
            "Fetched resource exceeds {} bytes",
            state.config.body_limit
        )));
    }
    if cache {
        state
            .fetched_bytes
            .write()
            .await
            .insert(url.to_owned(), bytes.clone());
    }
    Ok(bytes)
}

pub async fn resolve_fonts(
    state: &SharedState,
    inputs: Vec<FontInput>,
    timeout_ms: u64,
    cache: bool,
) -> ApiResult<Fonts> {
    let mut fonts = state.fonts.read().await.clone();
    for input in inputs {
        let bytes = match input {
            FontInput::Url(url) => fetch_bytes(state, &url, timeout_ms, cache).await?.to_vec(),
            FontInput::Data { data } => STANDARD
                .decode(data)
                .map_err(|error| ApiError::BadRequest(format!("Invalid font base64: {error}")))?,
        };
        fonts
            .register(FontResource::new(bytes))
            .map_err(takumi::prelude::Error::from)?;
    }
    Ok(fonts)
}

pub async fn resolve_images(
    state: &SharedState,
    node: &Node,
    mut images: HashMap<Arc<str>, ImageSource>,
    fetch: bool,
    timeout_ms: u64,
    cache: bool,
) -> ApiResult<HashMap<Arc<str>, ImageSource>> {
    if fetch {
        let urls: Vec<String> = node.image_urls().map(ToOwned::to_owned).collect();
        for url in urls {
            if images.contains_key(url.as_str()) {
                continue;
            }
            let bytes = fetch_bytes(state, &url, timeout_ms, cache).await?;
            let image = ImageSource::from_bytes(&bytes)
                .map_err(|error| ApiError::ImageDecodeError(format!("{error:?}")))?;
            images.insert(url.into(), image);
        }
    }
    Ok(images)
}

pub fn font_families(names: Option<Vec<String>>) -> Option<FontFamily> {
    names.map(FontFamily::from_names)
}

pub fn lang(value: Option<String>) -> ApiResult<Option<Lang>> {
    value
        .map(|value| {
            Lang::parse(&value)
                .map_err(|error| ApiError::BadRequest(format!("Invalid language tag: {error}")))
        })
        .transpose()
}

pub fn dithering(value: Dithering) -> DitheringAlgorithm {
    match value {
        Dithering::None => DitheringAlgorithm::None,
        Dithering::OrderedBayer => DitheringAlgorithm::OrderedBayer,
        Dithering::FloydSteinberg => DitheringAlgorithm::FloydSteinberg,
    }
}
