use axum::Json;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use rust_api_hub::models::repository::TaskRepository;
use rust_api_hub::models::task::TaskCreate;

fn app_state() -> TaskRepository {
    TaskRepository::new()
}

#[tokio::test]
async fn import_json_valid_inserts_all() {
    let repo = app_state();
    let payload = vec![
        TaskCreate {
            title: "A".into(),
            description: "d1".into(),
        },
        TaskCreate {
            title: "B".into(),
            description: "d2".into(),
        },
    ];
    let body = Bytes::from(serde_json::to_vec(&payload).unwrap());
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );

    let (code, Json(resp)) =
        rust_api_hub::handlers::task_handler::import_tasks(State(repo.clone()), headers, body)
            .await;
    assert_eq!(code, StatusCode::CREATED);
    assert_eq!(resp["imported"].as_u64().unwrap(), 2);
    assert_eq!(repo.count(), 2);
    assert!(resp["tasks"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn import_json_partial_failure_reports_errors() {
    let repo = app_state();
    // second entry has empty title -> should be invalid (validation rule)
    let payload = vec![
        TaskCreate {
            title: "Good".into(),
            description: "d1".into(),
        },
        TaskCreate {
            title: "".into(),
            description: "d-bad".into(),
        },
    ];
    let body = Bytes::from(serde_json::to_vec(&payload).unwrap());
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );

    let (code, Json(resp)) =
        rust_api_hub::handlers::task_handler::import_tasks(State(repo.clone()), headers, body)
            .await;
    assert_eq!(code, StatusCode::CREATED);
    assert_eq!(resp["imported"].as_u64().unwrap(), 1);
    assert_eq!(resp["failed"].as_u64().unwrap(), 1);
    let errors = resp["errors"].as_array().unwrap();
    assert!(!errors.is_empty());
    assert_eq!(repo.count(), 1);
}

#[tokio::test]
async fn import_csv_partial_rows_are_reported() {
    let repo = app_state();
    // CSV: header + one valid row + one row missing title
    let csv = "title,description\nOkay,desc1\n,missing-title\n";
    let body = Bytes::from(csv);
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("text/csv"));

    let (code, Json(resp)) =
        rust_api_hub::handlers::task_handler::import_tasks(State(repo.clone()), headers, body)
            .await;
    assert_eq!(code, StatusCode::CREATED);
    assert_eq!(resp["imported"].as_u64().unwrap(), 1);
    assert_eq!(resp["failed"].as_u64().unwrap(), 1);
    assert_eq!(repo.count(), 1);
    let errors = resp["errors"].as_array().unwrap();
    assert!(errors.iter().any(|e| e["error"].as_str().is_some()));
}
