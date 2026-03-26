pub mod automation;
pub mod dashboard;
pub mod device;
pub mod error;
pub mod manager;
pub mod presence;
pub mod scene;
pub mod script;

pub use automation::AutomationEngine;
pub use device::{Area, Device, DeviceState, DeviceType, Entity, EntityKind, MatterFabric, Room, ThreadRole, ZigbeeRole, slugify};
pub use error::DomainError;
pub use manager::SmartHome;
