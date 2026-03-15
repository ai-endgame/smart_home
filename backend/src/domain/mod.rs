pub mod automation;
pub mod device;
pub mod manager;

pub use automation::AutomationEngine;
pub use device::{Device, DeviceState, DeviceType, Room};
pub use manager::SmartHome;
