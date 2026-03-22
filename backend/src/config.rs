pub const DEFAULT_BIND_ADDR: &str = "127.0.0.1:8080";

pub struct Config {
    /// Raw bind address string — parsed to SocketAddr by the server startup code
    /// so that an invalid value surfaces as a proper `ServerStartError`.
    pub bind_addr: String,
    pub database_url: Option<String>,
    /// Comma-separated list of allowed CORS origins. Defaults to allowing all.
    pub cors_origins: Vec<String>,
    /// Optional admin token required for privileged endpoints (e.g. /server/stop).
    /// When `None` the endpoint is disabled (returns 403).
    pub admin_token: Option<String>,
    /// Optional MQTT broker URL (e.g. `mqtt://localhost:1883`).
    /// When `None` the MQTT subscriber loop is not started.
    pub mqtt_url: Option<String>,
    /// Optional API key. When set, all write (non-GET) requests must supply
    /// `Authorization: Bearer <key>` or `X-API-Key: <key>`.
    pub api_key: Option<String>,
    /// Optional PIN for session-based auth. When set, clients must login via
    /// `POST /api/auth/login` to obtain a session token for write access.
    pub smart_home_pin: Option<String>,
    /// Whether any form of auth is enabled (api_key or smart_home_pin).
    pub auth_enabled: bool,
}

impl Config {
    pub fn from_env() -> Self {
        let bind_addr = std::env::var("SMART_HOME_BIND_ADDR")
            .or_else(|_| std::env::var("SMART_HOME_SERVER_ADDR"))
            .unwrap_or_else(|_| DEFAULT_BIND_ADDR.to_string());

        let cors_origins = std::env::var("CORS_ORIGINS")
            .map(|s| s.split(',').map(|o| o.trim().to_string()).collect())
            .unwrap_or_default();

        let admin_token = std::env::var("ADMIN_TOKEN").ok().filter(|s| !s.is_empty());

        let api_key = std::env::var("API_KEY").ok().filter(|s| !s.is_empty());
        let smart_home_pin = std::env::var("SMART_HOME_PIN").ok().filter(|s| !s.is_empty());
        let auth_enabled = api_key.is_some() || smart_home_pin.is_some();

        Config {
            bind_addr,
            database_url: std::env::var("DATABASE_URL").ok(),
            cors_origins,
            admin_token,
            mqtt_url: std::env::var("MQTT_URL").ok().filter(|s| !s.is_empty()),
            api_key,
            smart_home_pin,
            auth_enabled,
        }
    }
}
