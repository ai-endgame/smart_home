use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use uuid::Uuid;

use crate::domain::device::{DeviceType, Protocol};
use crate::http::errors::ApiError;
use crate::http::types::{CommissionJobResponse, CommissionRequest};
use crate::infrastructure::matter::{CommissionJob, CommissionStatus};
use crate::infrastructure::matter_control;
use crate::state::AppState;

pub async fn start_commission(
    State(state): State<AppState>,
    Json(payload): Json<CommissionRequest>,
) -> Result<(StatusCode, Json<CommissionJobResponse>), ApiError> {
    // Validate: 11-digit numeric setup code
    let code = payload.setup_code.trim().to_string();
    if code.len() != 11 || !code.chars().all(|c| c.is_ascii_digit()) {
        return Err(ApiError::BadRequest("invalid setup code format — must be exactly 11 digits".to_string()));
    }

    let job_id = Uuid::new_v4().to_string();
    let job = CommissionJob {
        job_id: job_id.clone(),
        status: CommissionStatus::Pending,
        message: "Commissioning queued".to_string(),
        device_id: None,
        error: None,
    };

    {
        let mut jobs = state.commission_jobs.write().await;
        // Keep only last 20 jobs
        if jobs.len() >= 20
            && let Some(oldest) = jobs.keys().next().cloned() {
                jobs.remove(&oldest);
            }
        jobs.insert(job_id.clone(), job);
    }

    // Spawn the commission task
    let jid = job_id.clone();
    let s = state.clone();
    let node_id = payload.node_id;
    tokio::spawn(async move {
        // Mark in-progress
        {
            let mut jobs = s.commission_jobs.write().await;
            if let Some(j) = jobs.get_mut(&jid) {
                j.status = CommissionStatus::InProgress;
                j.message = "Commissioning in progress…".to_string();
            }
        }

        match matter_control::run_commissioning(node_id, &code).await {
            Ok(()) => {
                // Add device to SmartHome
                let device_name = format!("matter-node-{node_id}");
                let device_id = {
                    let mut home = s.home.write().await;
                    let _ = home.add_device(&device_name, DeviceType::Light);
                    if let Some(d) = home.devices.get_mut(&device_name.to_lowercase()) {
                        d.control_protocol = Some(Protocol::Matter);
                        d.node_id = Some(node_id);
                        d.connected = true;
                    }
                    home.get_device(&device_name).map(|d| d.id.clone())
                };

                // Persist to DB
                {
                    let home = s.home.read().await;
                    if let Some(d) = home.get_device(&device_name) {
                        crate::http::helpers::persist_device(&s, d).await;
                    }
                }

                let mut jobs = s.commission_jobs.write().await;
                if let Some(j) = jobs.get_mut(&jid) {
                    j.status = CommissionStatus::Done;
                    j.message = format!("Device '{}' commissioned successfully", device_name);
                    j.device_id = device_id;
                }
            }
            Err(e) => {
                let mut jobs = s.commission_jobs.write().await;
                if let Some(j) = jobs.get_mut(&jid) {
                    j.status = CommissionStatus::Failed;
                    j.message = "Commissioning failed".to_string();
                    j.error = Some(e);
                }
            }
        }
    });

    let jobs = state.commission_jobs.read().await;
    let job = jobs.get(&job_id).unwrap();
    Ok((StatusCode::ACCEPTED, Json(CommissionJobResponse::from(job))))
}

pub async fn get_commission_job(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<Json<CommissionJobResponse>, ApiError> {
    let jobs = state.commission_jobs.read().await;
    let job = jobs.get(&job_id)
        .ok_or_else(|| ApiError::NotFound(format!("commission job '{}' not found", job_id)))?;
    Ok(Json(CommissionJobResponse::from(job)))
}

pub async fn list_commission_jobs(
    State(state): State<AppState>,
) -> Json<Vec<CommissionJobResponse>> {
    let jobs = state.commission_jobs.read().await;
    let list: Vec<CommissionJobResponse> = jobs.values().map(CommissionJobResponse::from).collect();
    Json(list)
}
