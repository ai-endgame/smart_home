use axum::{Json, extract::{Path, State}};
use chrono::Local;

use crate::domain::AutomationEngine;
use crate::http::{
    errors::ApiError,
    helpers::{action_to_response, map_common_error, map_create_error, record_event, rule_to_response, validate_name},
    types::{ActionInput, AddRuleRequest, AutomationRunResponse, EventKind, MessageResponse, RuleResponse, WebhookFireResponse},
};
use crate::infrastructure::automation_loop;
use crate::state::AppState;

pub async fn list_rules(State(state): State<AppState>) -> Json<Vec<RuleResponse>> {
    let automation = state.automation.read().await;
    Json(automation.list_rules().into_iter().map(rule_to_response).collect())
}

pub async fn add_rule(
    State(state): State<AppState>,
    Json(payload): Json<AddRuleRequest>,
) -> Result<Json<RuleResponse>, ApiError> {
    let name = payload.name.trim().to_string();
    if name.is_empty() {
        return Err(ApiError::BadRequest("rule name cannot be empty or whitespace".to_string()));
    }
    validate_name(&name)?;
    let trigger = payload.trigger.clone().to_domain()?;
    let action = payload.action.clone().to_domain()?;
    let time_range = payload.time_range.as_ref().map(|tr| (tr.from.clone(), tr.to.clone()));
    let conditions = payload.conditions.iter().map(|c| c.clone().to_domain()).collect::<Result<Vec<_>, _>>()?;
    let notify_url = payload.notify_url.clone();
    {
        let mut eng = state.automation.write().await;
        eng.add_rule(&name, trigger, action, time_range, conditions).map_err(map_create_error)?;
        if let (Some(url), Some(rule)) = (&notify_url, eng.rules.get_mut(&name.to_lowercase())) {
            rule.notify_url = Some(url.clone());
        }
    }
    record_event(&state, EventKind::Automation, "automation_rule", format!("rule '{}' added", name), None, None).await;
    Ok(Json(RuleResponse { name, enabled: true, safe_mode: false, trigger: payload.trigger, action: payload.action, time_range: payload.time_range, conditions: payload.conditions, notify_url }))
}

pub async fn remove_rule(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<MessageResponse>, ApiError> {
    { state.automation.write().await.remove_rule(&name).map_err(map_common_error)?; }
    record_event(&state, EventKind::Automation, "automation_rule", format!("rule '{}' removed", name), None, None).await;
    Ok(Json(MessageResponse { message: format!("rule '{}' removed", name) }))
}

pub async fn toggle_rule(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<RuleResponse>, ApiError> {
    let enabled = { state.automation.write().await.toggle_rule(&name).map_err(map_common_error)? };
    let rule = {
        let automation = state.automation.read().await;
        automation.list_rules().into_iter()
            .find(|r| r.name.eq_ignore_ascii_case(&name))
            .map(rule_to_response)
            .ok_or_else(|| ApiError::NotFound(format!("rule '{}' not found", name)))?
    };
    let status = if enabled { "enabled" } else { "disabled" };
    record_event(&state, EventKind::Automation, "automation_rule", format!("rule '{}' {}", name, status), None, None).await;
    Ok(Json(rule))
}

pub async fn run_automation(State(state): State<AppState>) -> Result<Json<AutomationRunResponse>, ApiError> {
    use crate::domain::automation::Action;
    use crate::infrastructure::{script_executor, webhook};
    let now = Local::now().time();
    let triggered = {
        let home = state.home.read().await;
        let automation = state.automation.read().await;
        let presence = state.presence.read().await;
        automation.evaluate_rules_with_meta(&home, &presence, now)
    };
    let actions: Vec<Action> = triggered.iter().map(|(_, _, a)| a.clone()).collect();
    let notifications = {
        let mut home = state.home.write().await;
        AutomationEngine::execute_actions(&actions, &mut home, false)
    };
    // Handle ScriptCall actions
    for action in &actions {
        if let Action::ScriptCall { script_name, args } = action {
            let script_opt = {
                let scripts = state.scripts.read().await;
                scripts.get_by_name(script_name).cloned()
            };
            if let Some(script) = script_opt {
                let state_clone = state.clone();
                let args_clone = args.clone();
                tokio::spawn(async move {
                    script_executor::run_script(script, args_clone, state_clone, 0).await;
                });
            }
        }
    }
    for msg in &notifications {
        record_event(&state, EventKind::Automation, "automation_engine", msg.clone(), None, None).await;
    }
    // Dispatch webhooks for triggered rules with notify_url
    if !notifications.is_empty() {
        for (rule_name, notify_url, _) in &triggered {
            if let Some(url) = notify_url {
                let url = url.clone();
                let rn = rule_name.clone();
                let msgs = notifications.clone();
                tokio::spawn(async move {
                    for msg in &msgs {
                        webhook::dispatch_webhook(&url, &rn, msg).await;
                    }
                });
            }
        }
    }
    record_event(&state, EventKind::Automation, "automation_engine", format!("{} action(s) executed", actions.len()), None, None).await;
    let action_responses: Vec<ActionInput> = actions.iter().map(action_to_response).collect();
    Ok(Json(AutomationRunResponse { actions_executed: action_responses.len(), actions: action_responses }))
}

pub async fn toggle_safe_mode(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<axum::Json<serde_json::Value>, ApiError> {
    use serde_json::json;
    let new_val = { state.automation.write().await.toggle_safe_mode(&name).map_err(map_common_error)? };
    record_event(&state, EventKind::Automation, "automation_rule", format!("rule '{}' safe_mode={}", name, new_val), None, None).await;
    Ok(axum::Json(json!({ "name": name, "safe_mode": new_val })))
}

pub async fn webhook_trigger(
    State(state): State<AppState>,
    Path(rule_name): Path<String>,
) -> Result<Json<WebhookFireResponse>, ApiError> {
    // Validate the rule exists and has a Webhook trigger
    {
        use crate::domain::automation::Trigger;
        let automation = state.automation.read().await;
        let key = rule_name.to_lowercase();
        let rule = automation.rules.get(&key)
            .ok_or_else(|| ApiError::NotFound(format!("rule '{}' not found", rule_name)))?;
        if !matches!(rule.trigger, Trigger::Webhook { .. }) {
            return Err(ApiError::BadRequest(format!(
                "rule '{}' does not have a webhook trigger", rule_name
            )));
        }
    }
    let notifications = automation_loop::fire_rule(&state, &rule_name).await
        .map_err(ApiError::NotFound)?;
    let msg = notifications.first().cloned().unwrap_or_else(|| format!("rule '{}' fired", rule_name));
    record_event(&state, EventKind::Automation, "webhook", format!("rule '{}' triggered via webhook", rule_name), None, None).await;
    Ok(Json(WebhookFireResponse { rule_name, action_executed: true, message: msg }))
}
