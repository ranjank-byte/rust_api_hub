use axum::body::Body;
use axum::http::Request;
use rust_api_hub::routes::tasks::routes;
use tower::ServiceExt; // oneshot

#[tokio::test]
async fn test_health_ok() {
    let app = routes();
    let req = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn test_info_ok() {
    let app = routes();
    let req = Request::builder().uri("/info").body(Body::empty()).unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
}
