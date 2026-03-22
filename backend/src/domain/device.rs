use std::fmt;

use chrono::{DateTime, Utc};
use serde_json::json;
use uuid::Uuid;

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Convert a display name to a kebab-case slug (e.g. "Living Room" → "living-room").
pub fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

// ── EntityKind ────────────────────────────────────────────────────────────────

/// Home Assistant entity domain — the type of capability an entity represents.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityKind {
    Switch,
    Light,
    Sensor,
    BinarySensor,
    Cover,
    Climate,
    MediaPlayer,
    Lock,
    Camera,
    Number,
    Select,
    Button,
    Person,
}

impl fmt::Display for EntityKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EntityKind::Switch       => write!(f, "switch"),
            EntityKind::Light        => write!(f, "light"),
            EntityKind::Sensor       => write!(f, "sensor"),
            EntityKind::BinarySensor => write!(f, "binary_sensor"),
            EntityKind::Cover        => write!(f, "cover"),
            EntityKind::Climate      => write!(f, "climate"),
            EntityKind::MediaPlayer  => write!(f, "media_player"),
            EntityKind::Lock         => write!(f, "lock"),
            EntityKind::Camera       => write!(f, "camera"),
            EntityKind::Number       => write!(f, "number"),
            EntityKind::Select       => write!(f, "select"),
            EntityKind::Button       => write!(f, "button"),
            EntityKind::Person       => write!(f, "person"),
        }
    }
}

// ── Entity ────────────────────────────────────────────────────────────────────

/// A single observable/controllable attribute exposed by a device.
/// Mirrors Home Assistant's entity model: one device → many entities.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Entity {
    /// HA-style entity ID: `{domain}.{device_slug}` or `{domain}.{device_slug}_{attr}`.
    pub entity_id: String,
    pub kind: EntityKind,
    pub device_id: String,
    pub name: String,
    pub state: String,
    pub unit_of_measurement: Option<String>,
    pub attributes: serde_json::Value,
}

impl Entity {
    #[allow(clippy::too_many_arguments)]
    fn new(
        kind: EntityKind,
        device_id: &str,
        device_slug: &str,
        attr_suffix: Option<&str>,
        name: &str,
        state: String,
        unit: Option<&str>,
        attributes: serde_json::Value,
    ) -> Self {
        let entity_id = match attr_suffix {
            Some(s) => format!("{}.{}_{}", kind, device_slug, s),
            None    => format!("{}.{}", kind, device_slug),
        };
        Entity {
            entity_id,
            kind,
            device_id: device_id.to_string(),
            name: name.to_string(),
            state,
            unit_of_measurement: unit.map(str::to_string),
            attributes,
        }
    }
}

// ── ThreadRole ────────────────────────────────────────────────────────────────

/// Role of a device within a Thread mesh network.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThreadRole {
    BorderRouter,
    Router,
    EndDevice,
    Sleepy,
}

impl fmt::Display for ThreadRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ThreadRole::BorderRouter => write!(f, "border_router"),
            ThreadRole::Router       => write!(f, "router"),
            ThreadRole::EndDevice    => write!(f, "end_device"),
            ThreadRole::Sleepy       => write!(f, "sleepy"),
        }
    }
}

impl ThreadRole {
    /// Parse from the Matter `_T` TXT record bitmask.
    /// Bit 5 (0x20) = Border Router capability.
    /// Bit 1 (0x02) = Full Thread Device (Router-eligible).
    /// Bit 0 (0x01) = Minimal Thread Device (End Device / Sleepy).
    pub fn from_txt_bitmask(bits: u16) -> Option<ThreadRole> {
        if bits == 0 {
            return None;
        }
        if bits & 0x20 != 0 {
            Some(ThreadRole::BorderRouter)
        } else if bits & 0x02 != 0 {
            Some(ThreadRole::Router)
        } else if bits & 0x04 != 0 {
            // Sleepy End Device bit
            Some(ThreadRole::Sleepy)
        } else {
            Some(ThreadRole::EndDevice)
        }
    }

    /// Parse from a stored string (Display round-trip).
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<ThreadRole> {
        match s {
            "border_router" => Some(ThreadRole::BorderRouter),
            "router"        => Some(ThreadRole::Router),
            "end_device"    => Some(ThreadRole::EndDevice),
            "sleepy"        => Some(ThreadRole::Sleepy),
            _               => None,
        }
    }
}

// ── MatterFabric ──────────────────────────────────────────────────────────────

/// Which ecosystem commissioned this Matter device.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MatterFabric {
    /// Opaque fabric identifier (hex string from the Matter fabric table).
    pub fabric_id: String,
    /// Vendor ID of the commissioner (e.g. 0x1049 = Apple).
    pub vendor_id: u16,
    /// Human-readable commissioner name (e.g. "Apple Home", "Google Home").
    pub commissioner: String,
}

// ── ZigbeeRole ────────────────────────────────────────────────────────────────

/// Role of a device within a Zigbee mesh network.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZigbeeRole {
    Coordinator,
    Router,
    EndDevice,
}

impl fmt::Display for ZigbeeRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ZigbeeRole::Coordinator => write!(f, "coordinator"),
            ZigbeeRole::Router      => write!(f, "router"),
            ZigbeeRole::EndDevice   => write!(f, "end_device"),
        }
    }
}

impl ZigbeeRole {
    /// Parse from a Zigbee2MQTT `type` field string.
    pub fn from_z2m_type(s: &str) -> Option<ZigbeeRole> {
        match s.to_lowercase().as_str() {
            "coordinator"          => Some(ZigbeeRole::Coordinator),
            "router"               => Some(ZigbeeRole::Router),
            "enddevice" | "end_device" | "end device" => Some(ZigbeeRole::EndDevice),
            _                      => None,
        }
    }
}

// ── Protocol ──────────────────────────────────────────────────────────────────

/// Metadata about a communication protocol.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ProtocolInfo {
    pub transport: &'static str,
    pub local_only: bool,
    pub mesh: bool,
    pub description: &'static str,
}

/// Communication protocol used by a device.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Protocol {
    Zigbee,
    ZWave,
    Matter,
    Thread,
    WiFi,
    Shelly,
    Tasmota,
    ESPHome,
    WLED,
    Unknown,
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Protocol::Zigbee  => write!(f, "zigbee"),
            Protocol::ZWave   => write!(f, "z_wave"),
            Protocol::Matter  => write!(f, "matter"),
            Protocol::Thread  => write!(f, "thread"),
            Protocol::WiFi    => write!(f, "wifi"),
            Protocol::Shelly  => write!(f, "shelly"),
            Protocol::Tasmota => write!(f, "tasmota"),
            Protocol::ESPHome => write!(f, "esphome"),
            Protocol::WLED    => write!(f, "wled"),
            Protocol::Unknown => write!(f, "unknown"),
        }
    }
}

impl Protocol {
    /// Parse a protocol from a string (case-insensitive, generous alias matching).
    pub fn from_str_loose(s: &str) -> Option<Protocol> {
        match s.to_lowercase().replace(['-', ' '], "_").as_str() {
            "zigbee"                        => Some(Protocol::Zigbee),
            "z_wave" | "zwave" | "z-wave"  => Some(Protocol::ZWave),
            "matter"                        => Some(Protocol::Matter),
            "thread"                        => Some(Protocol::Thread),
            "wifi" | "wi_fi" | "wi-fi" | "ip" => Some(Protocol::WiFi),
            "shelly" | "shelly1" | "shelly_plus" | "shellyplus" => Some(Protocol::Shelly),
            "tasmota"                       => Some(Protocol::Tasmota),
            "esphome" | "esp_home" | "esp-home" => Some(Protocol::ESPHome),
            "wled"                          => Some(Protocol::WLED),
            "unknown"                       => Some(Protocol::Unknown),
            _                               => None,
        }
    }

    /// Returns static metadata for this protocol.
    pub fn info(&self) -> ProtocolInfo {
        match self {
            Protocol::Zigbee => ProtocolInfo {
                transport: "802.15.4",
                local_only: true,
                mesh: true,
                description: "Low-power mesh protocol for sensors and actuators. Requires a coordinator (USB dongle + Zigbee2MQTT or ZHA).",
            },
            Protocol::ZWave => ProtocolInfo {
                transport: "900 MHz sub-GHz",
                local_only: true,
                mesh: true,
                description: "Mesh protocol on a dedicated frequency band — avoids 2.4 GHz congestion. Common for locks and dimmers.",
            },
            Protocol::Matter => ProtocolInfo {
                transport: "IP/Thread/Wi-Fi",
                local_only: true,
                mesh: true,
                description: "IP-native open standard backed by Apple/Google/Amazon/Samsung. Runs over Wi-Fi or Thread mesh.",
            },
            Protocol::Thread => ProtocolInfo {
                transport: "802.15.4 (IPv6)",
                local_only: true,
                mesh: true,
                description: "IPv6 mesh transport layer used by Matter for battery-powered devices. Requires a Thread Border Router.",
            },
            Protocol::WiFi => ProtocolInfo {
                transport: "Wi-Fi 2.4/5 GHz",
                local_only: false,
                mesh: false,
                description: "Direct IP devices on the LAN. No hub needed but relies on cloud by default unless locally controlled.",
            },
            Protocol::Shelly => ProtocolInfo {
                transport: "Wi-Fi (HTTP/MQTT)",
                local_only: true,
                mesh: false,
                description: "Shelly firmware over Wi-Fi. Supports fully local control via HTTP REST or MQTT without cloud.",
            },
            Protocol::Tasmota => ProtocolInfo {
                transport: "Wi-Fi (MQTT/HTTP)",
                local_only: true,
                mesh: false,
                description: "Open-source firmware for ESP8266/ESP32 devices. Full local control via MQTT or HTTP.",
            },
            Protocol::ESPHome => ProtocolInfo {
                transport: "Wi-Fi / Bluetooth (ESPHome API)",
                local_only: true,
                mesh: false,
                description: "YAML-configured firmware for ESP devices. Native API for HA — fully local, no cloud dependency.",
            },
            Protocol::WLED => ProtocolInfo {
                transport: "Wi-Fi (HTTP/MQTT/E1.31)",
                local_only: true,
                mesh: false,
                description: "LED strip controller firmware. Local control via JSON API, MQTT, or E1.31 streaming.",
            },
            Protocol::Unknown => ProtocolInfo {
                transport: "unknown",
                local_only: false,
                mesh: false,
                description: "Protocol not identified or not supported.",
            },
        }
    }

    /// All protocol variants, in a stable order — used by the protocols endpoint.
    pub fn all() -> &'static [Protocol] {
        &[
            Protocol::Zigbee,
            Protocol::ZWave,
            Protocol::Matter,
            Protocol::Thread,
            Protocol::WiFi,
            Protocol::Shelly,
            Protocol::Tasmota,
            Protocol::ESPHome,
            Protocol::WLED,
            Protocol::Unknown,
        ]
    }
}

// ── DeviceType ─────────────────────────────────────────────────────────────────

/// Represents the type of a smart home device.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceType {
    // Lighting
    Light,
    // Climate
    Thermostat,
    Fan,
    // Access control
    Lock,
    // Power
    Switch,
    Outlet,
    // Media & entertainment
    Tv,
    Speaker,
    MediaPlayer,
    // Sensing & safety
    Sensor,
    Camera,
    Alarm,
    // Covers / motorised
    Cover,
    // Infrastructure
    Hub,
}

impl fmt::Display for DeviceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // snake_case so helpers.rs can use format!("{}", dt) directly
        match self {
            DeviceType::Light       => write!(f, "light"),
            DeviceType::Thermostat  => write!(f, "thermostat"),
            DeviceType::Fan         => write!(f, "fan"),
            DeviceType::Lock        => write!(f, "lock"),
            DeviceType::Switch      => write!(f, "switch"),
            DeviceType::Outlet      => write!(f, "outlet"),
            DeviceType::Tv          => write!(f, "tv"),
            DeviceType::Speaker     => write!(f, "speaker"),
            DeviceType::MediaPlayer => write!(f, "media_player"),
            DeviceType::Sensor      => write!(f, "sensor"),
            DeviceType::Camera      => write!(f, "camera"),
            DeviceType::Alarm       => write!(f, "alarm"),
            DeviceType::Cover       => write!(f, "cover"),
            DeviceType::Hub         => write!(f, "hub"),
        }
    }
}

impl DeviceType {
    /// Parse a device type from a string (case-insensitive, accepts snake_case and spaces).
    pub fn from_str_loose(s: &str) -> Option<DeviceType> {
        match s.to_lowercase().replace(' ', "_").as_str() {
            "light"        => Some(DeviceType::Light),
            "thermostat"   => Some(DeviceType::Thermostat),
            "fan"          => Some(DeviceType::Fan),
            "lock"         => Some(DeviceType::Lock),
            "switch"       => Some(DeviceType::Switch),
            "outlet" | "plug" | "smart_plug"
                           => Some(DeviceType::Outlet),
            "tv" | "television"
                           => Some(DeviceType::Tv),
            "speaker" | "audio" | "soundbar"
                           => Some(DeviceType::Speaker),
            "media_player" | "mediaplayer" | "media"
                           => Some(DeviceType::MediaPlayer),
            "sensor"       => Some(DeviceType::Sensor),
            "camera" | "cam" | "doorbell"
                           => Some(DeviceType::Camera),
            "alarm" | "siren" | "security"
                           => Some(DeviceType::Alarm),
            "cover" | "blind" | "blinds" | "curtain" | "shutter" | "shade" | "garage"
                           => Some(DeviceType::Cover),
            "hub" | "bridge" | "gateway"
                           => Some(DeviceType::Hub),
            _              => None,
        }
    }

    /// Whether this device type supports a brightness value (0–100).
    pub fn has_brightness(&self) -> bool {
        matches!(self, DeviceType::Light | DeviceType::Cover)
    }

    /// Whether this device type supports a temperature value.
    pub fn has_temperature(&self) -> bool {
        matches!(self, DeviceType::Thermostat | DeviceType::Sensor)
    }
}

/// Represents the on/off state of a device.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeviceState {
    On,
    Off,
    Unknown,
}

impl fmt::Display for DeviceState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeviceState::On      => write!(f, "ON"),
            DeviceState::Off     => write!(f, "OFF"),
            DeviceState::Unknown => write!(f, "UNKNOWN"),
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
    pub connected: bool,
    pub last_error: Option<String>,
    /// Brightness / position level (0–100). Meaningful for lights and covers.
    pub brightness: u8,
    /// Temperature reading or setpoint. Meaningful for thermostats and sensors.
    pub temperature: Option<f64>,
    /// HTTP base URL for the physical device (e.g. "http://192.168.1.42").
    pub endpoint: Option<String>,
    /// Communication protocol used to control this device.
    pub control_protocol: Option<Protocol>,
    /// Role of this device within a Zigbee mesh network (if applicable).
    pub zigbee_role: Option<ZigbeeRole>,
    /// Role of this device within a Thread mesh network (if applicable).
    pub thread_role: Option<ThreadRole>,
    /// Matter fabric this device was commissioned into (if applicable).
    pub matter_fabric: Option<MatterFabric>,
    /// Free-form attributes bag (e.g. linkquality from Zigbee2MQTT).
    pub attributes: serde_json::Value,
    /// chip-tool node ID assigned during Matter commissioning.
    pub node_id: Option<u64>,
    /// Current power draw in watts (optional; set by PATCH /api/devices/{name}/energy).
    pub power_w: Option<f64>,
    /// Cumulative energy consumption in kilowatt-hours (optional).
    pub energy_kwh: Option<f64>,
}

impl Device {
    pub fn new(name: &str, device_type: DeviceType) -> Self {
        let brightness   = if device_type.has_brightness() { 100 } else { 0 };
        let temperature  = if device_type.has_temperature() { Some(22.0) } else { None };

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
            endpoint: None,
            control_protocol: None,
            zigbee_role: None,
            thread_role: None,
            matter_fabric: None,
            attributes: serde_json::Value::Object(serde_json::Map::new()),
            node_id: None,
            power_w: None,
            energy_kwh: None,
        }
    }

    /// Derive the HA-style entity list for this device based on its type and current state.
    /// This is a pure function — same device fields always yield the same entities.
    pub fn entities(&self) -> Vec<Entity> {
        let slug = slugify(&self.name);
        let id = &self.id;
        let on_off = match self.state { DeviceState::On => "on", DeviceState::Off | DeviceState::Unknown => "off" };

        match self.device_type {
            DeviceType::Light => vec![
                Entity::new(EntityKind::Switch, id, &slug, None,
                    &self.name, on_off.to_string(), None, json!({})),
                Entity::new(EntityKind::Number, id, &slug, Some("brightness"),
                    &format!("{} Brightness", self.name),
                    self.brightness.to_string(), Some("%"), json!({"min": 0, "max": 100})),
            ],
            DeviceType::Thermostat => vec![
                Entity::new(EntityKind::Climate, id, &slug, None,
                    &self.name, on_off.to_string(), None, json!({})),
                Entity::new(EntityKind::Sensor, id, &slug, Some("temperature"),
                    &format!("{} Temperature", self.name),
                    self.temperature.unwrap_or(0.0).to_string(), Some("°C"), json!({})),
                Entity::new(EntityKind::Number, id, &slug, Some("target_temp"),
                    &format!("{} Target Temperature", self.name),
                    self.temperature.unwrap_or(22.0).to_string(), Some("°C"),
                    json!({"min": -40, "max": 100})),
            ],
            DeviceType::Sensor => vec![
                Entity::new(EntityKind::Sensor, id, &slug,
                    if self.temperature.is_some() { Some("temperature") } else { None },
                    &self.name,
                    self.temperature.map_or_else(|| on_off.to_string(), |t| t.to_string()),
                    self.temperature.map(|_| "°C"), json!({})),
            ],
            DeviceType::Lock => vec![
                Entity::new(EntityKind::Lock, id, &slug, None,
                    &self.name, on_off.to_string(), None, json!({})),
            ],
            DeviceType::Cover => vec![
                Entity::new(EntityKind::Cover, id, &slug, None,
                    &self.name, on_off.to_string(), None, json!({})),
                Entity::new(EntityKind::Number, id, &slug, Some("position"),
                    &format!("{} Position", self.name),
                    self.brightness.to_string(), Some("%"), json!({"min": 0, "max": 100})),
            ],
            DeviceType::Switch | DeviceType::Outlet => vec![
                Entity::new(EntityKind::Switch, id, &slug, None,
                    &self.name, on_off.to_string(), None, json!({})),
            ],
            DeviceType::Fan => vec![
                Entity::new(EntityKind::Switch, id, &slug, None,
                    &self.name, on_off.to_string(), None, json!({})),
                Entity::new(EntityKind::Select, id, &slug, Some("speed"),
                    &format!("{} Speed", self.name),
                    "low".to_string(), None, json!({"options": ["low","medium","high"]})),
            ],
            DeviceType::Camera => vec![
                Entity::new(EntityKind::Camera, id, &slug, None,
                    &self.name, on_off.to_string(), None, json!({})),
            ],
            DeviceType::Alarm => vec![
                Entity::new(EntityKind::BinarySensor, id, &slug, None,
                    &self.name,
                    if self.state == DeviceState::On { "triggered".to_string() } else { "clear".to_string() },
                    None, json!({"device_class": "motion"})),
            ],
            DeviceType::Tv | DeviceType::Speaker | DeviceType::MediaPlayer => vec![
                Entity::new(EntityKind::MediaPlayer, id, &slug, None,
                    &self.name, on_off.to_string(), None, json!({})),
            ],
            DeviceType::Hub => vec![
                Entity::new(EntityKind::BinarySensor, id, &slug, None,
                    &self.name,
                    if self.connected { "online".to_string() } else { "offline".to_string() },
                    None, json!({"device_class": "connectivity"})),
            ],
        }
    }
}

impl fmt::Display for Device {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let room_str = self.room.as_deref().unwrap_or("unassigned");
        write!(
            f,
            "[{}] {} ({}) — {} | room: {}",
            self.id.get(..8).unwrap_or(&self.id),
            self.name,
            self.device_type,
            self.state,
            room_str,
        )?;
        write!(f, " | {}", if self.connected { "connected" } else { "disconnected" })?;
        if self.device_type.has_brightness() {
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

/// A named area (room/zone) in the smart home — mirrors the HA Area Registry.
#[derive(Debug, Clone)]
pub struct Area {
    /// Stable kebab-slug identifier (e.g. "living-room").
    pub area_id: String,
    pub name: String,
    pub device_ids: Vec<String>,
    /// Physical floor number (0 = ground, 1 = first, etc.).
    pub floor: Option<u8>,
    /// Optional icon name (e.g. MDI "mdi:sofa").
    pub icon: Option<String>,
}

impl Area {
    pub fn new(name: &str) -> Self {
        Area {
            area_id: slugify(name),
            name: name.to_string(),
            device_ids: Vec::new(),
            floor: None,
            icon: None,
        }
    }
}

impl fmt::Display for Area {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Area '{}' ({}) — {} device(s)", self.name, self.area_id, self.device_ids.len())
    }
}

/// Backward-compatible type alias so existing code that still references `Room` compiles.
pub type Room = Area;

// ── Device state history ─────────────────────────────────────────────────────

/// A single entry in a device's state-change history.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HistoryEntry {
    pub timestamp: DateTime<Utc>,
    pub state: DeviceState,
    pub brightness: u8,
    pub temperature: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_creation() {
        let device = Device::new("Desk Lamp", DeviceType::Light);
        assert_eq!(device.name, "Desk Lamp");
        assert_eq!(device.state, DeviceState::Off);
        assert!(!device.connected);
        assert_eq!(device.brightness, 100);
        assert!(device.temperature.is_none());
    }

    #[test]
    fn test_thermostat_defaults() {
        let device = Device::new("Thermo", DeviceType::Thermostat);
        assert_eq!(device.temperature, Some(22.0));
        assert_eq!(device.brightness, 0);
    }

    #[test]
    fn test_cover_has_brightness() {
        let device = Device::new("Blinds", DeviceType::Cover);
        assert_eq!(device.brightness, 100);
        assert!(device.temperature.is_none());
    }

    #[test]
    fn test_device_type_parse() {
        assert_eq!(DeviceType::from_str_loose("LIGHT"),        Some(DeviceType::Light));
        assert_eq!(DeviceType::from_str_loose("media_player"), Some(DeviceType::MediaPlayer));
        assert_eq!(DeviceType::from_str_loose("mediaplayer"),  Some(DeviceType::MediaPlayer));
        assert_eq!(DeviceType::from_str_loose("plug"),         Some(DeviceType::Outlet));
        assert_eq!(DeviceType::from_str_loose("television"),   Some(DeviceType::Tv));
        assert_eq!(DeviceType::from_str_loose("blinds"),       Some(DeviceType::Cover));
        assert_eq!(DeviceType::from_str_loose("bridge"),       Some(DeviceType::Hub));
        assert_eq!(DeviceType::from_str_loose("soundbar"),     Some(DeviceType::Speaker));
        assert_eq!(DeviceType::from_str_loose("unknown"),      None);
    }

    #[test]
    fn test_display_snake_case() {
        assert_eq!(format!("{}", DeviceType::Light),       "light");
        assert_eq!(format!("{}", DeviceType::MediaPlayer), "media_player");
        assert_eq!(format!("{}", DeviceType::Tv),          "tv");
        assert_eq!(format!("{}", DeviceType::Hub),         "hub");
        assert_eq!(format!("{}", DeviceState::On),         "ON");
    }

    #[test]
    fn test_display_impls() {
        let mut light = Device::new("Lamp", DeviceType::Light);
        light.connected = true;
        light.last_error = Some("offline".to_string());
        let s = format!("{}", light);
        assert!(s.contains("Lamp") && s.contains("connected") && s.contains("brightness: 100%"));

        let thermo = Device::new("Thermo", DeviceType::Thermostat);
        assert!(format!("{}", thermo).contains("temp: 22.0"));

        let room = Area { area_id: "office".to_string(), name: "Office".to_string(), device_ids: vec!["a".into(), "b".into()], floor: None, icon: None };
        assert_eq!(format!("{}", room), "Area 'Office' (office) — 2 device(s)");
    }

    #[test]
    fn test_protocol_from_str_loose() {
        assert_eq!(Protocol::from_str_loose("zigbee"),    Some(Protocol::Zigbee));
        assert_eq!(Protocol::from_str_loose("ZIGBEE"),    Some(Protocol::Zigbee));
        assert_eq!(Protocol::from_str_loose("zwave"),     Some(Protocol::ZWave));
        assert_eq!(Protocol::from_str_loose("z_wave"),    Some(Protocol::ZWave));
        assert_eq!(Protocol::from_str_loose("z-wave"),    Some(Protocol::ZWave));
        assert_eq!(Protocol::from_str_loose("matter"),    Some(Protocol::Matter));
        assert_eq!(Protocol::from_str_loose("thread"),    Some(Protocol::Thread));
        assert_eq!(Protocol::from_str_loose("wifi"),      Some(Protocol::WiFi));
        assert_eq!(Protocol::from_str_loose("wi-fi"),     Some(Protocol::WiFi));
        assert_eq!(Protocol::from_str_loose("shelly"),    Some(Protocol::Shelly));
        assert_eq!(Protocol::from_str_loose("shelly1"),   Some(Protocol::Shelly));
        assert_eq!(Protocol::from_str_loose("shelly_plus"), Some(Protocol::Shelly));
        assert_eq!(Protocol::from_str_loose("tasmota"),   Some(Protocol::Tasmota));
        assert_eq!(Protocol::from_str_loose("esphome"),   Some(Protocol::ESPHome));
        assert_eq!(Protocol::from_str_loose("esp-home"),  Some(Protocol::ESPHome));
        assert_eq!(Protocol::from_str_loose("wled"),      Some(Protocol::WLED));
        assert_eq!(Protocol::from_str_loose("unknown"),   Some(Protocol::Unknown));
        assert_eq!(Protocol::from_str_loose("foobar"),    None);
        assert_eq!(Protocol::from_str_loose(""),          None);
    }

    #[test]
    fn test_protocol_display() {
        assert_eq!(Protocol::Zigbee.to_string(),  "zigbee");
        assert_eq!(Protocol::ZWave.to_string(),   "z_wave");
        assert_eq!(Protocol::Matter.to_string(),  "matter");
        assert_eq!(Protocol::Thread.to_string(),  "thread");
        assert_eq!(Protocol::WiFi.to_string(),    "wifi");
        assert_eq!(Protocol::Shelly.to_string(),  "shelly");
        assert_eq!(Protocol::Tasmota.to_string(), "tasmota");
        assert_eq!(Protocol::ESPHome.to_string(), "esphome");
        assert_eq!(Protocol::WLED.to_string(),    "wled");
        assert_eq!(Protocol::Unknown.to_string(), "unknown");
    }

    #[test]
    fn test_protocol_info() {
        let z = Protocol::Zigbee.info();
        assert_eq!(z.transport, "802.15.4");
        assert!(z.local_only);
        assert!(z.mesh);

        let s = Protocol::Shelly.info();
        assert!(s.local_only);
        assert!(!s.mesh);

        let m = Protocol::Matter.info();
        assert_eq!(m.transport, "IP/Thread/Wi-Fi");
        assert!(m.local_only);
        assert!(m.mesh);

        let w = Protocol::WiFi.info();
        assert!(!w.local_only);
        assert!(!w.mesh);
    }

    #[test]
    fn test_protocol_all_variants() {
        assert_eq!(Protocol::all().len(), 10);
    }

    #[test]
    fn test_device_protocol_field() {
        let mut device = Device::new("LED Strip", DeviceType::Light);
        assert!(device.control_protocol.is_none());
        device.control_protocol = Some(Protocol::WLED);
        assert_eq!(device.control_protocol, Some(Protocol::WLED));
    }

    #[test]
    fn test_entity_kind_display() {
        assert_eq!(EntityKind::Switch.to_string(),       "switch");
        assert_eq!(EntityKind::BinarySensor.to_string(), "binary_sensor");
        assert_eq!(EntityKind::MediaPlayer.to_string(),  "media_player");
        assert_eq!(EntityKind::Climate.to_string(),      "climate");
        assert_eq!(EntityKind::Number.to_string(),       "number");
    }

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Living Room"),       "living-room");
        assert_eq!(slugify("Ground Floor/Hall"), "ground-floor-hall");
        assert_eq!(slugify("Desk Lamp"),         "desk-lamp");
        assert_eq!(slugify("  extra  spaces  "), "extra-spaces");
    }

    #[test]
    fn test_light_entities() {
        let mut light = Device::new("Desk Lamp", DeviceType::Light);
        light.state = DeviceState::On;
        light.brightness = 75;
        let entities = light.entities();
        assert_eq!(entities.len(), 2);
        assert_eq!(entities[0].kind, EntityKind::Switch);
        assert_eq!(entities[0].entity_id, "switch.desk-lamp");
        assert_eq!(entities[0].state, "on");
        assert_eq!(entities[1].kind, EntityKind::Number);
        assert_eq!(entities[1].entity_id, "number.desk-lamp_brightness");
        assert_eq!(entities[1].state, "75");
        assert_eq!(entities[1].unit_of_measurement.as_deref(), Some("%"));
    }

    #[test]
    fn test_thermostat_entities() {
        let mut thermo = Device::new("Hall Thermo", DeviceType::Thermostat);
        thermo.temperature = Some(21.5);
        let entities = thermo.entities();
        assert_eq!(entities.len(), 3);
        assert_eq!(entities[0].kind, EntityKind::Climate);
        assert_eq!(entities[1].kind, EntityKind::Sensor);
        assert_eq!(entities[1].state, "21.5");
        assert_eq!(entities[1].unit_of_measurement.as_deref(), Some("°C"));
        assert_eq!(entities[2].kind, EntityKind::Number);
    }

    #[test]
    fn test_entities_pure() {
        let device = Device::new("Lamp", DeviceType::Light);
        let e1 = device.entities();
        let e2 = device.entities();
        assert_eq!(e1.len(), e2.len());
        assert_eq!(e1[0].entity_id, e2[0].entity_id);
    }

    #[test]
    fn test_thread_role_display() {
        assert_eq!(ThreadRole::BorderRouter.to_string(), "border_router");
        assert_eq!(ThreadRole::Router.to_string(),       "router");
        assert_eq!(ThreadRole::EndDevice.to_string(),    "end_device");
        assert_eq!(ThreadRole::Sleepy.to_string(),       "sleepy");
    }

    #[test]
    fn test_thread_role_from_bitmask() {
        // Bit 5 set → BorderRouter
        assert_eq!(ThreadRole::from_txt_bitmask(0x20), Some(ThreadRole::BorderRouter));
        // Bit 1 set → Router
        assert_eq!(ThreadRole::from_txt_bitmask(0x02), Some(ThreadRole::Router));
        // Bit 2 set → Sleepy
        assert_eq!(ThreadRole::from_txt_bitmask(0x04), Some(ThreadRole::Sleepy));
        // No bits → None
        assert_eq!(ThreadRole::from_txt_bitmask(0x00), None);
        // Other bits → EndDevice fallback
        assert_eq!(ThreadRole::from_txt_bitmask(0x08), Some(ThreadRole::EndDevice));
    }

    #[test]
    fn test_thread_role_from_str() {
        assert_eq!(ThreadRole::from_str("border_router"), Some(ThreadRole::BorderRouter));
        assert_eq!(ThreadRole::from_str("router"),        Some(ThreadRole::Router));
        assert_eq!(ThreadRole::from_str("end_device"),    Some(ThreadRole::EndDevice));
        assert_eq!(ThreadRole::from_str("sleepy"),        Some(ThreadRole::Sleepy));
        assert_eq!(ThreadRole::from_str("unknown"),       None);
    }
}
