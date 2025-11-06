//! Integration test helpers

pub fn setup_logging() {
    // try_init will not panic if logger already initialized
    let _ = env_logger::try_init();
}
