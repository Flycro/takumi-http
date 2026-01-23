mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode, header::CONTENT_TYPE},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use http_body_util::BodyExt;
use image::{ImageFormat, RgbaImage};
use std::io::Cursor;
use tower::ServiceExt;

fn create_1x1_png_base64() -> String {
    let img = RgbaImage::from_pixel(1, 1, image::Rgba([255, 0, 0, 255]));
    let mut bytes = Vec::new();
    img.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
        .unwrap();
    STANDARD.encode(&bytes)
}

#[tokio::test]
async fn test_add_image_to_cache() {
    let app = common::create_test_app();

    let body = serde_json::json!({
        "src": "test-image",
        "data": create_1x1_png_base64()
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/images")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(json["src"], "test-image");
}

#[tokio::test]
async fn test_add_image_invalid_base64() {
    let app = common::create_test_app();

    let body = serde_json::json!({
        "src": "test-image",
        "data": "not-valid-base64!!!"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/images")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_add_image_invalid_image_data() {
    let app = common::create_test_app();

    let body = serde_json::json!({
        "src": "test-image",
        "data": STANDARD.encode(b"not an image")
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/images")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_clear_images() {
    let app = common::create_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/images")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    assert!(json["cleared_count"].is_number());
}

fn create_50x50_png_base64() -> String {
    let img = RgbaImage::from_pixel(50, 50, image::Rgba([0, 255, 0, 255])); // Green
    let mut bytes = Vec::new();
    img.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
        .unwrap();
    STANDARD.encode(&bytes)
}

#[tokio::test]
async fn test_cache_then_render() {
    let app = common::create_test_app();

    // Step 1: Add image to cache
    let add_body = serde_json::json!({
        "src": "cached-logo",
        "data": create_50x50_png_base64()
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/images")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(add_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    // Step 2: Render using the cached image
    let render_body = serde_json::json!({
        "node": {
            "type": "container",
            "tw": "w-[200] h-[200] bg-white flex items-center justify-center",
            "children": [{
                "type": "image",
                "src": "cached-logo",
                "tw": "w-[100] h-[100]"
            }]
        },
        "options": {
            "format": "png",
            "width": 200,
            "height": 200
        }
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/render")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(render_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers().get(CONTENT_TYPE).unwrap(), "image/png");
}
