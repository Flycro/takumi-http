mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode, header::CONTENT_TYPE},
};
use http_body_util::BodyExt;
use tower::ServiceExt;

#[tokio::test]
async fn test_measure_fixed_size() {
    let app = common::create_test_app();

    let body = r#"{
        "node": {
            "type": "container",
            "style": {
                "width": "200px",
                "height": "100px"
            }
        },
        "options": {
            "width": 200,
            "height": 100
        }
    }"#;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/measure")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(json["width"], 200.0);
    assert_eq!(json["height"], 100.0);
}

#[tokio::test]
async fn test_measure_with_children() {
    let app = common::create_test_app();

    let body = r#"{
        "node": {
            "type": "container",
            "tw": "w-[300] h-[200]",
            "children": [{
                "type": "container",
                "tw": "w-[100] h-[50]"
            }]
        }
    }"#;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/measure")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    assert!(json["children"].is_array());
}

#[tokio::test]
async fn test_measure_tailwind_arbitrary_values() {
    let app = common::create_test_app();

    // Provide a larger viewport so the container can expand to its Tailwind-defined size
    let body = r#"{
        "node": {
            "type": "container",
            "tw": "w-[200px] h-[100px]"
        },
        "options": {
            "width": 1000,
            "height": 1000
        }
    }"#;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/measure")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(json["width"], 200.0);
    assert_eq!(json["height"], 100.0);
}

#[tokio::test]
async fn test_measure_invalid_node() {
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
                .uri("/measure")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
