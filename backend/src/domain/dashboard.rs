use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::error::DomainError;

// ── Card content ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "card_type", rename_all = "snake_case")]
pub enum CardContent {
    EntityCard { entity_id: String },
    GaugeCard { entity_id: String, min: f64, max: f64, #[serde(skip_serializing_if = "Option::is_none")] unit: Option<String> },
    ButtonCard { entity_id: String, action: String },
    StatCard { title: String, entity_ids: Vec<String>, aggregation: String },
    HistoryCard { entity_id: String, hours: u32 },
}

// ── Card ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(flatten)]
    pub content: CardContent,
}

impl Card {
    pub fn new(content: CardContent) -> Self {
        Card { id: Uuid::new_v4().to_string(), title: None, content }
    }
}

// ── View ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct View {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    pub cards: Vec<Card>,
}

impl View {
    pub fn new(title: &str, icon: Option<String>) -> Self {
        View { id: Uuid::new_v4().to_string(), title: title.to_string(), icon, cards: Vec::new() }
    }
}

// ── Dashboard ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dashboard {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    pub views: Vec<View>,
    pub created_at: DateTime<Utc>,
}

impl Dashboard {
    pub fn new(name: &str, icon: Option<String>) -> Self {
        Dashboard {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            icon,
            views: Vec::new(),
            created_at: Utc::now(),
        }
    }
}

// ── DashboardRegistry ─────────────────────────────────────────────────────────

pub struct DashboardRegistry {
    by_id: HashMap<String, Dashboard>,
    /// name_key (lowercased) → id
    by_name: HashMap<String, String>,
}

impl DashboardRegistry {
    pub fn new() -> Self {
        DashboardRegistry { by_id: HashMap::new(), by_name: HashMap::new() }
    }

    pub fn add(&mut self, dashboard: Dashboard) -> Result<(), DomainError> {
        let name_key = dashboard.name.to_lowercase();
        if self.by_name.contains_key(&name_key) {
            return Err(DomainError::AlreadyExists(format!("Dashboard '{}' already exists.", dashboard.name)));
        }
        self.by_name.insert(name_key, dashboard.id.clone());
        self.by_id.insert(dashboard.id.clone(), dashboard);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&Dashboard> {
        self.by_id.get(id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut Dashboard> {
        self.by_id.get_mut(id)
    }

    pub fn remove(&mut self, id: &str) -> Result<Dashboard, DomainError> {
        let dashboard = self.by_id.remove(id)
            .ok_or_else(|| DomainError::NotFound(format!("Dashboard '{}' not found.", id)))?;
        self.by_name.remove(&dashboard.name.to_lowercase());
        Ok(dashboard)
    }

    pub fn list(&self) -> Vec<&Dashboard> {
        let mut v: Vec<&Dashboard> = self.by_id.values().collect();
        v.sort_by(|a, b| a.name.cmp(&b.name));
        v
    }
}

impl Default for DashboardRegistry {
    fn default() -> Self { Self::new() }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_dashboard_has_unique_id() {
        let d = Dashboard::new("Home", None);
        assert!(!d.id.is_empty());
        assert_eq!(d.name, "Home");
        assert!(d.views.is_empty());
    }

    #[test]
    fn duplicate_name_rejected() {
        let mut reg = DashboardRegistry::new();
        reg.add(Dashboard::new("Home", None)).unwrap();
        let err = reg.add(Dashboard::new("home", None)).unwrap_err();
        assert!(matches!(err, DomainError::AlreadyExists(_)));
    }

    #[test]
    fn remove_returns_dashboard() {
        let mut reg = DashboardRegistry::new();
        let d = Dashboard::new("Test", None);
        let id = d.id.clone();
        reg.add(d).unwrap();
        let removed = reg.remove(&id).unwrap();
        assert_eq!(removed.id, id);
        assert!(reg.get(&id).is_none());
    }

    #[test]
    fn card_round_trips_entity_card() {
        let content = CardContent::EntityCard { entity_id: "device.lamp.switch".to_string() };
        let json = serde_json::to_string(&content).unwrap();
        assert!(json.contains("\"card_type\":\"entity_card\""));
        let back: CardContent = serde_json::from_str(&json).unwrap();
        assert_eq!(back, content);
    }

    #[test]
    fn card_round_trips_gauge_card() {
        let content = CardContent::GaugeCard { entity_id: "device.thermo.sensor".to_string(), min: 0.0, max: 100.0, unit: Some("°C".to_string()) };
        let json = serde_json::to_string(&content).unwrap();
        let back: CardContent = serde_json::from_str(&json).unwrap();
        assert_eq!(back, content);
    }

    #[test]
    fn card_round_trips_button_card() {
        let content = CardContent::ButtonCard { entity_id: "device.lamp.switch".to_string(), action: "toggle".to_string() };
        let json = serde_json::to_string(&content).unwrap();
        let back: CardContent = serde_json::from_str(&json).unwrap();
        assert_eq!(back, content);
    }

    #[test]
    fn card_round_trips_stat_card() {
        let content = CardContent::StatCard { title: "Lights On".to_string(), entity_ids: vec!["a".to_string()], aggregation: "count".to_string() };
        let json = serde_json::to_string(&content).unwrap();
        let back: CardContent = serde_json::from_str(&json).unwrap();
        assert_eq!(back, content);
    }

    #[test]
    fn card_round_trips_history_card() {
        let content = CardContent::HistoryCard { entity_id: "device.thermo.sensor".to_string(), hours: 24 };
        let json = serde_json::to_string(&content).unwrap();
        let back: CardContent = serde_json::from_str(&json).unwrap();
        assert_eq!(back, content);
    }
}
