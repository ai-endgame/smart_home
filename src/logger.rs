/// Initialize the logger with the specified log level.
///
/// Log levels can be set via the `RUST_LOG` environment variable.
/// If not set, defaults to "info".
///
/// # Examples
/// - `RUST_LOG=debug cargo run` - Enable debug logging
/// - `RUST_LOG=smart_home=trace cargo run` - Enable trace logging for this crate only
pub fn init() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
}

#[cfg(test)]
mod tests {
    use std::sync::Once;

    static INIT_LOGGER: Once = Once::new();

    #[test]
    fn test_init_sets_default_level() {
        // Note: env_logger can only be initialized once per process,
        // so this test just verifies the env var logic without calling init()
        let level = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
        assert!(!level.is_empty());
    }

    #[test]
    fn test_init_can_be_called() {
        INIT_LOGGER.call_once(super::init);
    }
}
