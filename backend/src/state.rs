use std::collections::HashMap;
use std::sync::Arc;

use sqlx::PgPool;
use tokio::sync::{Mutex, RwLock, oneshot};

use crate::domain::{AutomationEngine, SmartHome};
use crate::http::types::{ClientSession, ServerEvent};
use crate::infrastructure::mdns;

#[derive(Clone)]
pub struct AppState {
    pub home: Arc<RwLock<SmartHome>>,
    pub automation: Arc<RwLock<AutomationEngine>>,
    pub events: Arc<RwLock<Vec<ServerEvent>>>,
    pub clients: Arc<RwLock<HashMap<String, ClientSession>>>,
    pub shutdown_tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
    /// PostgreSQL pool; `None` when the server runs without a database URL.
    pub db: Option<PgPool>,
    /// Live mDNS-discovered devices on the local network.
    pub discovery: mdns::DiscoveryStore,
}

impl AppState {
    pub fn new(shutdown_tx: Option<oneshot::Sender<()>>) -> Self {
        Self {
            home: Arc::new(RwLock::new(SmartHome::new())),
            automation: Arc::new(RwLock::new(AutomationEngine::new())),
            events: Arc::new(RwLock::new(Vec::new())),
            clients: Arc::new(RwLock::new(HashMap::new())),
            shutdown_tx: Arc::new(Mutex::new(shutdown_tx)),
            db: None,
            discovery: mdns::new_store(),
        }
    }
}
