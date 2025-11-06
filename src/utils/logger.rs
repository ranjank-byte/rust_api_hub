//! Small logging helpers.
//! Kept tiny but present to show structured logging usage and be a PR-able unit.

pub fn log_info(msg: &str) {
    log::info!("{}", msg);
}

pub fn log_error(msg: &str) {
    log::error!("{}", msg);
}

// unit tests moved to `tests/logger_tests.rs` as integration tests
