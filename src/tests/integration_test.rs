//! Integration tests that exercise the running Router in memory.
//! These tests are slightly more involved and use tower::ServiceExt.

use axum::body::Body;
use axum::http::{Request, Method};
use tower::ServiceExt;
use serde_json::Value;
use rust_api_hub::routes::create_router;
use rust_api_hub::tests::setup_logging;
use std::str::FromStr;

#[tokio::test]
async fn integration_create_list_flow() {
    setup_logging();
    let app = create_router();

    // 1. Create a task
    let payload = serde_json::json!({
        "title": "integration task",
        "description": "do this"
    });
    let req = Request::builder()
        .method(Method::POST)
        .uri("/tasks")
        .header("content-type", "application/json")
        .body(Body::from(payload.to_string()))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 201);

    // 2. List tasks
    let req2 = Request::builder().method(Method::GET).uri("/tasks").body(Body::empty()).unwrap();
    let resp2 = app.oneshot(req2).await.unwrap();
    assert_eq!(resp2.status(), 200);
    let bytes = hyper::body::to_bytes(resp2.into_body()).await.unwrap();
    let v: Value = serde_json::from_slice(&bytes).unwrap();
    assert!(v.is_array());
    assert!(v.as_array().unwrap().len() >= 1);
}

#[tokio::test]
async fn integration_update_and_delete_flow() {
    setup_logging();
    let app = create_router();

    // create
    let p = serde_json::json!({"title":"u","description":"d"});
    let req = Request::builder().method(Method::POST).uri("/tasks")
        .header("content-type","application/json")
        .body(Body::from(p.to_string())).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 201);
    let bytes = hyper::body::to_bytes(resp.into_body()).await.unwrap();
    let v: Value = serde_json::from_slice(&bytes).unwrap();
    let id = v.get("id").or_else(|| v.get("task").and_then(|t| t.get("id"))).expect("id").as_str().expect("str").to_string();

    // update
    let up = serde_json::json!({"title":"u2","completed":true});
    let uri = format!("/tasks/{}", id);
    let req2 = Request::builder().method(Method::PUT).uri(&uri)
        .header("content-type","application/json")
        .body(Body::from(up.to_string())).unwrap();
    let resp2 = app.clone().oneshot(req2).await.unwrap();
    assert_eq!(resp2.status(), 200);

    // delete
    let req3 = Request::builder().method(Method::DELETE).uri(&uri).body(Body::empty()).unwrap();
    let resp3 = app.oneshot(req3).await.unwrap();
    assert!(resp3.status() == 204 || resp3.status() == 200);
}

#[tokio::test]
async fn integration_health_info() {
    setup_logging();
    let app = create_router();
    let req = Request::builder().method(Method::GET).uri("/health").body(Body::empty()).unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let req2 = Request::builder().method(Method::GET).uri("/info").body(Body::empty()).unwrap();
    let resp2 = app.oneshot(req2).await.unwrap();
    assert_eq!(resp2.status(), 200);
}
