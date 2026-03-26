use std::collections::HashMap;
use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::error::DomainError;

/// Raw evidence from a single source.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceState {
    Home,
    Away,
    Unknown,
}

impl fmt::Display for SourceState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SourceState::Home    => write!(f, "home"),
            SourceState::Away    => write!(f, "away"),
            SourceState::Unknown => write!(f, "unknown"),
        }
    }
}

/// Aggregated presence state for a person.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PresenceState {
    Home,
    Away,
    Unknown,
}

impl fmt::Display for PresenceState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PresenceState::Home    => write!(f, "home"),
            PresenceState::Away    => write!(f, "away"),
            PresenceState::Unknown => write!(f, "unknown"),
        }
    }
}

/// A person being tracked with multi-source evidence aggregation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonTracker {
    pub id: String,
    pub name: String,
    /// Grace period before `away` is committed after all sources go away (seconds).
    pub grace_period_secs: u32,
    /// source_name → source state
    pub sources: HashMap<String, SourceState>,
    /// Set when all sources transitioned to away/unknown; cleared on any home.
    pub away_since: Option<DateTime<Utc>>,
}

impl PersonTracker {
    pub fn new(name: &str, grace_period_secs: u32) -> Self {
        PersonTracker {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            grace_period_secs,
            sources: HashMap::new(),
            away_since: None,
        }
    }

    /// Compute the public-facing presence state at `now`.
    /// "any home wins" — if any source is Home, result is Home.
    /// Otherwise we're away/unknown; grace period delays the Away commitment.
    pub fn effective_state(&self, now: DateTime<Utc>) -> PresenceState {
        // Any source home → immediately home.
        if self.sources.values().any(|s| *s == SourceState::Home) {
            return PresenceState::Home;
        }
        // No sources at all → unknown.
        if self.sources.is_empty() {
            return PresenceState::Unknown;
        }
        // All sources away or unknown.
        match self.away_since {
            None => {
                // away_since not yet set — treat as Home (transition not committed)
                PresenceState::Home
            }
            Some(since) => {
                let elapsed = (now - since).num_seconds() as u64;
                if elapsed >= self.grace_period_secs as u64 {
                    PresenceState::Away
                } else {
                    PresenceState::Home
                }
            }
        }
    }

    /// Update a source and adjust `away_since` accordingly.
    pub fn update_source(&mut self, source: &str, state: SourceState, now: DateTime<Utc>) {
        self.sources.insert(source.to_string(), state);

        // If any source is home, clear away_since.
        if self.sources.values().any(|s| *s == SourceState::Home) {
            self.away_since = None;
            return;
        }
        // All sources away/unknown — set away_since if not already set.
        if self.away_since.is_none() {
            self.away_since = Some(now);
        }
    }
}

/// In-memory registry of tracked persons.
pub struct PresenceRegistry {
    by_id: HashMap<String, PersonTracker>,
    /// name_key (lowercased) → id
    by_name: HashMap<String, String>,
}

impl PresenceRegistry {
    pub fn new() -> Self {
        PresenceRegistry { by_id: HashMap::new(), by_name: HashMap::new() }
    }

    pub fn add(&mut self, person: PersonTracker) -> Result<(), DomainError> {
        let name_key = person.name.to_lowercase();
        if self.by_name.contains_key(&name_key) {
            return Err(DomainError::AlreadyExists(format!("Person '{}' already exists.", person.name)));
        }
        self.by_name.insert(name_key, person.id.clone());
        self.by_id.insert(person.id.clone(), person);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&PersonTracker> {
        self.by_id.get(id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut PersonTracker> {
        self.by_id.get_mut(id)
    }

    pub fn get_by_name(&self, name: &str) -> Option<&PersonTracker> {
        let id = self.by_name.get(&name.to_lowercase())?;
        self.by_id.get(id)
    }

    pub fn remove(&mut self, id: &str) -> Result<PersonTracker, DomainError> {
        let person = self.by_id.remove(id)
            .ok_or_else(|| DomainError::NotFound(format!("Person '{}' not found.", id)))?;
        self.by_name.remove(&person.name.to_lowercase());
        Ok(person)
    }

    pub fn list(&self) -> Vec<&PersonTracker> {
        let mut v: Vec<&PersonTracker> = self.by_id.values().collect();
        v.sort_by(|a, b| a.name.cmp(&b.name));
        v
    }

    /// Update a source on a person by id. Returns the updated tracker.
    pub fn update_source(
        &mut self,
        id: &str,
        source: &str,
        state: SourceState,
        now: DateTime<Utc>,
    ) -> Result<&PersonTracker, DomainError> {
        let person = self.by_id.get_mut(id)
            .ok_or_else(|| DomainError::NotFound(format!("Person '{}' not found.", id)))?;
        person.update_source(source, state, now);
        Ok(&self.by_id[id])
    }
}

impl Default for PresenceRegistry {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> DateTime<Utc> { Utc::now() }
    fn past(secs: i64) -> DateTime<Utc> { Utc::now() - chrono::Duration::seconds(secs) }

    #[test]
    fn new_person_is_unknown() {
        let p = PersonTracker::new("alice", 120);
        assert_eq!(p.effective_state(now()), PresenceState::Unknown);
    }

    #[test]
    fn any_home_source_wins() {
        let mut p = PersonTracker::new("alice", 120);
        p.sources.insert("network".to_string(), SourceState::Away);
        p.sources.insert("ble".to_string(), SourceState::Home);
        assert_eq!(p.effective_state(now()), PresenceState::Home);
    }

    #[test]
    fn grace_period_not_elapsed_stays_home() {
        let mut p = PersonTracker::new("alice", 120);
        p.sources.insert("network".to_string(), SourceState::Away);
        p.away_since = Some(past(60)); // 60s ago, grace = 120s
        assert_eq!(p.effective_state(now()), PresenceState::Home);
    }

    #[test]
    fn grace_period_elapsed_becomes_away() {
        let mut p = PersonTracker::new("alice", 120);
        p.sources.insert("network".to_string(), SourceState::Away);
        p.away_since = Some(past(150)); // 150s ago, grace = 120s
        assert_eq!(p.effective_state(now()), PresenceState::Away);
    }

    #[test]
    fn duplicate_name_rejected() {
        let mut reg = PresenceRegistry::new();
        reg.add(PersonTracker::new("Alice", 120)).unwrap();
        let err = reg.add(PersonTracker::new("alice", 60)).unwrap_err();
        assert!(matches!(err, DomainError::AlreadyExists(_)));
    }

    #[test]
    fn update_source_clears_away_since_on_home() {
        let mut p = PersonTracker::new("alice", 120);
        p.sources.insert("network".to_string(), SourceState::Away);
        p.away_since = Some(past(200));
        p.update_source("network", SourceState::Home, now());
        assert!(p.away_since.is_none());
        assert_eq!(p.effective_state(now()), PresenceState::Home);
    }

    #[test]
    fn update_source_sets_away_since_when_all_away() {
        let mut p = PersonTracker::new("alice", 120);
        p.sources.insert("network".to_string(), SourceState::Home);
        p.sources.insert("ble".to_string(), SourceState::Away);
        // Both need to be away for away_since to set
        p.update_source("network", SourceState::Away, now());
        assert!(p.away_since.is_some());
    }
}
