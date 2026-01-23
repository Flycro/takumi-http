use std::sync::Arc;
use takumi::GlobalContext;
use takumi_http::{AppState, Config, create_router};

const GEIST_FONT: &[u8] = include_bytes!("../../assets/fonts/Geist[wght].woff2");
const GEIST_MONO_FONT: &[u8] = include_bytes!("../../assets/fonts/GeistMono[wght].woff2");
const TWEMOJI_FONT: &[u8] = include_bytes!("../../assets/fonts/TwemojiMozilla-colr.woff2");

pub fn create_test_app() -> axum::Router {
    let config = Config {
        port: 3000,
        font_dir: None,
        load_default_fonts: true,
        body_limit: 50_000_000,
        enable_cache: true,
        log_level: "info".to_string(),
    };

    let mut context = GlobalContext::default();

    // Load fonts for text rendering
    context
        .font_context
        .load_and_store(GEIST_FONT, None, None)
        .unwrap();
    context
        .font_context
        .load_and_store(GEIST_MONO_FONT, None, None)
        .unwrap();
    context
        .font_context
        .load_and_store(TWEMOJI_FONT, None, None)
        .unwrap();

    let state = Arc::new(AppState::new(config, context, 3));

    create_router(state)
}
