use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum FontInput {
    Url(String),
    Data { data: String },
}

#[derive(Debug, Deserialize, Clone, Copy, Default)]
#[serde(rename_all = "camelCase")]
pub enum Dithering {
    #[default]
    None,
    OrderedBayer,
    FloydSteinberg,
}

pub fn default_true() -> bool {
    true
}
