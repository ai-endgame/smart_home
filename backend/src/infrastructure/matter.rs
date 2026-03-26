use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use mdns_sd::{ServiceDaemon, ServiceEvent};
use serde::Serialize;

use crate::domain::device::{Protocol, ThreadRole};
use crate::infrastructure::mdns::{DiscoveredDevice, DiscoveryStore};

// ── MatterStatus ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Default)]
pub struct MatterStatus {
    pub devices_seen: u64,
    pub commissioning_count: u64,
    /// ISO 8601 timestamp of the last resolved Matter device.
    pub last_seen_at: Option<String>,
    /// Whether the Matter state sync loop is enabled.
    pub sync_enabled: bool,
    /// ISO 8601 timestamp of the last completed sync cycle.
    pub last_sync_at: Option<String>,
}

// ── Commission job tracking ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CommissionStatus {
    Pending,
    InProgress,
    Done,
    Failed,
}

#[derive(Debug, Clone, Serialize)]
pub struct CommissionJob {
    pub job_id: String,
    pub status: CommissionStatus,
    pub message: String,
    pub device_id: Option<String>,
    pub error: Option<String>,
}

pub type CommissionStore = Arc<tokio::sync::RwLock<std::collections::HashMap<String, CommissionJob>>>;

/// Uses `std::sync::RwLock` (not tokio) so the scanner `std::thread` can write without async.
pub type MatterStatusStore = Arc<RwLock<MatterStatus>>;

pub fn new_status_store() -> MatterStatusStore {
    Arc::new(RwLock::new(MatterStatus::default()))
}

// ── Scanner ───────────────────────────────────────────────────────────────────

/// Spawn a background `std::thread` that passively scans for Matter devices
/// via mDNS (`_matter._tcp` and `_matterc._tcp`) and pushes them into `DiscoveryStore`.
///
/// Returns `true` if the scanner started, `false` if mDNS is disabled.
pub fn start_matter_scanner(store: DiscoveryStore, status: MatterStatusStore) -> bool {
    if std::env::var("MDNS_DISABLED").map(|v| v.eq_ignore_ascii_case("true")).unwrap_or(false) {
        return false;
    }
    std::thread::Builder::new()
        .name("matter-scanner".into())
        .spawn(move || matter_scan_loop(store, status))
        .expect("failed to spawn Matter scanner thread");
    true
}

fn matter_scan_loop(store: DiscoveryStore, status: MatterStatusStore) {
    let mdns = match ServiceDaemon::new() {
        Ok(d) => d,
        Err(e) => { log::error!("matter: mDNS daemon init failed: {e}"); return; }
    };

    let mut receivers = Vec::new();
    for stype in ["_matter._tcp.local.", "_matterc._tcp.local."] {
        match mdns.browse(stype) {
            Ok(r) => { log::debug!("matter: browsing {stype}"); receivers.push((r, stype)); }
            Err(e) => log::warn!("matter: cannot browse {stype}: {e}"),
        }
    }

    if receivers.is_empty() {
        log::warn!("matter: no service types could be browsed");
        return;
    }

    log::info!("matter: scanner active");

    loop {
        for (receiver, _stype) in &receivers {
            while let Ok(event) = receiver.try_recv() {
                handle_matter_event(event, &store, &status);
            }
        }
        std::thread::sleep(Duration::from_millis(250));
    }
}

fn handle_matter_event(event: ServiceEvent, store: &DiscoveryStore, status: &MatterStatusStore) {
    match event {
        ServiceEvent::ServiceResolved(info) => {
            let id = info.get_fullname().to_string();

            // Parse TXT record fields
            let mut properties: HashMap<String, String> = HashMap::new();
            for prop in info.get_properties().iter() {
                let val = prop.val()
                    .map(|v| String::from_utf8_lossy(v).into_owned())
                    .unwrap_or_default();
                properties.insert(prop.key().to_string(), val);
            }

            let discriminator: Option<u16> = properties.get("D")
                .and_then(|v| v.parse().ok());
            let (vendor_id, product_id) = parse_vp(properties.get("VP").map(String::as_str));
            let commissioning_mode: u8 = properties.get("CM")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0);
            let thread_role: Option<ThreadRole> = properties.get("_T")
                .and_then(|v| v.parse::<u16>().ok())
                .and_then(ThreadRole::from_txt_bitmask);

            // Build a human-friendly name
            let name = properties.get("DN")
                .cloned()
                .unwrap_or_else(|| {
                    info.get_fullname().split('.').next().unwrap_or("Matter Device").replace('_', " ")
                });

            let addresses: Vec<String> = info.get_addresses().iter().map(|a| a.to_string()).collect();

            // Store parsed metadata back into properties for the response DTO
            if let Some(d) = discriminator {
                properties.insert("discriminator".to_string(), d.to_string());
            }
            if let Some(v) = vendor_id {
                properties.insert("vendor_id".to_string(), v.to_string());
            }
            if let Some(p) = product_id {
                properties.insert("product_id".to_string(), p.to_string());
            }
            properties.insert("commissioning_mode".to_string(), commissioning_mode.to_string());
            if let Some(ref tr) = thread_role {
                properties.insert("thread_role".to_string(), tr.to_string());
            }

            let device = DiscoveredDevice {
                id: id.clone(),
                name: name.clone(),
                service_type: "_matter._tcp".to_string(),
                host: info.get_hostname().trim_end_matches('.').to_string(),
                addresses,
                port: info.get_port(),
                properties,
                suggested_type: infer_suggested_type(product_id),
                protocol: Some(Protocol::Matter),
                zigbee_role: None,
            };

            log::info!("matter: found '{}' (vendor={:?} product={:?} CM={})",
                name, vendor_id, product_id, commissioning_mode);

            store.write().unwrap_or_else(|e| e.into_inner()).insert(id, device);

            // Update status — std::sync::RwLock so safe from this std::thread
            if let Ok(mut s) = status.write() {
                s.devices_seen += 1;
                s.last_seen_at = Some(chrono::Utc::now().to_rfc3339());
                if commissioning_mode != 0 {
                    s.commissioning_count += 1;
                }
            }
        }
        ServiceEvent::ServiceRemoved(_, fullname) => {
            log::info!("matter: lost '{fullname}'");
            store.write().unwrap_or_else(|e| e.into_inner()).remove(&fullname);
        }
        _ => {}
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Parse the `VP` TXT field: `"<vendor_id>+<product_id>"` → `(Option<u16>, Option<u16>)`.
fn parse_vp(vp: Option<&str>) -> (Option<u16>, Option<u16>) {
    match vp {
        None => (None, None),
        Some(s) => {
            let mut parts = s.splitn(2, '+');
            let v = parts.next().and_then(|p| p.parse().ok());
            let p = parts.next().and_then(|p| p.parse().ok());
            (v, p)
        }
    }
}

/// Infer a device type hint from the product ID (very rough heuristic).
fn infer_suggested_type(product_id: Option<u16>) -> String {
    match product_id {
        Some(p) if p < 0x0100 => "light".to_string(),
        Some(p) if p < 0x0200 => "switch".to_string(),
        Some(p) if p < 0x0300 => "sensor".to_string(),
        _ => "sensor".to_string(),
    }
}
