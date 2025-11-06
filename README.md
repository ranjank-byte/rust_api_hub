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
- `GET /tasks/{id}` — get a single task
- `PUT /tasks/{id}` — update a task (partial fields allowed)
- `DELETE /tasks/{id}` — delete a task

Example curl (when server is running):

```powershell
# create
curl -X POST http://127.0.0.1:8080/tasks -H "Content-Type: application/json" -d '{"title":"t","description":"d"}'

# list
curl http://127.0.0.1:8080/tasks

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

