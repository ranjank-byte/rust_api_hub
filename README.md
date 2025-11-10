# rust_api_hub

A small Axum-based Task Management REST API used as a training repository.  
This repo is intentionally structured for iterative PR creation (features, tests, bugfixes).

## Overview
- Language: Rust (Axum)
- Purpose: Create/List/Get/Update/Delete tasks
- Storage: in-memory (`HashMap`) protected by `parking_lot::RwLock`
- Tests: unit + integration (10+ test cases included)
- Dockerfile: intentionally **omitted** (tooling team will dockerize)

## Running locally

Quick checks:

1. Run the test-suite (recommended):

	cargo test

2. Run the binary:

	cargo run

Note: in this repository the `main` binary currently constructs the router and
initializes logging but does not actively bind and serve HTTP by default. When
you run `cargo run` you'll see a log like:

```
[2025-11-06T18:21:33Z INFO  rust_api_hub] Server running at http://127.0.0.1:8080
```

That log indicates the configured address, but the process will not accept
incoming HTTP connections until server startup is enabled in `src/main.rs`.

If you'd like the binary to actually listen for requests, I can add the
startup snippet to `main.rs` and verify `cargo run` starts the HTTP server.

## Endpoints

The API exposes the following routes (when the server is listening):

- `POST /tasks` — create a task (JSON payload: { "title": "...", "description": "..." })
- `GET /tasks` — list tasks

List query parameters (GET /tasks)

- `completed` (optional) — filter by completion status. Use `?completed=true` or `?completed=false`.
- `page` (optional) — 1-based page number for pagination. Default: `1`.
- `per_page` (optional) — number of items per page. Default: `20`, capped at `100`.
- `sort` (optional) — sorting key. Supported: `created_at` or `priority`, with optional `:asc` / `:desc` suffix (default asc).
  - Examples: `?sort=created_at:desc`, `?sort=priority:asc`

The `GET /tasks` response now returns a JSON object with metadata, for example:

```json
{
	"items": [ /* array of task objects */ ],
	"total": 123,
	"page": 1,
	"per_page": 20
}
```
- `GET /tasks/{id}` — get a single task
- `PUT /tasks/{id}` — update a task (partial fields allowed)
- `DELETE /tasks/{id}` — delete a task

- `PUT /tasks/{id}/tags` — replace the tag set for a task (payload: `{ "tags": ["feature", "backend"] }`)
- `GET /tasks/{id}/tags` — fetch the current tags for a task
- `GET /tasks/search/by_tag?tag=...` — list tasks containing the tag (case-insensitive)

- `PUT /tasks/{id}/priority` — set task priority (payload: `{ "priority": "high" }`)
- `GET /tasks/{id}/priority` — get task priority
- `GET /tasks/search/by_priority?priority=...` — list tasks with specific priority (low, medium, high, critical)

- `GET /tasks/stats` — retrieve statistics about all tasks. Returns:
	- `total` — total number of tasks
	- `completed` — number of completed tasks
	- `incomplete` — number of incomplete tasks
	- `tag_distribution` — array of `{ tag, count }` objects for the top 10 most-used tags (sorted by count descending, then alphabetically)
	- `oldest_created_at` — ISO 8601 timestamp of the oldest task (null if no tasks)
	- `newest_created_at` — ISO 8601 timestamp of the newest task (null if no tasks)

- `POST /tasks/import` — import tasks in bulk. Accepts either:
	- `application/json` — a JSON array of TaskCreate objects: `[{"title":"...","description":"..."}, ...]`.
	- `text/csv` — CSV body with header row containing `title,description`.
	- The endpoint validates rows (title must be non-empty), allows partial successes, and returns `201 Created` with a summary:

```json
{
	"imported": 3,
	"failed": 1,
	"errors": [{"index":0,"error":"..."}],
	"tasks": [ /* created tasks */ ]
}
```

	- On invalid payload (unparseable JSON or invalid UTF-8 CSV) the endpoint returns `400 Bad Request`.

	- `POST /tasks/import/file` — upload a CSV file using multipart/form-data (field name `file`).
		- Useful for browser-based or file-upload clients.
		- The server enforces a maximum upload size (5 MB by default) and returns `413 Payload Too Large` if exceeded.
		- The response mirrors the unified import format and reports partial successes: `{ imported, failed, errors, tasks }`.

Example curl (when server is running):

```powershell
# create
curl -X POST http://127.0.0.1:8080/tasks -H "Content-Type: application/json" -d '{"title":"t","description":"d"}'

# list
curl http://127.0.0.1:8080/tasks

# list only completed tasks
curl "http://127.0.0.1:8080/tasks?completed=true"

# paginated list, page 2, 10 items per page
curl "http://127.0.0.1:8080/tasks?page=2&per_page=10"

# sorted by created_at descending
curl "http://127.0.0.1:8080/tasks?sort=created_at:desc"

# sorted by priority descending (critical first)
curl "http://127.0.0.1:8080/tasks?sort=priority:desc"

# get
curl http://127.0.0.1:8080/tasks/<uuid>

# update
curl -X PUT http://127.0.0.1:8080/tasks/<uuid> -H "Content-Type: application/json" -d '{"title":"new"}'

# delete
curl -X DELETE http://127.0.0.1:8080/tasks/<uuid>

# replace tags (normalized to lowercase, trimmed, deduplicated)
curl -X PUT http://127.0.0.1:8080/tasks/<uuid>/tags \
	-H "Content-Type: application/json" \
	-d '{"tags":["Feature","backend","feature"]}'

# get tags
curl http://127.0.0.1:8080/tasks/<uuid>/tags

# search by tag (case-insensitive)
curl "http://127.0.0.1:8080/tasks/search/by_tag?tag=feature"

# set priority (case-insensitive: low, medium, high, critical)
curl -X PUT http://127.0.0.1:8080/tasks/<uuid>/priority \
	-H "Content-Type: application/json" \
	-d '{"priority":"high"}'

# get priority
curl http://127.0.0.1:8080/tasks/<uuid>/priority

# search by priority
curl "http://127.0.0.1:8080/tasks/search/by_priority?priority=critical"

# get statistics
curl http://127.0.0.1:8080/tasks/stats
```

## Notes
- Keep PRs small and test-driven.
- Do not mix languages.

## Tags

- Each task now includes a `tags` array in its JSON representation.
- Managing tags uses dedicated endpoints:
	- `PUT /tasks/{id}/tags` to replace all tags for a task.
	- `GET /tasks/{id}/tags` to view current tags.
	- `GET /tasks/search/by_tag?tag=...` to retrieve tasks that include a given tag.
- Validation rules:
	- Tags are trimmed and lowercased.
	- Empty/whitespace-only tags are rejected (400).
	- Max tag length: 64 characters.
	- Duplicates are removed case-insensitively.

Backwards compatibility: Task creation/update DTOs are unchanged; tags are managed solely via the dedicated tags endpoints above.

## Priority

- Each task includes a `priority` field in its JSON representation.
- Priority levels: `low`, `medium` (default), `high`, `critical`
- Managing priorities:
	- `PUT /tasks/{id}/priority` to set task priority (case-insensitive input)
	- `GET /tasks/{id}/priority` to view current priority
	- `GET /tasks/search/by_priority?priority=...` to filter tasks by priority level
	- `GET /tasks?sort=priority:asc` or `sort=priority:desc` to sort tasks by priority
- Validation rules:
	- Only valid priority values accepted: `low`, `medium`, `high`, `critical`
	- Case-insensitive parsing
	- Invalid priority values return 400 Bad Request
- New tasks default to `medium` priority
- Priority sorting: low (1) < medium (2) < high (3) < critical (4)

Backwards compatibility: Task creation/update DTOs are unchanged; priority is managed via dedicated priority endpoints.


