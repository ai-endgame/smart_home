pub const DEFAULT_BIND_ADDR: &str = "127.0.0.1:8080";

pub struct Config {
    /// Raw bind address string — parsed to SocketAddr by the server startup code
    /// so that an invalid value surfaces as a proper `ServerStartError`.
    pub bind_addr: String,
    pub database_url: Option<String>,
    /// Comma-separated list of allowed CORS origins. Defaults to allowing all.
    pub cors_origins: Vec<String>,
}

impl Config {
    pub fn from_env() -> Self {
        let bind_addr = std::env::var("SMART_HOME_BIND_ADDR")
            .or_else(|_| std::env::var("SMART_HOME_SERVER_ADDR"))
            .unwrap_or_else(|_| DEFAULT_BIND_ADDR.to_string());

        let cors_origins = std::env::var("CORS_ORIGINS")
            .map(|s| s.split(',').map(|o| o.trim().to_string()).collect())
            .unwrap_or_default();

        Config {
            bind_addr,
            database_url: std::env::var("DATABASE_URL").ok(),
            cors_origins,
        }
    }
}
