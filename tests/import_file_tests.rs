use axum::Json;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use rust_api_hub::models::repository::TaskRepository;

fn app_state() -> TaskRepository {
    TaskRepository::new()
}

#[tokio::test]
async fn file_import_valid_csv_inserts_all() {
    let repo = app_state();
    // build a simple multipart body with boundary 'BOUND'
    let boundary = "BOUND";
    let csv = "title,description\nOne,desc1\nTwo,desc2\n";
    let mut body = String::new();
    body.push_str(&format!("--{}\r\n", boundary));
    body.push_str("Content-Disposition: form-data; name=\"file\"; filename=\"tasks.csv\"\r\n");
    body.push_str("Content-Type: text/csv\r\n\r\n");
    body.push_str(csv);
    body.push_str(&format!("\r\n--{}--\r\n", boundary));

    let bytes = Bytes::from(body);
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&format!("multipart/form-data; boundary={}", boundary)).unwrap(),
    );

    let (code, Json(resp)) = rust_api_hub::handlers::task_handler::import_tasks_file(
        State(repo.clone()),
        headers,
        bytes,
    )
    .await;
    assert_eq!(code, StatusCode::CREATED);
    assert_eq!(resp["imported"].as_u64().unwrap(), 2);
    assert_eq!(repo.count(), 2);
}

#[tokio::test]
async fn file_import_partial_failure_reports_rows() {
    let repo = app_state();
    let boundary = "BOUND";
    let csv = "title,description\nGood,ok\n,missing-title\n";
    let mut body = String::new();
    body.push_str(&format!("--{}\r\n", boundary));
    body.push_str("Content-Disposition: form-data; name=\"file\"; filename=\"tasks.csv\"\r\n");
    body.push_str("Content-Type: text/csv\r\n\r\n");
    body.push_str(csv);
    body.push_str(&format!("\r\n--{}--\r\n", boundary));

    let bytes = Bytes::from(body);
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&format!("multipart/form-data; boundary={}", boundary)).unwrap(),
    );

    let (code, Json(resp)) = rust_api_hub::handlers::task_handler::import_tasks_file(
        State(repo.clone()),
        headers,
        bytes,
    )
    .await;
    assert_eq!(code, StatusCode::CREATED);
    assert_eq!(resp["imported"].as_u64().unwrap(), 1);
    assert_eq!(resp["failed"].as_u64().unwrap(), 1);
    assert_eq!(repo.count(), 1);
    let errors = resp["errors"].as_array().unwrap();
    assert!(errors.iter().any(|e| e["row"].as_u64().is_some()));
}

#[tokio::test]
async fn file_import_too_large_returns_413() {
    let repo = app_state();
    let boundary = "BOUND";
    // create a csv large enough to exceed the 5 MB limit by repeating a line
    let mut csv = String::from("title,description\n");
    // build a large csv by repeating a long description to exceed 5MB
    for _ in 0..6000 {
        csv.push_str(&format!("tline,{}\n", "x".repeat(1000)));
    }
    let mut body = String::new();
    body.push_str(&format!("--{}\r\n", boundary));
    body.push_str("Content-Disposition: form-data; name=\"file\"; filename=\"tasks.csv\"\r\n");
    body.push_str("Content-Type: text/csv\r\n\r\n");
    body.push_str(&csv);
    body.push_str(&format!("\r\n--{}--\r\n", boundary));

    let bytes = Bytes::from(body);
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&format!("multipart/form-data; boundary={}", boundary)).unwrap(),
    );

    let (code, _resp) = rust_api_hub::handlers::task_handler::import_tasks_file(
        State(repo.clone()),
        headers,
        bytes,
    )
    .await;
    assert_eq!(code, StatusCode::PAYLOAD_TOO_LARGE);
}
