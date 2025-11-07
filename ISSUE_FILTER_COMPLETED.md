Title: feat(api): add completed filter to GET /tasks (?completed=true|false)

Summary

- Add an optional query parameter to GET /tasks: `?completed=true` or `?completed=false`.
- When present, server returns only tasks matching the completed flag. When absent, API returns all tasks (existing behavior).
- This change modifies the `get_tasks` handler to accept `Query<FilterParams>` and filter results in-memory.

Motivation

- Clients need an efficient, server-side filter to avoid fetching all tasks and filtering client-side.
- Keeps the API small and pragmatic while supporting typical UI use-cases (show completed/incomplete tasks).

What changed / planned changes

- src/handlers/task_handler.rs
  - Update `get_tasks` signature to accept `Query<FilterParams>` and retain tasks where `task.completed == params.completed` when provided.
- tests/filter_completed_tests.rs (new)
  - Add integration tests for: no filter (returns all), `?completed=true` (returns only completed), and `?completed=false` (returns only incomplete).
- No change to repository shape required — filtering is applied on the returned list.

How to test locally

- Run the full test suite (recommended):
```powershell
cargo test -q
```
- Or run only the new tests after they are added:
```powershell
cargo test filter_completed_tests -q
```

Suggested tests (integration)

1) returns_all_when_no_filter
- Create 3 tasks where some are completed and some not (use update to set completed=true for some).
- Call `get_tasks(State(repo), Query(FilterParams { completed: None }))` or the equivalent route.
- Expect list length == 3 (all tasks returned).

2) returns_only_completed_when_true
- Seed tasks where 2 tasks are completed and 1 is not.
- Call `get_tasks` with `?completed=true`.
- Expect returned list length == 2 and each task has `completed == true`.

3) returns_only_incomplete_when_false
- Seed tasks where 2 tasks are incomplete and 1 is completed.
- Call `get_tasks` with `?completed=false`.
- Expect returned list length == 2 and each task has `completed == false`.

Suggested labels

- enhancement
- api
- tests

Suggested PR title and commit message

- PR title: feat(api): add completed filter to GET /tasks
- Commit message (example):
```
feat(api): add optional ?completed filter to GET /tasks

- Update get_tasks handler to accept `Query<FilterParams>` and filter results by `completed` flag when provided.
- Add integration tests to cover no-filter, completed=true and completed=false cases.

All tests pass locally.
```

Example gh command to create the issue (if `gh` is installed and authenticated):
```powershell
gh issue create --title "feat(api): add completed filter to GET /tasks" --body-file .\ISSUE_FILTER_COMPLETED.md --label enhancement,api,tests
```

Notes

- This is a low-risk change; filtering happens in-memory on the repository's `list()` result. For large datasets consider adding pagination or repository-level query methods in a follow-up PR.
