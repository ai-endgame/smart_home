use std::fmt;

use uuid::Uuid;

/// Represents the type of a smart home device.
#[derive(Debug, Clone, PartialEq)]
pub enum DeviceType {
    Light,
    Thermostat,
    Lock,
    Switch,
    Sensor,
}

impl fmt::Display for DeviceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeviceType::Light => write!(f, "Light"),
            DeviceType::Thermostat => write!(f, "Thermostat"),
            DeviceType::Lock => write!(f, "Lock"),
            DeviceType::Switch => write!(f, "Switch"),
            DeviceType::Sensor => write!(f, "Sensor"),
        }
    }
}

impl DeviceType {
    /// Parse a device type from a string (case-insensitive).
    pub fn from_str_loose(s: &str) -> Option<DeviceType> {
        match s.to_lowercase().as_str() {
            "light" => Some(DeviceType::Light),
            "thermostat" => Some(DeviceType::Thermostat),
            "lock" => Some(DeviceType::Lock),
            "switch" => Some(DeviceType::Switch),
            "sensor" => Some(DeviceType::Sensor),
            _ => None,
        }
    }
}

/// Represents the on/off state of a device.
#[derive(Debug, Clone, PartialEq)]
pub enum DeviceState {
    On,
    Off,
}

impl fmt::Display for DeviceState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeviceState::On => write!(f, "ON"),
            DeviceState::Off => write!(f, "OFF"),
        }
    }
}

/// A smart home device.
#[derive(Debug, Clone)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub device_type: DeviceType,
    pub state: DeviceState,
    pub room: Option<String>,
    /// Whether this device is currently connected to the server.
    pub connected: bool,
    /// Most recently reported device error.
    pub last_error: Option<String>,
    /// Brightness level (0–100), only meaningful for lights.
    pub brightness: u8,
    /// Temperature setting, only meaningful for thermostats.
    pub temperature: Option<f64>,
}

impl Device {
    /// Create a new device with default state (Off).
    pub fn new(name: &str, device_type: DeviceType) -> Self {
        let brightness = if device_type == DeviceType::Light {
            100
        } else {
            0
        };
        let temperature = if device_type == DeviceType::Thermostat {
            Some(22.0)
        } else {
            None
        };

        Device {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            device_type,
            state: DeviceState::Off,
            room: None,
            connected: false,
            last_error: None,
            brightness,
            temperature,
        }
    }
}

impl fmt::Display for Device {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let room_str = self.room.as_deref().unwrap_or("unassigned");

        write!(
            f,
            "[{}] {} ({}) — {} | room: {}",
            &self.id[..8],
            self.name,
            self.device_type,
            self.state,
            room_str
        )?;
        let connection = if self.connected {
            "connected"
        } else {
            "disconnected"
        };
        write!(f, " | {}", connection)?;

        if self.device_type == DeviceType::Light {
            write!(f, " | brightness: {}%", self.brightness)?;
        }
        if let Some(temp) = self.temperature {
            write!(f, " | temp: {:.1}°C", temp)?;
        }
        if let Some(err) = &self.last_error {
            write!(f, " | last error: {}", err)?;
        }

        Ok(())
    }
}

/// A room in the smart home.
#[derive(Debug, Clone)]
pub struct Room {
    pub name: String,
    pub device_ids: Vec<String>,
}

impl Room {
    pub fn new(name: &str) -> Self {
        Room {
            name: name.to_string(),
            device_ids: Vec::new(),
        }
    }
}

impl fmt::Display for Room {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Room '{}' — {} device(s)",
            self.name,
            self.device_ids.len()
        )
    }
}

// ─── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_creation() {
        let device = Device::new("Desk Lamp", DeviceType::Light);
        assert_eq!(device.name, "Desk Lamp");
        assert_eq!(device.device_type, DeviceType::Light);
        assert_eq!(device.state, DeviceState::Off);
        assert!(!device.connected);
        assert!(device.last_error.is_none());
        assert_eq!(device.brightness, 100);
        assert!(device.temperature.is_none());
    }

    #[test]
    fn test_thermostat_defaults() {
        let device = Device::new("Living Room Thermo", DeviceType::Thermostat);
        assert_eq!(device.temperature, Some(22.0));
        assert_eq!(device.brightness, 0);
    }

    #[test]
    fn test_device_type_parse() {
        assert_eq!(DeviceType::from_str_loose("LIGHT"), Some(DeviceType::Light));
        assert_eq!(
            DeviceType::from_str_loose("thermostat"),
            Some(DeviceType::Thermostat)
        );
        assert_eq!(DeviceType::from_str_loose("unknown"), None);
    }

    #[test]
    fn test_room_creation() {
        let room = Room::new("Kitchen");
        assert_eq!(room.name, "Kitchen");
        assert!(room.device_ids.is_empty());
    }

    #[test]
    fn test_display_impls_cover_device_and_room() {
        let mut light = Device::new("Lamp", DeviceType::Light);
        light.connected = true;
        light.last_error = Some("offline".to_string());

        let rendered = format!("{}", light);
        assert!(rendered.contains("Lamp"));
        assert!(rendered.contains("connected"));
        assert!(rendered.contains("brightness: 100%"));
        assert!(rendered.contains("last error: offline"));

        let thermo = Device::new("Thermo", DeviceType::Thermostat);
        let rendered = format!("{}", thermo);
        assert!(rendered.contains("temp: 22.0"));

        let room = Room {
            name: "Office".to_string(),
            device_ids: vec!["a".into(), "b".into()],
        };
        assert_eq!(format!("{}", room), "Room 'Office' — 2 device(s)");

        assert_eq!(format!("{}", DeviceType::Lock), "Lock");
        assert_eq!(format!("{}", DeviceType::Switch), "Switch");
        assert_eq!(format!("{}", DeviceType::Sensor), "Sensor");
        assert_eq!(format!("{}", DeviceState::On), "ON");
        assert_eq!(format!("{}", DeviceState::Off), "OFF");
    }
}
