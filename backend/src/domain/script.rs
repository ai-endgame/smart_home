use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::domain::error::DomainError;

/// A single step in a script.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ScriptStep {
    SetState   { device_name: String, state: String },
    SetBrightness { device_name: String, brightness: Value },
    SetTemperature { device_name: String, temperature: Value },
    /// Sleep for up to 60 000 ms.
    Delay      { milliseconds: u64 },
    ApplyScene { scene_name: String },
    CallScript { script_name: String, #[serde(default)] args: HashMap<String, Value> },
}

/// Definition of an input parameter accepted by a script.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptParam {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub default: Option<Value>,
}

/// A named, parameterised script composed of ordered steps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Script {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub params: Vec<ScriptParam>,
    pub steps: Vec<ScriptStep>,
}

impl Script {
    pub fn new(name: &str, description: &str, params: Vec<ScriptParam>, steps: Vec<ScriptStep>) -> Self {
        Script {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            description: description.to_string(),
            params,
            steps,
        }
    }
}

/// In-memory registry of scripts, keyed by lowercase name.
pub struct ScriptRegistry {
    by_id: HashMap<String, Script>,
    /// name_key → id
    by_name: HashMap<String, String>,
}

impl ScriptRegistry {
    pub fn new() -> Self {
        ScriptRegistry { by_id: HashMap::new(), by_name: HashMap::new() }
    }

    pub fn add(&mut self, script: Script) -> Result<(), DomainError> {
        let name_key = script.name.to_lowercase();
        if self.by_name.contains_key(&name_key) {
            return Err(DomainError::AlreadyExists(format!("Script '{}' already exists.", script.name)));
        }
        self.by_name.insert(name_key, script.id.clone());
        self.by_id.insert(script.id.clone(), script);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&Script> {
        self.by_id.get(id)
    }

    pub fn get_by_name(&self, name: &str) -> Option<&Script> {
        let id = self.by_name.get(&name.to_lowercase())?;
        self.by_id.get(id)
    }

    pub fn update(&mut self, id: &str, updated: Script) -> Result<(), DomainError> {
        let old = self.by_id.get(id)
            .ok_or_else(|| DomainError::NotFound(format!("Script '{}' not found.", id)))?;
        let old_name_key = old.name.to_lowercase();
        let new_name_key = updated.name.to_lowercase();
        if old_name_key != new_name_key {
            if self.by_name.contains_key(&new_name_key) {
                return Err(DomainError::AlreadyExists(format!("Script '{}' already exists.", updated.name)));
            }
            self.by_name.remove(&old_name_key);
            self.by_name.insert(new_name_key, id.to_string());
        }
        self.by_id.insert(id.to_string(), updated);
        Ok(())
    }

    pub fn remove(&mut self, id: &str) -> Result<Script, DomainError> {
        let script = self.by_id.remove(id)
            .ok_or_else(|| DomainError::NotFound(format!("Script '{}' not found.", id)))?;
        self.by_name.remove(&script.name.to_lowercase());
        Ok(script)
    }

    pub fn list(&self) -> Vec<&Script> {
        let mut v: Vec<&Script> = self.by_id.values().collect();
        v.sort_by(|a, b| a.name.cmp(&b.name));
        v
    }
}

impl Default for ScriptRegistry {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_script(name: &str) -> Script {
        Script::new(name, "", vec![], vec![ScriptStep::Delay { milliseconds: 100 }])
    }

    #[test]
    fn duplicate_name_rejected() {
        let mut reg = ScriptRegistry::new();
        reg.add(make_script("dim_all")).unwrap();
        let err = reg.add(make_script("dim_all")).unwrap_err();
        assert!(matches!(err, DomainError::AlreadyExists(_)));
    }

    #[test]
    fn case_insensitive_lookup() {
        let mut reg = ScriptRegistry::new();
        reg.add(make_script("Night Mode")).unwrap();
        assert!(reg.get_by_name("night mode").is_some());
        assert!(reg.get_by_name("NIGHT MODE").is_some());
        assert!(reg.get_by_name("Night Mode").is_some());
    }

    #[test]
    fn remove_succeeds() {
        let mut reg = ScriptRegistry::new();
        let s = make_script("test");
        let id = s.id.clone();
        reg.add(s).unwrap();
        reg.remove(&id).unwrap();
        assert!(reg.get(&id).is_none());
        assert!(reg.get_by_name("test").is_none());
    }
}
