mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode, header::CONTENT_TYPE},
};
use http_body_util::BodyExt;
use tower::ServiceExt;

#[tokio::test]
async fn test_extract_image_urls() {
    let app = common::create_test_app();

    let body = r#"{
        "node": {
            "type": "container",
            "children": [{
                "type": "image",
                "src": "https://example.com/image.png"
            }]
        }
    }"#;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/extract-urls")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    assert!(json["urls"].is_array());
    let urls = json["urls"].as_array().unwrap();
    assert!(urls.iter().any(|u| u == "https://example.com/image.png"));
}

#[tokio::test]
async fn test_extract_no_urls() {
    let app = common::create_test_app();

    let body = r#"{
        "node": {
            "type": "container",
            "tw": "w-[100] h-[100] bg-blue-500"
        }
    }"#;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/extract-urls")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    assert!(json["urls"].is_array());
    assert!(json["urls"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_extract_urls_invalid_node() {
    let app = common::create_test_app();

    let body = r#"{
        "node": {
            "type": "invalid"
        }
    }"#;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/extract-urls")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    // Invalid node type returns 422 (Unprocessable Entity) or 400 (Bad Request)
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY
    );
}
