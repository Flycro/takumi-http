mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode, header::CONTENT_TYPE},
};
use tower::ServiceExt;

#[tokio::test]
async fn test_render_animation_webp() {
    let app = common::create_test_app();

    let body = r#"{
        "frames": [
            {
                "node": {"type": "container", "tw": "w-[100] h-[100] bg-red-500"},
                "durationMs": 500
            },
            {
                "node": {"type": "container", "tw": "w-[100] h-[100] bg-blue-500"},
                "durationMs": 500
            }
        ],
        "options": {
            "format": "webp",
            "width": 100,
            "height": 100
        }
    }"#;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/render/animation")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers().get(CONTENT_TYPE).unwrap(), "image/webp");
}

#[tokio::test]
async fn test_render_animation_apng() {
    let app = common::create_test_app();

    let body = r#"{
        "frames": [
            {
                "node": {"type": "container", "tw": "w-[50] h-[50] bg-green-500"},
                "durationMs": 200
            },
            {
                "node": {"type": "container", "tw": "w-[50] h-[50] bg-yellow-500"},
                "durationMs": 200
            }
        ],
        "options": {
            "format": "apng"
        }
    }"#;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/render/animation")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers().get(CONTENT_TYPE).unwrap(), "image/png");
}

#[tokio::test]
async fn test_render_animation_empty_frames() {
    let app = common::create_test_app();

    let body = r#"{
        "frames": [],
        "options": {
            "format": "webp"
        }
    }"#;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/render/animation")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_render_animation_invalid_node() {
    let app = common::create_test_app();

    let body = r#"{
        "frames": [
            {
                "node": {"type": "invalid"},
                "durationMs": 100
            }
        ]
    }"#;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/render/animation")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_render_animation_gif() {
    let app = common::create_test_app();

    let body = r#"{
        "frames": [
            {
                "node": {"type": "container", "tw": "w-[50] h-[50] bg-red-500"},
                "durationMs": 200
            },
            {
                "node": {"type": "container", "tw": "w-[50] h-[50] bg-blue-500"},
                "durationMs": 200
            }
        ],
        "options": {
            "format": "gif",
            "width": 50,
            "height": 50
        }
    }"#;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/render/animation")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers().get(CONTENT_TYPE).unwrap(), "image/gif");
}

#[tokio::test]
async fn test_render_animation_scenes_with_keyframes() {
    let app = common::create_test_app();

    let body = r#"{
        "scenes": [
            {
                "node": {
                    "type": "container",
                    "tw": "w-full h-full items-center justify-center",
                    "children": [
                        {
                            "type": "container",
                            "tw": "w-10 h-10 bg-red-500 animate-[move_1s_ease-in-out_infinite_alternate]"
                        }
                    ]
                },
                "durationMs": 1000
            }
        ],
        "options": {
            "fps": 10,
            "format": "webp",
            "width": 100,
            "height": 100
        },
        "keyframes": {
            "move": {
                "from": {
                    "transform": "translateX(0)"
                },
                "to": {
                    "transform": "translateX(60px)"
                }
            }
        }
    }"#;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/render/animation")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers().get(CONTENT_TYPE).unwrap(), "image/webp");
}

#[tokio::test]
async fn test_render_animation_scenes_with_stylesheets() {
    let app = common::create_test_app();

    let body = r#"{
        "scenes": [
            {
                "node": {
                    "type": "container",
                    "tw": "w-full h-full",
                    "children": [
                        {
                            "type": "container",
                            "tw": "w-10 h-10 bg-blue-500 animate-[slide_500ms_linear]"
                        }
                    ]
                },
                "durationMs": 500
            }
        ],
        "options": {
            "fps": 10,
            "format": "gif",
            "width": 100,
            "height": 100,
            "loopCount": 0
        },
        "stylesheets": [
            "@keyframes slide { from { transform: translateX(0); } to { transform: translateX(50px); } }"
        ]
    }"#;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/render/animation")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers().get(CONTENT_TYPE).unwrap(), "image/gif");
}

#[tokio::test]
async fn test_render_animation_scenes_requires_fps() {
    let app = common::create_test_app();

    let body = r#"{
        "scenes": [
            {
                "node": {"type": "container", "tw": "w-[50] h-[50] bg-red-500"},
                "durationMs": 1000
            }
        ],
        "options": {
            "format": "webp",
            "width": 50,
            "height": 50
        }
    }"#;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/render/animation")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_render_animation_frames_and_scenes_exclusive() {
    let app = common::create_test_app();

    let body = r#"{
        "frames": [
            {
                "node": {"type": "container", "tw": "w-[50] h-[50] bg-red-500"},
                "durationMs": 100
            }
        ],
        "scenes": [
            {
                "node": {"type": "container", "tw": "w-[50] h-[50] bg-red-500"},
                "durationMs": 1000
            }
        ],
        "options": {
            "fps": 10,
            "format": "webp",
            "width": 50,
            "height": 50
        }
    }"#;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/render/animation")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
