use axum::Json;
use axum::extract::State;
use rust_api_hub::models::repository::TaskRepository;
use rust_api_hub::models::task::TaskCreate;

fn repo() -> TaskRepository {
    TaskRepository::new()
}

#[tokio::test]
async fn stats_empty_repo_returns_zeros() {
    let repo = repo();
    let Json(resp) = rust_api_hub::handlers::task_handler::get_stats(State(repo.clone())).await;

    assert_eq!(resp["total"].as_u64().unwrap(), 0);
    assert_eq!(resp["completed"].as_u64().unwrap(), 0);
    assert_eq!(resp["incomplete"].as_u64().unwrap(), 0);
    assert_eq!(resp["tag_distribution"].as_array().unwrap().len(), 0);
    assert!(resp["oldest_created_at"].is_null());
    assert!(resp["newest_created_at"].is_null());
}

#[tokio::test]
async fn stats_mixed_completed_counts_correct() {
    let repo = repo();
    // create 5 tasks: 3 completed, 2 incomplete
    for i in 0..5 {
        let payload = TaskCreate {
            title: format!("task{}", i),
            description: "d".into(),
        };
        let (_code, Json(task)) =
            rust_api_hub::handlers::task_handler::create_task(State(repo.clone()), Json(payload))
                .await;

        if i < 3 {
            // mark first 3 as completed
            let upd = rust_api_hub::models::task::TaskUpdate {
                title: None,
                description: None,
                completed: Some(true),
            };
            let _ = rust_api_hub::handlers::task_handler::update_task(
                axum::extract::Path(task.id.to_string()),
                State(repo.clone()),
                Json(upd),
            )
            .await;
        }
    }

    let Json(resp) = rust_api_hub::handlers::task_handler::get_stats(State(repo.clone())).await;

    assert_eq!(resp["total"].as_u64().unwrap(), 5);
    assert_eq!(resp["completed"].as_u64().unwrap(), 3);
    assert_eq!(resp["incomplete"].as_u64().unwrap(), 2);
    assert!(resp["oldest_created_at"].is_string());
    assert!(resp["newest_created_at"].is_string());
}

#[tokio::test]
async fn stats_tag_distribution_sorted_by_frequency() {
    let repo = repo();
    // create 4 tasks with overlapping tags
    // task0: [a, b]
    // task1: [a, c]
    // task2: [b, c]
    // task3: [a]
    // expected: a=3, b=2, c=2

    let tasks_tags = vec![vec!["a", "b"], vec!["a", "c"], vec!["b", "c"], vec!["a"]];

    for (i, tags) in tasks_tags.iter().enumerate() {
        let payload = TaskCreate {
            title: format!("t{}", i),
            description: "d".into(),
        };
        let (_code, Json(task)) =
            rust_api_hub::handlers::task_handler::create_task(State(repo.clone()), Json(payload))
                .await;

        // set tags
        let tags_payload = rust_api_hub::handlers::task_handler::TagsPayload {
            tags: tags.iter().map(|s| s.to_string()).collect(),
        };
        let _ = rust_api_hub::handlers::task_handler::set_tags(
            axum::extract::Path(task.id.to_string()),
            State(repo.clone()),
            Json(tags_payload),
        )
        .await;
    }

    let Json(resp) = rust_api_hub::handlers::task_handler::get_stats(State(repo.clone())).await;

    let dist = resp["tag_distribution"].as_array().unwrap();
    assert_eq!(dist.len(), 3); // a, b, c

    // top tag should be "a" with count 3
    assert_eq!(dist[0]["tag"].as_str().unwrap(), "a");
    assert_eq!(dist[0]["count"].as_u64().unwrap(), 3);

    // second and third are b and c with count 2 each (alphabetically sorted for ties)
    assert_eq!(dist[1]["tag"].as_str().unwrap(), "b");
    assert_eq!(dist[1]["count"].as_u64().unwrap(), 2);

    assert_eq!(dist[2]["tag"].as_str().unwrap(), "c");
    assert_eq!(dist[2]["count"].as_u64().unwrap(), 2);
}
