use std::collections::BTreeMap;
use std::collections::HashMap;

use log::{debug, info, warn};

use crate::domain::device::{Area, Device, DeviceState, DeviceType};
use crate::domain::error::DomainError;

/// Central manager for all smart home devices and areas.
pub struct SmartHome {
    pub devices: BTreeMap<String, Device>,
    pub areas: HashMap<String, Area>,
    /// Reverse index: device UUID → lowercased name key.
    devices_by_id: HashMap<String, String>,
}

impl SmartHome {
    pub fn new() -> Self {
        SmartHome {
            devices: BTreeMap::new(),
            areas: HashMap::new(),
            devices_by_id: HashMap::new(),
        }
    }

    // ── Device operations ───────────────────────────────────────────

    /// Insert a device that already has a UUID (used when loading from the database).
    pub fn insert_device(&mut self, device: Device) {
        let key = device.name.to_lowercase();
        self.devices_by_id.insert(device.id.clone(), key.clone());
        self.devices.insert(key, device);
    }

    pub fn add_device(&mut self, name: &str, device_type: DeviceType) -> Result<String, DomainError> {
        let key = name.to_lowercase();
        if self.devices.contains_key(&key) {
            warn!("Attempted to add duplicate device: {}", name);
            return Err(DomainError::AlreadyExists(format!("Device '{}' already exists.", name)));
        }
        let device = Device::new(name, device_type.clone());
        let id = device.id.clone();
        self.devices_by_id.insert(id.clone(), key.clone());
        self.devices.insert(key, device);
        info!("Added device '{}' (type: {:?}, id: {})", name, device_type, id.get(..8).unwrap_or(&id));
        Ok(id)
    }

    pub fn remove_device(&mut self, name: &str) -> Result<(), DomainError> {
        let key = name.to_lowercase();
        match self.devices.remove(&key) {
            Some(device) => {
                info!("Removed device '{}' (id: {})", device.name, device.id.get(..8).unwrap_or(&device.id));
                self.devices_by_id.remove(&device.id);
                if let Some(room_name) = &device.room {
                    let room_key = room_name.to_lowercase();
                    if let Some(area) = self.areas.get_mut(&room_key) {
                        area.device_ids.retain(|id| id != &device.id);
                        debug!("Removed device from area '{}'", room_name);
                    }
                }
                Ok(())
            }
            None => {
                warn!("Attempted to remove non-existent device: {}", name);
                Err(DomainError::NotFound(format!("Device '{}' not found.", name)))
            }
        }
    }

    pub fn get_device(&self, name: &str) -> Option<&Device> {
        self.devices.get(&name.to_lowercase())
    }

    pub fn get_device_mut(&mut self, name: &str) -> Option<&mut Device> {
        self.devices.get_mut(&name.to_lowercase())
    }

    pub fn set_state(&mut self, name: &str, state: DeviceState) -> Result<(), DomainError> {
        match self.get_device_mut(name) {
            Some(device) => {
                debug!("Setting device '{}' state to {:?}", name, state);
                device.state = state;
                Ok(())
            }
            None => {
                warn!("Attempted to set state on non-existent device: {}", name);
                Err(DomainError::NotFound(format!("Device '{}' not found.", name)))
            }
        }
    }

    pub fn set_brightness(&mut self, name: &str, brightness: u8) -> Result<(), DomainError> {
        let key = name.to_lowercase();
        match self.devices.get_mut(&key) {
            Some(device) => {
                if device.device_type != DeviceType::Light {
                    return Err(DomainError::InvalidOperation(format!("'{}' is not a Light.", name)));
                }
                if brightness > 100 {
                    return Err(DomainError::InvalidOperation("Brightness must be 0–100.".to_string()));
                }
                device.brightness = brightness;
                Ok(())
            }
            None => Err(DomainError::NotFound(format!("Device '{}' not found.", name))),
        }
    }

    pub fn set_temperature(&mut self, name: &str, temp: f64) -> Result<(), DomainError> {
        let key = name.to_lowercase();
        match self.devices.get_mut(&key) {
            Some(device) => {
                if device.device_type != DeviceType::Thermostat {
                    return Err(DomainError::InvalidOperation(format!("'{}' is not a Thermostat.", name)));
                }
                device.temperature = Some(temp);
                Ok(())
            }
            None => Err(DomainError::NotFound(format!("Device '{}' not found.", name))),
        }
    }

    pub fn connect_device(&mut self, name: &str) -> Result<(), DomainError> {
        match self.get_device_mut(name) {
            Some(device) => { device.connected = true; Ok(()) }
            None => Err(DomainError::NotFound(format!("Device '{}' not found.", name))),
        }
    }

    pub fn disconnect_device(&mut self, name: &str) -> Result<(), DomainError> {
        match self.get_device_mut(name) {
            Some(device) => { device.connected = false; Ok(()) }
            None => Err(DomainError::NotFound(format!("Device '{}' not found.", name))),
        }
    }

    pub fn set_device_error(&mut self, name: &str, message: String) -> Result<(), DomainError> {
        match self.get_device_mut(name) {
            Some(device) => { device.last_error = Some(message); Ok(()) }
            None => Err(DomainError::NotFound(format!("Device '{}' not found.", name))),
        }
    }

    pub fn clear_device_error(&mut self, name: &str) -> Result<(), DomainError> {
        match self.get_device_mut(name) {
            Some(device) => { device.last_error = None; Ok(()) }
            None => Err(DomainError::NotFound(format!("Device '{}' not found.", name))),
        }
    }

    pub fn list_devices(&self) -> Vec<&Device> {
        // BTreeMap iterates in sorted order — no extra sort needed.
        self.devices.values().collect()
    }

    // ── Area operations ─────────────────────────────────────────────

    pub fn add_room(&mut self, name: &str) -> Result<(), DomainError> {
        let key = name.to_lowercase();
        if self.areas.contains_key(&key) {
            warn!("Attempted to add duplicate area: {}", name);
            return Err(DomainError::AlreadyExists(format!("Area '{}' already exists.", name)));
        }
        self.areas.insert(key, Area::new(name));
        info!("Added area '{}'", name);
        Ok(())
    }

    pub fn assign_device_to_room(&mut self, device_name: &str, room_name: &str) -> Result<(), DomainError> {
        let device_key = device_name.to_lowercase();
        let room_key = room_name.to_lowercase();
        if !self.areas.contains_key(&room_key) {
            return Err(DomainError::NotFound(format!("Area '{}' not found.", room_name)));
        }
        let device = self
            .devices
            .get_mut(&device_key)
            .ok_or_else(|| DomainError::NotFound(format!("Device '{}' not found.", device_name)))?;
        if let Some(old_room_name) = &device.room {
            let old_key = old_room_name.to_lowercase();
            if let Some(old_area) = self.areas.get_mut(&old_key) {
                old_area.device_ids.retain(|id| id != &device.id);
            }
        }
        let device_id = device.id.clone();
        device.room = Some(room_name.to_string());
        let area = self.areas.get_mut(&room_key)
            .expect("area must exist: checked above");
        area.device_ids.push(device_id);
        Ok(())
    }

    pub fn list_rooms(&self) -> Vec<&Area> {
        let mut areas: Vec<&Area> = self.areas.values().collect();
        areas.sort_by(|a, b| a.name.cmp(&b.name));
        areas
    }

    /// Expose the device UUID → name-key reverse index (read-only).
    pub fn devices_by_id(&self) -> &HashMap<String, String> {
        &self.devices_by_id
    }

    pub fn get_area(&self, area_id: &str) -> Option<&Area> {
        // area_id is the slug; areas are keyed by lowercase name — try slug match
        self.areas.values().find(|a| a.area_id == area_id)
    }

    pub fn get_room_devices(&self, room_name: &str) -> Vec<&Device> {
        let room_key = room_name.to_lowercase();
        match self.areas.get(&room_key) {
            Some(area) => area
                .device_ids
                .iter()
                .filter_map(|id| {
                    self.devices_by_id
                        .get(id)
                        .and_then(|key| self.devices.get(key))
                })
                .collect(),
            None => Vec::new(),
        }
    }
}

impl Default for SmartHome {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_get_device() {
        let mut home = SmartHome::new();
        let id = home.add_device("Desk Lamp", DeviceType::Light).unwrap();
        assert!(!id.is_empty());
        let device = home.get_device("desk lamp").unwrap();
        assert_eq!(device.name, "Desk Lamp");
        assert_eq!(device.state, DeviceState::Off);
    }

    #[test]
    fn test_duplicate_device() {
        let mut home = SmartHome::new();
        home.add_device("Lamp", DeviceType::Light).unwrap();
        assert!(home.add_device("lamp", DeviceType::Light).is_err());
    }

    #[test]
    fn test_remove_device() {
        let mut home = SmartHome::new();
        home.add_device("Lamp", DeviceType::Light).unwrap();
        assert!(home.remove_device("lamp").is_ok());
        assert!(home.get_device("lamp").is_none());
    }

    #[test]
    fn test_rooms_and_assignment() {
        let mut home = SmartHome::new();
        home.add_room("Kitchen").unwrap();
        home.add_device("Lamp", DeviceType::Light).unwrap();
        home.assign_device_to_room("Lamp", "Kitchen").unwrap();
        assert_eq!(home.get_device("lamp").unwrap().room.as_deref(), Some("Kitchen"));
        assert_eq!(home.get_room_devices("Kitchen").len(), 1);
    }

    #[test]
    fn test_missing_entities() {
        let mut home = SmartHome::new();
        assert!(home.set_state("missing", DeviceState::On).is_err());
        assert!(home.set_brightness("missing", 10).is_err());
        assert!(home.set_temperature("missing", 10.0).is_err());
        assert!(home.connect_device("missing").is_err());
        assert!(home.disconnect_device("missing").is_err());
        assert!(home.set_device_error("missing", "x".to_string()).is_err());
        assert!(home.clear_device_error("missing").is_err());
    }

    #[test]
    fn test_default_impl() {
        let home = SmartHome::default();
        assert!(home.devices.is_empty());
    }

    #[test]
    fn test_area_slug_derived() {
        let area = crate::domain::device::Area::new("Living Room");
        assert_eq!(area.area_id, "living-room");
        assert_eq!(area.name, "Living Room");
        assert!(area.floor.is_none());
    }

    #[test]
    fn test_area_operations() {
        let mut home = SmartHome::new();
        home.add_room("Kitchen").unwrap();
        // duplicate rejected
        assert!(home.add_room("kitchen").is_err());
        // list_rooms returns sorted
        home.add_room("Attic").unwrap();
        let names: Vec<&str> = home.list_rooms().iter().map(|a| a.name.as_str()).collect();
        assert_eq!(names, vec!["Attic", "Kitchen"]);
        // get_area by slug
        assert!(home.get_area("kitchen").is_some());
        assert!(home.get_area("nonexistent").is_none());
    }
}
