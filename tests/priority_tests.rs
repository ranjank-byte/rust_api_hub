use axum::Json;
use axum::extract::{Path, Query, State};
use rust_api_hub::handlers::task_handler::PriorityPayload;
use rust_api_hub::models::repository::TaskRepository;
use rust_api_hub::models::task::TaskCreate;

fn repo() -> TaskRepository {
    TaskRepository::new()
}

#[tokio::test]
async fn set_and_get_priority_roundtrip() {
    let repo = repo();

    // create a task
    let payload = TaskCreate {
        title: "test task".into(),
        description: "desc".into(),
    };
    let (_code, Json(task)) =
        rust_api_hub::handlers::task_handler::create_task(State(repo.clone()), Json(payload)).await;

    // default priority should be medium
    assert_eq!(task.priority, rust_api_hub::models::task::Priority::Medium);

    // set priority to high
    let priority_payload = PriorityPayload {
        priority: "high".into(),
    };
    let result = rust_api_hub::handlers::task_handler::set_priority(
        Path(task.id.to_string()),
        State(repo.clone()),
        Json(priority_payload),
    )
    .await;
    assert!(result.is_ok());

    // get priority and verify
    let result = rust_api_hub::handlers::task_handler::get_priority(
        Path(task.id.to_string()),
        State(repo.clone()),
    )
    .await;
    assert!(result.is_ok());
    let Json(resp) = result.unwrap();
    assert_eq!(resp["priority"].as_str().unwrap(), "high");
}

#[tokio::test]
async fn search_by_priority_filters_correctly() {
    let repo = repo();

    // create 5 tasks with different priorities
    let priorities = vec!["low", "medium", "high", "critical", "medium"];

    for (i, prio) in priorities.iter().enumerate() {
        let payload = TaskCreate {
            title: format!("task{}", i),
            description: "d".into(),
        };
        let (_code, Json(task)) =
            rust_api_hub::handlers::task_handler::create_task(State(repo.clone()), Json(payload))
                .await;

        // set priority
        let priority_payload = PriorityPayload {
            priority: prio.to_string(),
        };
        let _ = rust_api_hub::handlers::task_handler::set_priority(
            Path(task.id.to_string()),
            State(repo.clone()),
            Json(priority_payload),
        )
        .await;
    }

    // search for medium priority tasks (should be 2)
    let mut params = std::collections::HashMap::new();
    params.insert("priority".to_string(), "medium".to_string());

    let result = rust_api_hub::handlers::task_handler::get_tasks_by_priority(
        State(repo.clone()),
        Query(params),
    )
    .await;
    assert!(result.is_ok());
    let Json(tasks) = result.unwrap();
    assert_eq!(tasks.len(), 2);
    for task in tasks {
        assert_eq!(task.priority, rust_api_hub::models::task::Priority::Medium);
    }
}

#[tokio::test]
async fn invalid_priority_rejected() {
    let repo = repo();

    // create a task
    let payload = TaskCreate {
        title: "test".into(),
        description: "d".into(),
    };
    let (_code, Json(task)) =
        rust_api_hub::handlers::task_handler::create_task(State(repo.clone()), Json(payload)).await;

    // try to set invalid priority
    let priority_payload = PriorityPayload {
        priority: "invalid".into(),
    };
    let result = rust_api_hub::handlers::task_handler::set_priority(
        Path(task.id.to_string()),
        State(repo.clone()),
        Json(priority_payload),
    )
    .await;

    assert!(result.is_err());
    let (status, msg) = result.unwrap_err();
    assert_eq!(status, axum::http::StatusCode::BAD_REQUEST);
    assert!(msg.contains("invalid priority"));
}

#[tokio::test]
async fn sort_by_priority_orders_correctly() {
    let repo = repo();

    // create tasks with different priorities
    let priorities = vec!["low", "critical", "medium", "high"];

    for (i, prio) in priorities.iter().enumerate() {
        let payload = TaskCreate {
            title: format!("task{}", i),
            description: "d".into(),
        };
        let (_code, Json(task)) =
            rust_api_hub::handlers::task_handler::create_task(State(repo.clone()), Json(payload))
                .await;

        let priority_payload = PriorityPayload {
            priority: prio.to_string(),
        };
        let _ = rust_api_hub::handlers::task_handler::set_priority(
            Path(task.id.to_string()),
            State(repo.clone()),
            Json(priority_payload),
        )
        .await;
    }

    // get tasks sorted by priority ascending (low to critical)
    let params = rust_api_hub::handlers::task_handler::ListParams {
        completed: None,
        page: None,
        per_page: None,
        sort: Some("priority:asc".into()),
    };

    let Json(resp) =
        rust_api_hub::handlers::task_handler::get_tasks(State(repo.clone()), Query(params)).await;

    let items = resp["items"].as_array().unwrap();
    assert_eq!(items.len(), 4);

    // verify order: low, medium, high, critical
    assert_eq!(items[0]["priority"].as_str().unwrap(), "low");
    assert_eq!(items[1]["priority"].as_str().unwrap(), "medium");
    assert_eq!(items[2]["priority"].as_str().unwrap(), "high");
    assert_eq!(items[3]["priority"].as_str().unwrap(), "critical");

    // test descending order
    let params_desc = rust_api_hub::handlers::task_handler::ListParams {
        completed: None,
        page: None,
        per_page: None,
        sort: Some("priority:desc".into()),
    };

    let Json(resp_desc) =
        rust_api_hub::handlers::task_handler::get_tasks(State(repo.clone()), Query(params_desc))
            .await;

    let items_desc = resp_desc["items"].as_array().unwrap();
    assert_eq!(items_desc[0]["priority"].as_str().unwrap(), "critical");
    assert_eq!(items_desc[1]["priority"].as_str().unwrap(), "high");
    assert_eq!(items_desc[2]["priority"].as_str().unwrap(), "medium");
    assert_eq!(items_desc[3]["priority"].as_str().unwrap(), "low");
}
