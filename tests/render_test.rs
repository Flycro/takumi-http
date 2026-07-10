mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode, header::CONTENT_TYPE},
};
use base64::Engine;
use image::{ImageFormat, RgbaImage};
use std::io::Cursor;
use tower::ServiceExt;

#[tokio::test]
async fn test_render_simple_container() {
    let app = common::create_test_app();

    let body = r#"{
        "node": {
            "type": "container",
            "style": {
                "width": "100px",
                "height": "100px",
                "backgroundColor": "blue"
            }
        },
        "options": {
            "format": "png",
            "width": 100,
            "height": 100
        }
    }"#;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/render")
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
async fn test_render_with_tailwind() {
    let app = common::create_test_app();

    let body = r#"{
        "node": {
            "type": "container",
            "tw": "w-[200] h-[200] bg-red-500"
        },
        "options": {
            "format": "webp"
        }
    }"#;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/render")
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
async fn test_render_jpeg_format() {
    let app = common::create_test_app();

    let body = r#"{
        "node": {
            "type": "container",
            "tw": "w-[100] h-[100] bg-green-500"
        },
        "options": {
            "format": "jpeg",
            "quality": 80
        }
    }"#;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/render")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers().get(CONTENT_TYPE).unwrap(), "image/jpeg");
}

#[tokio::test]
async fn test_render_lossless_webp_format() {
    let app = common::create_test_app();
    let body = r#"{
        "node": {"type": "container", "tw": "w-[32] h-[32] bg-blue-500"},
        "options": {"format": "webp", "lossless": true}
    }"#;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/render")
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
async fn test_render_ico_format() {
    let app = common::create_test_app();
    let body = r#"{
        "node": {"type": "container", "tw": "w-[32] h-[32] bg-blue-500"},
        "options": {"format": "ico", "width": 32, "height": 32}
    }"#;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/render")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get(CONTENT_TYPE).unwrap(),
        "image/x-icon"
    );
}

#[tokio::test]
async fn test_render_html_to_svg() {
    let app = common::create_test_app();
    let body = r#"{
        "html": "<div tw='w-[120px] h-[60px] bg-blue-500'><span>Hello</span></div>",
        "options": {
            "format": "svg",
            "width": 120,
            "height": 60,
            "fontFamilies": ["Geist"],
            "lang": "en-US"
        }
    }"#;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/render")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get(CONTENT_TYPE).unwrap(),
        "image/svg+xml"
    );
    let bytes = http_body_util::BodyExt::collect(response.into_body())
        .await
        .unwrap()
        .to_bytes();
    assert!(bytes.starts_with(b"<svg"));
}

#[tokio::test]
async fn test_render_with_dithering() {
    let app = common::create_test_app();
    let body = r#"{
        "node": {"type": "container", "tw": "w-[32] h-[32] bg-gradient-to-r from-black to-white"},
        "options": {"format": "png", "dithering": "orderedBayer"}
    }"#;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/render")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_render_fetches_remote_image() {
    let png = create_test_png();
    let resource_app = axum::Router::new().route(
        "/image.png",
        axum::routing::get(move || {
            let png = png.clone();
            async move { ([(CONTENT_TYPE, "image/png")], png) }
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, resource_app).await.unwrap() });

    let app = common::create_test_app();
    let body = serde_json::json!({
        "node": {
            "type": "image",
            "src": format!("http://{address}/image.png"),
            "tw": "w-[10px] h-[10px]"
        },
        "options": {
            "format": "png",
            "width": 10,
            "height": 10,
            "fetchImages": true,
            "fetchCache": false,
            "fetchTimeoutMs": 2000
        }
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/render")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_render_with_request_font() {
    let app = common::create_test_app();
    let font = include_bytes!("../assets/fonts/GeistMono[wght].woff2");
    let body = serde_json::json!({
        "node": {
            "type": "text",
            "text": "Request font",
            "tw": "text-2xl"
        },
        "options": {
            "format": "png",
            "width": 240,
            "height": 80,
            "fonts": [{
                "data": base64::engine::general_purpose::STANDARD.encode(font)
            }],
            "fontFamilies": ["Geist Mono"],
            "lang": "en"
        }
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/render")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_render_invalid_node_type() {
    let app = common::create_test_app();

    let body = r#"{
        "node": {
            "type": "invalid_type"
        }
    }"#;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/render")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_render_malformed_json() {
    let app = common::create_test_app();

    let body = r#"{ invalid json }"#;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/render")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_render_with_text_node() {
    let app = common::create_test_app();

    let body = r#"{
        "node": {
            "type": "container",
            "tw": "w-[300] h-[100] bg-white",
            "children": [{
                "type": "text",
                "text": "Hello World",
                "tw": "text-black text-2xl"
            }]
        },
        "options": {
            "format": "png"
        }
    }"#;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/render")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_render_complex_layout() {
    let app = common::create_test_app();

    let body = r#"{
        "node": {
            "type": "container",
            "tw": "w-full h-full flex justify-center bg-black items-center",
            "style": {
                "backgroundImage": "radial-gradient(circle at 25px 25px, lightgray 2%, transparent 0%), radial-gradient(circle at 75px 75px, lightgray 2%, transparent 0%)",
                "backgroundSize": "100px 100px"
            },
            "children": [{
                "type": "container",
                "tw": "flex flex-col justify-center items-center",
                "children": [
                    {
                        "type": "container",
                        "tw": "flex flex-row gap-3",
                        "children": [
                            { "type": "text", "text": "Welcome to", "tw": "text-white font-semibold text-6xl" },
                            { "type": "text", "text": "Takumi", "tw": "text-[#ff3535] font-semibold text-6xl" },
                            { "type": "text", "text": "Playground 👋", "tw": "text-white font-semibold text-6xl" }
                        ]
                    },
                    {
                        "type": "text",
                        "text": "You can try out and experiment with Takumi here.",
                        "tw": "text-white opacity-75 text-4xl mt-4",
                        "style": { "fontFamily": "Geist Mono" }
                    }
                ]
            }]
        },
        "options": {
            "format": "png",
            "width": 1200,
            "height": 630
        }
    }"#;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/render")
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
async fn test_render_with_device_pixel_ratio() {
    let app = common::create_test_app();

    let body = r#"{
        "node": {
            "type": "container",
            "tw": "w-[100] h-[100] bg-blue-500"
        },
        "options": {
            "format": "png",
            "width": 100,
            "height": 100,
            "devicePixelRatio": 2.0
        }
    }"#;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/render")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

fn create_test_png() -> Vec<u8> {
    let img = RgbaImage::from_pixel(50, 50, image::Rgba([255, 0, 0, 255]));
    let mut bytes = Vec::new();
    img.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
        .unwrap();
    bytes
}

#[tokio::test]
async fn test_render_multipart_with_image() {
    let app = common::create_test_app();

    let png_data = create_test_png();
    let boundary = "----TestBoundary1234567890";

    let node_json = r#"{"type":"container","tw":"w-[200] h-[200] bg-gray-100 flex items-center justify-center","children":[{"type":"image","src":"test-logo","tw":"w-[100] h-[100] object-cover"}]}"#;
    let options_json = r#"{"format":"png","width":200,"height":200}"#;

    let mut body = Vec::new();

    // Add node field
    body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
    body.extend_from_slice(b"Content-Disposition: form-data; name=\"node\"\r\n\r\n");
    body.extend_from_slice(node_json.as_bytes());
    body.extend_from_slice(b"\r\n");

    // Add options field
    body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
    body.extend_from_slice(b"Content-Disposition: form-data; name=\"options\"\r\n\r\n");
    body.extend_from_slice(options_json.as_bytes());
    body.extend_from_slice(b"\r\n");

    // Add image file
    body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
    body.extend_from_slice(
        b"Content-Disposition: form-data; name=\"resource_test-logo\"; filename=\"test.png\"\r\n",
    );
    body.extend_from_slice(b"Content-Type: image/png\r\n\r\n");
    body.extend_from_slice(&png_data);
    body.extend_from_slice(b"\r\n");

    // End boundary
    body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/render")
                .header(
                    CONTENT_TYPE,
                    format!("multipart/form-data; boundary={boundary}"),
                )
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers().get(CONTENT_TYPE).unwrap(), "image/png");
}
