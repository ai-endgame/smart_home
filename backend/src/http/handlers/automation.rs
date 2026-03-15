use axum::{Json, extract::{Path, State}};

use crate::domain::AutomationEngine;
use crate::http::{
    errors::ApiError,
    helpers::{action_to_response, map_common_error, map_create_error, record_event, rule_to_response},
    types::{ActionInput, AddRuleRequest, AutomationRunResponse, EventKind, MessageResponse, RuleResponse},
};
use crate::state::AppState;

pub async fn list_rules(State(state): State<AppState>) -> Json<Vec<RuleResponse>> {
    let automation = state.automation.read().await;
    Json(automation.list_rules().iter().map(rule_to_response).collect())
}

pub async fn add_rule(
    State(state): State<AppState>,
    Json(payload): Json<AddRuleRequest>,
) -> Result<Json<RuleResponse>, ApiError> {
    let trigger = payload.trigger.clone().to_domain()?;
    let action = payload.action.clone().to_domain()?;
    { state.automation.write().await.add_rule(&payload.name, trigger, action).map_err(map_create_error)?; }
    record_event(&state, EventKind::Automation, "automation_rule", format!("rule '{}' added", payload.name), None, None).await;
    Ok(Json(RuleResponse { name: payload.name, enabled: true, trigger: payload.trigger, action: payload.action }))
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
        automation.list_rules().iter()
            .find(|r| r.name.eq_ignore_ascii_case(&name))
            .map(rule_to_response)
            .ok_or_else(|| ApiError::NotFound(format!("rule '{}' not found", name)))?
    };
    let status = if enabled { "enabled" } else { "disabled" };
    record_event(&state, EventKind::Automation, "automation_rule", format!("rule '{}' {}", name, status), None, None).await;
    Ok(Json(rule))
}

pub async fn run_automation(State(state): State<AppState>) -> Result<Json<AutomationRunResponse>, ApiError> {
    let actions = {
        let home = state.home.read().await;
        let automation = state.automation.read().await;
        automation.evaluate_rules(&home)
    };
    { let mut home = state.home.write().await; AutomationEngine::execute_actions(&actions, &mut home); }
    record_event(&state, EventKind::Automation, "automation_engine", format!("{} action(s) executed", actions.len()), None, None).await;
    let action_responses: Vec<ActionInput> = actions.iter().map(action_to_response).collect();
    Ok(Json(AutomationRunResponse { actions_executed: action_responses.len(), actions: action_responses }))
}
