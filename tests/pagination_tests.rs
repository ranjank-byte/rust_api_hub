use axum::Json;
use axum::extract::State;
use rust_api_hub::models::repository::TaskRepository;
use rust_api_hub::models::task::TaskCreate;

fn app_state() -> TaskRepository {
    TaskRepository::new()
}

#[tokio::test]
async fn pagination_returns_correct_page() {
    let repo = app_state();
    // create 25 tasks
    for i in 0..25 {
        let payload = TaskCreate {
            title: format!("t{}", i),
            description: "d".into(),
        };
        let (_code, _created) =
            rust_api_hub::handlers::task_handler::create_task(State(repo.clone()), Json(payload))
                .await;
    }

    use rust_api_hub::handlers::task_handler::ListParams;
    let params = axum::extract::Query(ListParams {
        page: Some(2),
        per_page: Some(10),
        sort: None,
        completed: None,
    });
    let Json(resp) =
        rust_api_hub::handlers::task_handler::get_tasks(State(repo.clone()), params).await;
    assert_eq!(resp["items"].as_array().unwrap().len(), 10);
    assert_eq!(resp["page"].as_u64().unwrap(), 2);
    assert_eq!(resp["per_page"].as_u64().unwrap(), 10);
    assert_eq!(resp["total"].as_u64().unwrap(), 25);
}

#[tokio::test]
async fn per_page_limits_results_and_caps() {
    let repo = app_state();
    for i in 0..5 {
        let payload = TaskCreate {
            title: format!("t{}", i),
            description: "d".into(),
        };
        let (_code, _created) =
            rust_api_hub::handlers::task_handler::create_task(State(repo.clone()), Json(payload))
                .await;
    }

    use rust_api_hub::handlers::task_handler::ListParams;
    // request huge per_page
    let params = axum::extract::Query(ListParams {
        page: Some(1),
        per_page: Some(1000),
        sort: None,
        completed: None,
    });
    let Json(resp) =
        rust_api_hub::handlers::task_handler::get_tasks(State(repo.clone()), params).await;
    // items should be 5 (only 5 tasks exist)
    assert_eq!(resp["items"].as_array().unwrap().len(), 5);
    // server should report capped per_page (100)
    assert_eq!(resp["per_page"].as_u64().unwrap(), 100);
    assert_eq!(resp["total"].as_u64().unwrap(), 5);
}

#[tokio::test]
async fn sorting_by_created_at_desc() {
    let repo = app_state();
    for i in 0..5 {
        let payload = TaskCreate {
            title: format!("t{}", i),
            description: "d".into(),
        };
        let (_code, _created) =
            rust_api_hub::handlers::task_handler::create_task(State(repo.clone()), Json(payload))
                .await;
        // ensure distinct timestamps
        std::thread::sleep(std::time::Duration::from_millis(1));
    }

    use rust_api_hub::handlers::task_handler::ListParams;
    let params = axum::extract::Query(ListParams {
        page: Some(1),
        per_page: Some(5),
        sort: Some("created_at:desc".into()),
        completed: None,
    });
    let Json(resp) =
        rust_api_hub::handlers::task_handler::get_tasks(State(repo.clone()), params).await;
    let items = resp["items"].as_array().unwrap();
    assert_eq!(items[0]["title"].as_str().unwrap(), "t4");
}
