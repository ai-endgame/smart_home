use chrono::Utc;

pub async fn dispatch_webhook(url: &str, rule_name: &str, message: &str) {
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            log::warn!("webhook: failed to build client for '{}': {e}", rule_name);
            return;
        }
    };

    let payload = serde_json::json!({
        "rule": rule_name,
        "message": message,
        "timestamp": Utc::now().to_rfc3339(),
    });

    if let Err(e) = client.post(url).json(&payload).send().await {
        log::warn!("webhook: POST to '{}' for rule '{}' failed: {e}", url, rule_name);
    }
}
