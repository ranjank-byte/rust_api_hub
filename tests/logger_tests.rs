use rust_api_hub::utils::logger::{log_error, log_info};

#[test]
fn test_log_info_no_panic() {
    log_info("test message");
    assert!(true);
}

#[test]
fn test_log_error_no_panic() {
    log_error("error message");
    assert!(true);
}
