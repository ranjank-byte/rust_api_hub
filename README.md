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
- `sort` (optional) — sorting key. Supported: `created_at`, or `created_at:asc` / `created_at:desc` (default asc).

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

- `POST /tasks/import` — import tasks from a JSON array. Request body: `[{"title":"...","description":"..."}, ...]`.
	- Response: `201 Created` with body `{ "imported": N, "tasks": [ /* created tasks */ ] }`.

- `POST /tasks/import/csv` — import tasks from CSV body. Expects a header row with `title,description`.
	- Content-Type: `text/csv` (or send raw body). Example CSV:

```
title,description
task A,desc A
task B,desc B
```

	- Response: `201 Created` with body `{ "imported": N, "tasks": [ /* created tasks */ ] }`.
	- On parse errors or invalid UTF-8 the endpoint returns `400 Bad Request` with an error message.

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

# get
curl http://127.0.0.1:8080/tasks/<uuid>

# update
curl -X PUT http://127.0.0.1:8080/tasks/<uuid> -H "Content-Type: application/json" -d '{"title":"new"}'

# delete
curl -X DELETE http://127.0.0.1:8080/tasks/<uuid>
```

## Notes
- Keep PRs small and test-driven.
- Do not mix languages.

