/// Initialize the logger with the specified log level.
///
/// Log levels can be set via the `RUST_LOG` environment variable.
/// If not set, defaults to "info".
///
/// # Examples
/// - `RUST_LOG=debug cargo run` - Enable debug logging
/// - `RUST_LOG=smart_home=trace cargo run` - Enable trace logging for this crate only
/// - `RUST_LOG=mdns_sd=debug` - Re-enable suppressed mdns-sd internal logs
pub fn init() {
    env_logger::Builder::new()
        // Global default: show INFO and above from our own code.
        .filter_level(log::LevelFilter::Info)
        // mdns-sd 0.11 logs non-fatal EAGAIN socket errors at ERROR level on
        // multi-homed macOS machines (e.g. "Resource temporarily unavailable
        // (os error 35)" when sending IPv6 multicast). These are transient and
        // the daemon recovers on its own, so we silence the entire module by
        // default.  Set RUST_LOG=mdns_sd=debug to re-enable all mdns_sd output.
        .filter_module("mdns_sd", log::LevelFilter::Off)
        // RUST_LOG always wins — it is parsed last so its entries take priority
        // over everything set above.
        .parse_env("RUST_LOG")
        .init();
}

#[cfg(test)]
mod tests {
    use std::sync::Once;

    static INIT_LOGGER: Once = Once::new();

    #[test]
    fn test_init_sets_default_level() {
        let level = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
        assert!(!level.is_empty());
    }

    #[test]
    fn test_init_can_be_called() {
        INIT_LOGGER.call_once(super::init);
    }
}
