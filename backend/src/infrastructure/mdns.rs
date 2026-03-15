use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use mdns_sd::{ServiceDaemon, ServiceEvent};
use serde::Serialize;

const BROWSE_TYPES: &[(&str, &str)] = &[
    ("_hap._tcp.local.", "sensor"),
    ("_googlecast._tcp.local.", "switch"),
    ("_esphomelib._tcp.local.", "sensor"),
    ("_wled._tcp.local.", "light"),
    ("_shelly._tcp.local.", "switch"),
    ("_hue._tcp.local.", "light"),
    ("_arduino._tcp.local.", "sensor"),
    ("_smartthings._tcp.local.", "switch"),
    ("_matter._tcp.local.", "sensor"),
];

#[derive(Debug, Clone, Serialize)]
pub struct DiscoveredDevice {
    pub id: String,
    pub name: String,
    pub service_type: String,
    pub host: String,
    pub addresses: Vec<String>,
    pub port: u16,
    pub properties: HashMap<String, String>,
    pub suggested_type: String,
}

pub type DiscoveryStore = Arc<RwLock<HashMap<String, DiscoveredDevice>>>;

pub fn new_store() -> DiscoveryStore {
    Arc::new(RwLock::new(HashMap::new()))
}

pub fn start(store: DiscoveryStore) {
    std::thread::Builder::new()
        .name("mdns-discovery".into())
        .spawn(move || discovery_loop(store))
        .expect("failed to spawn mDNS discovery thread");
}

fn discovery_loop(store: DiscoveryStore) {
    let mdns = match ServiceDaemon::new() {
        Ok(d) => d,
        Err(e) => { log::error!("mDNS: daemon init failed: {e}"); return; }
    };

    let mut receivers = Vec::new();
    for (stype, dtype) in BROWSE_TYPES {
        match mdns.browse(stype) {
            Ok(r) => { log::debug!("mDNS: browsing {stype}"); receivers.push((r, stype.to_string(), dtype.to_string())); }
            Err(e) => log::warn!("mDNS: cannot browse {stype}: {e}"),
        }
    }

    if receivers.is_empty() {
        log::warn!("mDNS: no service types could be browsed — discovery unavailable");
        return;
    }

    log::info!("mDNS: discovery active ({} service types)", receivers.len());

    loop {
        for (receiver, stype, dtype) in &receivers {
            while let Ok(event) = receiver.try_recv() {
                handle_event(event, &store, stype, dtype);
            }
        }
        std::thread::sleep(Duration::from_millis(200));
    }
}

fn handle_event(event: ServiceEvent, store: &DiscoveryStore, service_type: &str, suggested_type: &str) {
    match event {
        ServiceEvent::ServiceResolved(info) => {
            let id = info.get_fullname().to_string();
            let mut properties: HashMap<String, String> = HashMap::new();
            for prop in info.get_properties().iter() {
                let val = prop.val().map(|v| String::from_utf8_lossy(v).into_owned()).unwrap_or_default();
                properties.insert(prop.key().to_string(), val);
            }
            let name = properties.get("fn")
                .or_else(|| properties.get("md"))
                .or_else(|| properties.get("n"))
                .cloned()
                .unwrap_or_else(|| {
                    info.get_fullname().split('.').next().unwrap_or("Unknown Device").replace('_', " ")
                });
            let addresses: Vec<String> = info.get_addresses().iter().map(|a| a.to_string()).collect();
            let device = DiscoveredDevice {
                id: id.clone(),
                name,
                service_type: service_type.to_string(),
                host: info.get_hostname().trim_end_matches('.').to_string(),
                addresses,
                port: info.get_port(),
                properties,
                suggested_type: suggested_type.to_string(),
            };
            log::info!("mDNS: found '{}' ({})", device.name, service_type);
            store.write().unwrap().insert(id, device);
        }
        ServiceEvent::ServiceRemoved(_, fullname) => {
            log::info!("mDNS: lost '{fullname}'");
            store.write().unwrap().remove(&fullname);
        }
        _ => {}
    }
}
