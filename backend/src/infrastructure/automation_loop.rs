use std::time::Duration;

use chrono::Local;
use log::info;
use tokio::time;

use crate::domain::AutomationEngine;
use crate::http::helpers::record_event;
use crate::http::types::EventKind;
use crate::state::AppState;

/// Spawn a background task that evaluates time/sun automation triggers every 60 seconds.
pub fn start_automation_loop(state: AppState) {
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(60));
        interval.tick().await; // skip the immediate first tick
        loop {
            interval.tick().await;
            let now = Local::now().time();
            let (triggered, notifications) = {
                let mut home = state.home.write().await;
                let automation = state.automation.read().await;
                let presence = state.presence.read().await;
                let triggered = automation.evaluate_rules_with_meta(&home, &presence, now);
                if triggered.is_empty() { continue; }
                let actions: Vec<_> = triggered.iter().map(|(_, _, a)| a.clone()).collect();
                info!("automation loop: {} action(s) triggered at {}", actions.len(), now.format("%H:%M"));
                let notifications = AutomationEngine::execute_actions(&actions, &mut home, false);
                (triggered, notifications)
            };
            for msg in &notifications {
                record_event(&state, EventKind::Automation, "automation_loop", msg.clone(), None, None).await;
            }
            // Dispatch webhooks for rules that have notify_url and produced notifications
            if !notifications.is_empty() {
                for (rule_name, notify_url, _) in triggered {
                    if let Some(url) = notify_url {
                        let rn = rule_name.clone();
                        let msgs = notifications.clone();
                        tokio::spawn(async move {
                            for msg in &msgs {
                                crate::infrastructure::webhook::dispatch_webhook(&url, &rn, msg).await;
                            }
                        });
                    }
                }
            }
            record_event(&state, EventKind::Automation, "automation_loop", "rules evaluated".to_string(), None, None).await;
        }
    });
}

/// Fire a single named rule's action immediately (used by webhook endpoint).
/// Returns the notification messages (if the action is Notify) or an empty vec.
pub async fn fire_rule(state: &AppState, rule_name: &str) -> Result<Vec<String>, String> {
    let (action, safe_mode, notify_url) = {
        let automation = state.automation.read().await;
        let key = rule_name.to_lowercase();
        automation.rules.get(&key)
            .map(|r| (r.action.clone(), r.safe_mode, r.notify_url.clone()))
            .ok_or_else(|| format!("rule '{}' not found", rule_name))?
    };
    let notifications = {
        let mut home = state.home.write().await;
        let actions = [action];
        AutomationEngine::execute_actions(&actions, &mut home, safe_mode)
    };
    if let Some(warning) = notifications.first() {
        // Notify actions — emit SSE
        record_event(state, EventKind::Automation, "webhook", warning.clone(), None, None).await;
    }
    if !notifications.is_empty()
        && let Some(url) = notify_url
    {
        let rn = rule_name.to_string();
        let msgs = notifications.clone();
        tokio::spawn(async move {
            for msg in &msgs {
                crate::infrastructure::webhook::dispatch_webhook(&url, &rn, msg).await;
            }
        });
    }
    Ok(notifications)
}
