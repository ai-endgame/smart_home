use std::collections::HashMap;

use log::{debug, info, warn};

use crate::domain::device::{Device, DeviceState, DeviceType, Room};

/// Central manager for all smart home devices and rooms.
pub struct SmartHome {
    pub devices: HashMap<String, Device>,
    pub rooms: HashMap<String, Room>,
    /// Reverse index: device UUID → lowercased name key.
    devices_by_id: HashMap<String, String>,
}

impl SmartHome {
    pub fn new() -> Self {
        SmartHome {
            devices: HashMap::new(),
            rooms: HashMap::new(),
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

    pub fn add_device(&mut self, name: &str, device_type: DeviceType) -> Result<String, String> {
        let key = name.to_lowercase();
        if self.devices.contains_key(&key) {
            warn!("Attempted to add duplicate device: {}", name);
            return Err(format!("Device '{}' already exists.", name));
        }
        let device = Device::new(name, device_type.clone());
        let id = device.id.clone();
        self.devices_by_id.insert(id.clone(), key.clone());
        self.devices.insert(key, device);
        info!("Added device '{}' (type: {:?}, id: {})", name, device_type, &id[..8]);
        Ok(id)
    }

    pub fn remove_device(&mut self, name: &str) -> Result<(), String> {
        let key = name.to_lowercase();
        match self.devices.remove(&key) {
            Some(device) => {
                info!("Removed device '{}' (id: {})", device.name, &device.id[..8]);
                self.devices_by_id.remove(&device.id);
                if let Some(room_name) = &device.room {
                    let room_key = room_name.to_lowercase();
                    if let Some(room) = self.rooms.get_mut(&room_key) {
                        room.device_ids.retain(|id| id != &device.id);
                        debug!("Removed device from room '{}'", room_name);
                    }
                }
                Ok(())
            }
            None => {
                warn!("Attempted to remove non-existent device: {}", name);
                Err(format!("Device '{}' not found.", name))
            }
        }
    }

    pub fn get_device(&self, name: &str) -> Option<&Device> {
        self.devices.get(&name.to_lowercase())
    }

    pub fn get_device_mut(&mut self, name: &str) -> Option<&mut Device> {
        self.devices.get_mut(&name.to_lowercase())
    }

    pub fn set_state(&mut self, name: &str, state: DeviceState) -> Result<(), String> {
        match self.get_device_mut(name) {
            Some(device) => {
                debug!("Setting device '{}' state to {:?}", name, state);
                device.state = state;
                Ok(())
            }
            None => {
                warn!("Attempted to set state on non-existent device: {}", name);
                Err(format!("Device '{}' not found.", name))
            }
        }
    }

    pub fn set_brightness(&mut self, name: &str, brightness: u8) -> Result<(), String> {
        let key = name.to_lowercase();
        match self.devices.get_mut(&key) {
            Some(device) => {
                if device.device_type != DeviceType::Light {
                    return Err(format!("'{}' is not a Light.", name));
                }
                if brightness > 100 {
                    return Err("Brightness must be 0–100.".to_string());
                }
                device.brightness = brightness;
                Ok(())
            }
            None => Err(format!("Device '{}' not found.", name)),
        }
    }

    pub fn set_temperature(&mut self, name: &str, temp: f64) -> Result<(), String> {
        let key = name.to_lowercase();
        match self.devices.get_mut(&key) {
            Some(device) => {
                if device.device_type != DeviceType::Thermostat {
                    return Err(format!("'{}' is not a Thermostat.", name));
                }
                device.temperature = Some(temp);
                Ok(())
            }
            None => Err(format!("Device '{}' not found.", name)),
        }
    }

    pub fn connect_device(&mut self, name: &str) -> Result<(), String> {
        match self.get_device_mut(name) {
            Some(device) => { device.connected = true; Ok(()) }
            None => Err(format!("Device '{}' not found.", name)),
        }
    }

    pub fn disconnect_device(&mut self, name: &str) -> Result<(), String> {
        match self.get_device_mut(name) {
            Some(device) => { device.connected = false; Ok(()) }
            None => Err(format!("Device '{}' not found.", name)),
        }
    }

    pub fn set_device_error(&mut self, name: &str, message: String) -> Result<(), String> {
        match self.get_device_mut(name) {
            Some(device) => { device.last_error = Some(message); Ok(()) }
            None => Err(format!("Device '{}' not found.", name)),
        }
    }

    pub fn clear_device_error(&mut self, name: &str) -> Result<(), String> {
        match self.get_device_mut(name) {
            Some(device) => { device.last_error = None; Ok(()) }
            None => Err(format!("Device '{}' not found.", name)),
        }
    }

    pub fn list_devices(&self) -> Vec<&Device> {
        let mut devices: Vec<&Device> = self.devices.values().collect();
        devices.sort_by(|a, b| a.name.cmp(&b.name));
        devices
    }

    // ── Room operations ─────────────────────────────────────────────

    pub fn add_room(&mut self, name: &str) -> Result<(), String> {
        let key = name.to_lowercase();
        if self.rooms.contains_key(&key) {
            warn!("Attempted to add duplicate room: {}", name);
            return Err(format!("Room '{}' already exists.", name));
        }
        self.rooms.insert(key, Room::new(name));
        info!("Added room '{}'", name);
        Ok(())
    }

    pub fn assign_device_to_room(&mut self, device_name: &str, room_name: &str) -> Result<(), String> {
        let device_key = device_name.to_lowercase();
        let room_key = room_name.to_lowercase();
        if !self.rooms.contains_key(&room_key) {
            return Err(format!("Room '{}' not found.", room_name));
        }
        let device = self
            .devices
            .get_mut(&device_key)
            .ok_or_else(|| format!("Device '{}' not found.", device_name))?;
        if let Some(old_room_name) = &device.room {
            let old_key = old_room_name.to_lowercase();
            if let Some(old_room) = self.rooms.get_mut(&old_key) {
                old_room.device_ids.retain(|id| id != &device.id);
            }
        }
        let device_id = device.id.clone();
        device.room = Some(room_name.to_string());
        let room = self.rooms.get_mut(&room_key).unwrap();
        room.device_ids.push(device_id);
        Ok(())
    }

    pub fn list_rooms(&self) -> Vec<&Room> {
        let mut rooms: Vec<&Room> = self.rooms.values().collect();
        rooms.sort_by(|a, b| a.name.cmp(&b.name));
        rooms
    }

    pub fn get_room_devices(&self, room_name: &str) -> Vec<&Device> {
        let room_key = room_name.to_lowercase();
        match self.rooms.get(&room_key) {
            Some(room) => room
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
}
