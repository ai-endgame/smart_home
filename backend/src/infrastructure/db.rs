use std::collections::HashMap;

use sqlx::{postgres::PgPoolOptions, PgPool, Row};

use crate::domain::dashboard::Dashboard;
use crate::domain::device::{Area, Device, DeviceState, DeviceType, MatterFabric, Protocol, ThreadRole, ZigbeeRole};
use crate::domain::presence::PersonTracker;
use crate::domain::scene::{Scene, SceneState};
use crate::domain::script::Script;

pub async fn create_pool(url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new().max_connections(5).connect(url).await?;
    migrate(&pool).await?;
    log::info!("database: connected and schema ready");
    Ok(pool)
}

async fn migrate(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS dashboards (
            id         TEXT PRIMARY KEY,
            name       TEXT NOT NULL UNIQUE,
            icon       TEXT,
            views      JSONB NOT NULL DEFAULT '[]',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS persons (
            id                TEXT PRIMARY KEY,
            name              TEXT NOT NULL UNIQUE,
            grace_period_secs INT  NOT NULL DEFAULT 120,
            sources           JSONB NOT NULL DEFAULT '{}',
            away_since        TIMESTAMPTZ,
            created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS scripts (
            id          TEXT PRIMARY KEY,
            name        TEXT NOT NULL UNIQUE,
            description TEXT NOT NULL DEFAULT '',
            params      JSONB NOT NULL DEFAULT '[]',
            steps       JSONB NOT NULL DEFAULT '[]',
            created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS scenes (
            id         TEXT PRIMARY KEY,
            name       TEXT NOT NULL UNIQUE,
            states     JSONB NOT NULL DEFAULT '{}',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS devices (
            id               TEXT PRIMARY KEY,
            name             TEXT NOT NULL,
            device_type      TEXT NOT NULL,
            state            TEXT NOT NULL DEFAULT 'off',
            room             TEXT,
            connected        BOOLEAN NOT NULL DEFAULT false,
            last_error       TEXT,
            brightness       SMALLINT NOT NULL DEFAULT 0,
            temperature      DOUBLE PRECISION,
            endpoint         TEXT,
            control_protocol TEXT,
            created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
    )
    .execute(pool)
    .await?;
    // Idempotent column additions for existing databases
    for ddl in [
        "ALTER TABLE devices ADD COLUMN IF NOT EXISTS endpoint TEXT",
        "ALTER TABLE devices ADD COLUMN IF NOT EXISTS control_protocol TEXT",
        "ALTER TABLE devices ADD COLUMN IF NOT EXISTS area_floor SMALLINT",
        "ALTER TABLE devices ADD COLUMN IF NOT EXISTS area_icon TEXT",
        "ALTER TABLE devices ADD COLUMN IF NOT EXISTS zigbee_role TEXT",
        "ALTER TABLE devices ADD COLUMN IF NOT EXISTS thread_role TEXT",
        "ALTER TABLE devices ADD COLUMN IF NOT EXISTS matter_fabric TEXT",
        "ALTER TABLE devices ADD COLUMN IF NOT EXISTS node_id BIGINT",
        "ALTER TABLE devices ADD COLUMN IF NOT EXISTS power_w DOUBLE PRECISION",
        "ALTER TABLE devices ADD COLUMN IF NOT EXISTS energy_kwh DOUBLE PRECISION",
    ] {
        sqlx::query(ddl).execute(pool).await?;
    }
    Ok(())
}

/// Returns loaded devices plus a map of `room_name → (floor, icon)` for area metadata reconstruction.
pub async fn load_all_devices(pool: &PgPool) -> Result<(Vec<Device>, HashMap<String, (Option<u8>, Option<String>)>), sqlx::Error> {
    let rows = sqlx::query(
        "SELECT id, name, device_type, state, room, connected, \
         last_error, brightness, temperature, endpoint, control_protocol, \
         area_floor, area_icon, zigbee_role, thread_role, matter_fabric, node_id, \
         power_w, energy_kwh \
         FROM devices ORDER BY name",
    )
    .fetch_all(pool)
    .await?;

    let mut area_meta: HashMap<String, (Option<u8>, Option<String>)> = HashMap::new();

    let devices = rows
        .iter()
        .filter_map(|row| {
            let raw_type: &str = row.try_get("device_type").ok()?;
            let device_type = match parse_device_type(raw_type) {
                Some(dt) => dt,
                None => {
                    let name: &str = row.try_get("name").unwrap_or("<unknown>");
                    log::warn!("db: skipping device '{}' with unknown device_type '{}'", name, raw_type);
                    return None;
                }
            };
            let state_str: &str = row.try_get("state").ok()?;
            let state = if state_str == "on" { DeviceState::On } else { DeviceState::Off };
            let brightness_raw: i16 = row.try_get("brightness").ok()?;
            let name_val: String = row.try_get("name").ok()?;
            let brightness = if (0..=100).contains(&brightness_raw) {
                brightness_raw as u8
            } else {
                log::warn!("db: brightness {} out of range for device '{}', defaulting to 0", brightness_raw, name_val);
                0u8
            };
            let room: Option<String> = row.try_get::<Option<String>, _>("room").ok()?;
            // Collect area metadata keyed by room name
            if let Some(ref room_name) = room {
                let floor: Option<i16> = row.try_get::<Option<i16>, _>("area_floor").ok().flatten();
                let icon: Option<String> = row.try_get::<Option<String>, _>("area_icon").ok().flatten();
                area_meta.entry(room_name.clone())
                    .or_insert((floor.map(|f| f as u8), icon));
            }
            Some(Device {
                id: row.try_get("id").ok()?,
                name: name_val,
                device_type,
                state,
                room,
                connected: row.try_get("connected").ok()?,
                last_error: row.try_get::<Option<String>, _>("last_error").ok()?,
                brightness,
                temperature: row.try_get::<Option<f64>, _>("temperature").ok()?,
                endpoint: row.try_get::<Option<String>, _>("endpoint").ok().flatten(),
                control_protocol: {
                    let raw: Option<String> = row.try_get::<Option<String>, _>("control_protocol").ok().flatten();
                    raw.and_then(|s| {
                        Protocol::from_str_loose(&s).or_else(|| {
                            let name: &str = row.try_get("name").unwrap_or("<unknown>");
                            log::warn!("db: unknown control_protocol '{}' for device '{}', ignoring", s, name);
                            None
                        })
                    })
                },
                zigbee_role: {
                    let raw: Option<String> = row.try_get::<Option<String>, _>("zigbee_role").ok().flatten();
                    raw.and_then(|s| ZigbeeRole::from_z2m_type(&s))
                },
                thread_role: {
                    let raw: Option<String> = row.try_get::<Option<String>, _>("thread_role").ok().flatten();
                    raw.and_then(|s| ThreadRole::from_str(&s))
                },
                matter_fabric: {
                    let raw: Option<String> = row.try_get::<Option<String>, _>("matter_fabric").ok().flatten();
                    raw.and_then(|s| serde_json::from_str::<MatterFabric>(&s).ok())
                },
                attributes: serde_json::Value::Object(serde_json::Map::new()),
                node_id: {
                    let raw: Option<i64> = row.try_get::<Option<i64>, _>("node_id").ok().flatten();
                    raw.map(|v| v as u64)
                },
                power_w: row.try_get::<Option<f64>, _>("power_w").ok().flatten(),
                energy_kwh: row.try_get::<Option<f64>, _>("energy_kwh").ok().flatten(),
            })
        })
        .collect();

    Ok((devices, area_meta))
}

pub async fn upsert_device(pool: &PgPool, device: &Device, area: Option<&Area>) -> Result<(), sqlx::Error> {
    let area_floor: Option<i16> = area.and_then(|a| a.floor).map(|f| f as i16);
    let area_icon: Option<&str> = area.and_then(|a| a.icon.as_deref());
    sqlx::query(
        "INSERT INTO devices
             (id, name, device_type, state, room, connected, last_error, brightness, temperature, endpoint, control_protocol, area_floor, area_icon, zigbee_role, thread_role, matter_fabric, node_id, power_w, energy_kwh, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, NOW())
         ON CONFLICT (id) DO UPDATE SET
             state            = EXCLUDED.state,
             room             = EXCLUDED.room,
             connected        = EXCLUDED.connected,
             last_error       = EXCLUDED.last_error,
             brightness       = EXCLUDED.brightness,
             temperature      = EXCLUDED.temperature,
             endpoint         = EXCLUDED.endpoint,
             control_protocol = EXCLUDED.control_protocol,
             area_floor       = EXCLUDED.area_floor,
             area_icon        = EXCLUDED.area_icon,
             zigbee_role      = EXCLUDED.zigbee_role,
             thread_role      = EXCLUDED.thread_role,
             matter_fabric    = EXCLUDED.matter_fabric,
             node_id          = EXCLUDED.node_id,
             power_w          = EXCLUDED.power_w,
             energy_kwh       = EXCLUDED.energy_kwh,
             updated_at       = NOW()",
    )
    .bind(&device.id)
    .bind(&device.name)
    .bind(device_type_to_str(&device.device_type))
    .bind(state_to_str(&device.state))
    .bind(&device.room)
    .bind(device.connected)
    .bind(&device.last_error)
    .bind(device.brightness as i16)
    .bind(device.temperature)
    .bind(&device.endpoint)
    .bind(device.control_protocol.as_ref().map(|p| p.to_string()))
    .bind(area_floor)
    .bind(area_icon)
    .bind(device.zigbee_role.as_ref().map(|r| r.to_string()))
    .bind(device.thread_role.as_ref().map(|r| r.to_string()))
    .bind(device.matter_fabric.as_ref().and_then(|f| serde_json::to_string(f).ok()))
    .bind(device.node_id.map(|n| n as i64))
    .bind(device.power_w)
    .bind(device.energy_kwh)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_device(pool: &PgPool, device_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM devices WHERE id = $1")
        .bind(device_id)
        .execute(pool)
        .await?;
    Ok(())
}

fn parse_device_type(s: &str) -> Option<DeviceType> {
    // Use the domain's own parser so this stays in sync with from_str_loose.
    DeviceType::from_str_loose(s)
}

fn device_type_to_str(dt: &DeviceType) -> String {
    // Use the Display impl which already returns the canonical snake_case string.
    dt.to_string()
}

fn state_to_str(s: &DeviceState) -> &'static str {
    match s { DeviceState::On => "on", DeviceState::Off => "off", DeviceState::Unknown => "off" }
}

// ── Script persistence ────────────────────────────────────────────────────────

pub async fn upsert_script(pool: &PgPool, script: &Script) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO scripts (id, name, description, params, steps)
         VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (id) DO UPDATE SET
             name        = EXCLUDED.name,
             description = EXCLUDED.description,
             params      = EXCLUDED.params,
             steps       = EXCLUDED.steps",
    )
    .bind(&script.id)
    .bind(&script.name)
    .bind(&script.description)
    .bind(serde_json::to_value(&script.params).unwrap_or_default())
    .bind(serde_json::to_value(&script.steps).unwrap_or_default())
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_script(pool: &PgPool, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM scripts WHERE id = $1").bind(id).execute(pool).await?;
    Ok(())
}

pub async fn load_all_scripts(pool: &PgPool) -> Result<Vec<Script>, sqlx::Error> {
    let rows = sqlx::query("SELECT id, name, description, params, steps FROM scripts ORDER BY name")
        .fetch_all(pool)
        .await?;
    let scripts = rows.iter().filter_map(|row| {
        let params_val: serde_json::Value = row.try_get("params").ok()?;
        let steps_val:  serde_json::Value = row.try_get("steps").ok()?;
        Some(Script {
            id:          row.try_get("id").ok()?,
            name:        row.try_get("name").ok()?,
            description: row.try_get("description").ok()?,
            params:      serde_json::from_value(params_val).unwrap_or_default(),
            steps:       serde_json::from_value(steps_val).unwrap_or_default(),
        })
    }).collect();
    Ok(scripts)
}

// ── Scene persistence ─────────────────────────────────────────────────────────

pub async fn upsert_scene(pool: &PgPool, scene: &Scene) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO scenes (id, name, states)
         VALUES ($1, $2, $3)
         ON CONFLICT (id) DO UPDATE SET
             name   = EXCLUDED.name,
             states = EXCLUDED.states",
    )
    .bind(&scene.id)
    .bind(&scene.name)
    .bind(serde_json::to_value(&scene.states).unwrap_or_default())
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_scene(pool: &PgPool, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM scenes WHERE id = $1").bind(id).execute(pool).await?;
    Ok(())
}

pub async fn load_all_scenes(pool: &PgPool) -> Result<Vec<Scene>, sqlx::Error> {
    let rows = sqlx::query("SELECT id, name, states FROM scenes ORDER BY name")
        .fetch_all(pool)
        .await?;
    let scenes = rows.iter().filter_map(|row| {
        let states_val: serde_json::Value = row.try_get("states").ok()?;
        Some(Scene {
            id:     row.try_get("id").ok()?,
            name:   row.try_get("name").ok()?,
            states: serde_json::from_value::<std::collections::HashMap<String, SceneState>>(states_val).unwrap_or_default(),
        })
    }).collect();
    Ok(scenes)
}

// ── Person persistence ────────────────────────────────────────────────────────

pub async fn upsert_person(pool: &PgPool, person: &PersonTracker) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO persons (id, name, grace_period_secs, sources, away_since)
         VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (id) DO UPDATE SET
             name              = EXCLUDED.name,
             grace_period_secs = EXCLUDED.grace_period_secs,
             sources           = EXCLUDED.sources,
             away_since        = EXCLUDED.away_since",
    )
    .bind(&person.id)
    .bind(&person.name)
    .bind(person.grace_period_secs as i32)
    .bind(serde_json::to_value(&person.sources).unwrap_or_default())
    .bind(person.away_since)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_person(pool: &PgPool, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM persons WHERE id = $1").bind(id).execute(pool).await?;
    Ok(())
}

pub async fn load_all_persons(pool: &PgPool) -> Result<Vec<PersonTracker>, sqlx::Error> {
    let rows = sqlx::query("SELECT id, name, grace_period_secs, sources, away_since FROM persons ORDER BY name")
        .fetch_all(pool)
        .await?;
    let persons = rows.iter().filter_map(|row| {
        let sources_val: serde_json::Value = row.try_get("sources").ok()?;
        Some(PersonTracker {
            id:                row.try_get("id").ok()?,
            name:              row.try_get("name").ok()?,
            grace_period_secs: row.try_get::<i32, _>("grace_period_secs").ok()? as u32,
            sources:           serde_json::from_value(sources_val).unwrap_or_default(),
            away_since:        row.try_get("away_since").ok().flatten(),
        })
    }).collect();
    Ok(persons)
}

pub async fn upsert_dashboard(pool: &PgPool, dashboard: &Dashboard) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO dashboards (id, name, icon, views, created_at)
         VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (id) DO UPDATE SET
             name       = EXCLUDED.name,
             icon       = EXCLUDED.icon,
             views      = EXCLUDED.views",
    )
    .bind(&dashboard.id)
    .bind(&dashboard.name)
    .bind(&dashboard.icon)
    .bind(serde_json::to_value(&dashboard.views).unwrap_or_default())
    .bind(dashboard.created_at)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_dashboard(pool: &PgPool, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM dashboards WHERE id = $1").bind(id).execute(pool).await?;
    Ok(())
}

pub async fn load_all_dashboards(pool: &PgPool) -> Result<Vec<Dashboard>, sqlx::Error> {
    let rows = sqlx::query("SELECT id, name, icon, views, created_at FROM dashboards ORDER BY name")
        .fetch_all(pool)
        .await?;
    let dashboards = rows.iter().filter_map(|row| {
        let views_val: serde_json::Value = row.try_get("views").ok()?;
        Some(Dashboard {
            id:         row.try_get("id").ok()?,
            name:       row.try_get("name").ok()?,
            icon:       row.try_get("icon").ok()?,
            views:      serde_json::from_value(views_val).unwrap_or_default(),
            created_at: row.try_get("created_at").ok()?,
        })
    }).collect();
    Ok(dashboards)
}
