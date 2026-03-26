use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::device::DeviceState;
use crate::domain::error::DomainError;
use crate::domain::manager::SmartHome;

/// The device state snapshot stored inside a scene.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneState {
    pub state: Option<DeviceState>,
    pub brightness: Option<u8>,
    pub temperature: Option<f64>,
}

/// A named snapshot of device states.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    pub id: String,
    pub name: String,
    /// device_id → target state
    pub states: HashMap<String, SceneState>,
}

impl Scene {
    pub fn new(name: &str, states: HashMap<String, SceneState>) -> Self {
        Scene { id: Uuid::new_v4().to_string(), name: name.to_string(), states }
    }
}

/// In-memory registry of scenes.
pub struct SceneRegistry {
    by_id: HashMap<String, Scene>,
    /// name_key → id
    by_name: HashMap<String, String>,
}

impl SceneRegistry {
    pub fn new() -> Self {
        SceneRegistry { by_id: HashMap::new(), by_name: HashMap::new() }
    }

    pub fn add(&mut self, scene: Scene) -> Result<(), DomainError> {
        let name_key = scene.name.to_lowercase();
        if self.by_name.contains_key(&name_key) {
            return Err(DomainError::AlreadyExists(format!("Scene '{}' already exists.", scene.name)));
        }
        self.by_name.insert(name_key, scene.id.clone());
        self.by_id.insert(scene.id.clone(), scene);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&Scene> {
        self.by_id.get(id)
    }

    pub fn get_by_name(&self, name: &str) -> Option<&Scene> {
        let id = self.by_name.get(&name.to_lowercase())?;
        self.by_id.get(id)
    }

    pub fn update(&mut self, id: &str, states: HashMap<String, SceneState>) -> Result<Scene, DomainError> {
        let scene = self.by_id.get_mut(id)
            .ok_or_else(|| DomainError::NotFound(format!("Scene '{}' not found.", id)))?;
        scene.states = states;
        Ok(scene.clone())
    }

    pub fn remove(&mut self, id: &str) -> Result<Scene, DomainError> {
        let scene = self.by_id.remove(id)
            .ok_or_else(|| DomainError::NotFound(format!("Scene '{}' not found.", id)))?;
        self.by_name.remove(&scene.name.to_lowercase());
        Ok(scene)
    }

    pub fn list(&self) -> Vec<&Scene> {
        let mut v: Vec<&Scene> = self.by_id.values().collect();
        v.sort_by(|a, b| a.name.cmp(&b.name));
        v
    }

    /// Apply a scene to SmartHome. Returns (applied_count, errors).
    /// Partial failures are collected; remaining devices still proceed.
    pub fn apply(scene: &Scene, home: &mut SmartHome) -> (usize, Vec<String>) {
        let mut applied = 0usize;
        let mut errors = Vec::new();

        for (device_id, target) in &scene.states {
            // Find device by id (via the reverse index in SmartHome)
            let name_key = match home.devices_by_id().get(device_id) {
                Some(k) => k.clone(),
                None => {
                    errors.push(format!("device '{}' not found", device_id));
                    continue;
                }
            };
            let mut had_error = false;
            if let Some(state) = &target.state
                && home.set_state(&name_key, state.clone()).is_err() {
                    errors.push(format!("failed to set state on '{}'", device_id));
                    had_error = true;
                }
            if let Some(brightness) = target.brightness
                && home.set_brightness(&name_key, brightness).is_err() {
                    errors.push(format!("failed to set brightness on '{}'", device_id));
                    had_error = true;
                }
            if let Some(temp) = target.temperature
                && home.set_temperature(&name_key, temp).is_err() {
                    errors.push(format!("failed to set temperature on '{}'", device_id));
                    had_error = true;
                }
            if !had_error { applied += 1; }
        }
        (applied, errors)
    }
}

impl Default for SceneRegistry {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate_name_rejected() {
        let mut reg = SceneRegistry::new();
        reg.add(Scene::new("Evening", HashMap::new())).unwrap();
        let err = reg.add(Scene::new("evening", HashMap::new())).unwrap_err();
        assert!(matches!(err, DomainError::AlreadyExists(_)));
    }

    #[test]
    fn partial_apply_missing_device() {
        let mut reg = SceneRegistry::new();
        let mut states = HashMap::new();
        states.insert("nonexistent-uuid".to_string(), SceneState {
            state: Some(DeviceState::On),
            brightness: None,
            temperature: None,
        });
        let scene = Scene::new("test", states);
        reg.add(scene.clone()).unwrap();
        let mut home = SmartHome::new();
        let (applied, errors) = SceneRegistry::apply(&scene, &mut home);
        assert_eq!(applied, 0);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("not found"));
    }
}
