use axum::Json;
use axum::extract::State;
use rust_api_hub::models::repository::TaskRepository;
use rust_api_hub::models::task::TaskCreate;
use serde_json::json;

fn repo() -> TaskRepository {
    TaskRepository::new()
}

#[tokio::test]
async fn set_and_get_tags_roundtrip() {
    let repo = repo();
    // create task
    let payload = TaskCreate {
        title: "alpha".into(),
        description: "d".into(),
    };
    let (_code, Json(task)) =
        rust_api_hub::handlers::task_handler::create_task(State(repo.clone()), Json(payload)).await;

    // set tags
    let tags_payload = rust_api_hub::handlers::task_handler::TagsPayload {
        tags: vec!["Feature", "Backend", "feature"]
            .into_iter()
            .map(String::from)
            .collect(),
    };
    let (code_set, Json(resp_set)) = rust_api_hub::handlers::task_handler::set_tags(
        axum::extract::Path(task.id.to_string()),
        State(repo.clone()),
        Json(tags_payload),
    )
    .await;
    assert_eq!(code_set.as_u16(), 200);
    let updated = resp_set["task"].clone();
    let tag_array = updated["tags"].as_array().unwrap();
    assert_eq!(tag_array.len(), 2); // deduplicated & normalized

    // get tags
    let (code_get, Json(resp_tags)) = rust_api_hub::handlers::task_handler::get_tags(
        axum::extract::Path(task.id.to_string()),
        State(repo.clone()),
    )
    .await;
    assert_eq!(code_get.as_u16(), 200);
    assert_eq!(resp_tags["tags"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn search_by_tag_returns_only_matching() {
    let repo = repo();
    // create two tasks
    for name in ["t1", "t2"].iter() {
        let payload = TaskCreate {
            title: name.to_string(),
            description: "d".into(),
        };
        let (_code, Json(task)) =
            rust_api_hub::handlers::task_handler::create_task(State(repo.clone()), Json(payload))
                .await;
        // set tags differently
        let tags: Vec<String> = if *name == "t1" {
            vec!["A", "B"]
        } else {
            vec!["B", "C"]
        }
        .into_iter()
        .map(|s| s.to_string())
        .collect();
        let tags_payload = rust_api_hub::handlers::task_handler::TagsPayload { tags };
        let _ = rust_api_hub::handlers::task_handler::set_tags(
            axum::extract::Path(task.id.to_string()),
            State(repo.clone()),
            Json(tags_payload),
        )
        .await;
    }

    // search for tag 'a'
    let q =
        axum::extract::Query(rust_api_hub::handlers::task_handler::TagQuery { tag: "a".into() });
    let Json(resp) =
        rust_api_hub::handlers::task_handler::get_tasks_by_tag(State(repo.clone()), q).await;
    let items = resp["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["title"].as_str().unwrap(), "t1");
}

#[tokio::test]
async fn invalid_tags_rejected() {
    let repo = repo();
    let payload = TaskCreate {
        title: "bad-tags".into(),
        description: "d".into(),
    };
    let (_code, Json(task)) =
        rust_api_hub::handlers::task_handler::create_task(State(repo.clone()), Json(payload)).await;

    // include empty tag -> should fail
    let tags_payload = rust_api_hub::handlers::task_handler::TagsPayload {
        tags: vec!["valid".to_string(), "   ".to_string()],
    };
    let (code_set, Json(resp_set)) = rust_api_hub::handlers::task_handler::set_tags(
        axum::extract::Path(task.id.to_string()),
        State(repo.clone()),
        Json(tags_payload),
    )
    .await;
    assert_eq!(code_set.as_u16(), 400);
    assert!(resp_set["error"].as_str().unwrap().contains("empty"));
}
