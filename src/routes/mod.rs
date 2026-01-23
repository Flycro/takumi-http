use axum::{
    Router,
    routing::{delete, get, post},
};

use crate::state::SharedState;

pub mod animation;
pub mod extract_urls;
pub mod health;
pub mod images;
pub mod measure;
pub mod render;

pub fn create_router(state: SharedState) -> Router {
    Router::new()
        .route("/health", get(health::health))
        .route("/render", post(render::render))
        .route("/render/animation", post(animation::render_animation))
        .route("/measure", post(measure::measure))
        .route("/images", post(images::add_image))
        .route("/images", delete(images::clear_images))
        .route("/extract-urls", post(extract_urls::extract_urls))
        .with_state(state)
}
