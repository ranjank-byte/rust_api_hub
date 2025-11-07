use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use chrono::DateTime;
use rust_api_hub::handlers::task_handler::create_task;
use rust_api_hub::models::repository::TaskRepository;
use rust_api_hub::models::task::TaskCreate;

fn app_state() -> TaskRepository {
    TaskRepository::new()
}

#[tokio::test]
async fn created_at_is_present_and_valid_format() {
    let repo = app_state();
    let payload = TaskCreate {
        title: "t1".into(),
        description: "d1".into(),
    };
    let (code, created) = create_task(State(repo.clone()), Json(payload)).await;
    assert_eq!(code, StatusCode::CREATED);
    // created_at should be a valid RFC3339 timestamp when serialized
    let ca = created.created_at.to_rfc3339();
    let parsed = DateTime::parse_from_rfc3339(&ca).expect("created_at should be RFC3339");
    // parsed must exist; check timestamp > 1_000_000_000 as a sanity check (around 2001+)
    assert!(parsed.timestamp() > 1_000_000_000);
}

#[tokio::test]
async fn created_at_uniqueness_for_multiple_creates() {
    let repo = app_state();
    let mut timestamps = Vec::new();
    for i in 0..3 {
        let payload = TaskCreate {
            title: format!("t{}", i),
            description: "d".into(),
        };
        let (code, created) = create_task(State(repo.clone()), Json(payload)).await;
        assert_eq!(code, StatusCode::CREATED);
        timestamps.push(created.created_at.to_rfc3339());
        // small sleep to avoid identical timestamps on very fast systems
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    // Not all timestamps should be equal
    let all_equal = timestamps.iter().all(|t| t == &timestamps[0]);
    assert!(
        !all_equal,
        "expected at least one timestamp to differ between creates"
    );
}

#[tokio::test]
async fn created_at_retained_in_repository_and_serialized() {
    let repo = app_state();
    let payload = TaskCreate {
        title: "t1".into(),
        description: "d1".into(),
    };
    let (_code, created) = create_task(State(repo.clone()), Json(payload)).await;
    let id = created.id;
    // fetch stored task
    let stored = repo.get(&id).expect("task should be present");
    // ensure stored.created_at matches created.created_at
    assert_eq!(
        stored.created_at.to_rfc3339(),
        created.created_at.to_rfc3339()
    );
    // ensure parseable
    let _parsed: DateTime<chrono::FixedOffset> =
        DateTime::parse_from_rfc3339(&stored.created_at.to_rfc3339()).unwrap();
}
