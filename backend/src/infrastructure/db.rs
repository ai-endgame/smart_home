use sqlx::{postgres::PgPoolOptions, PgPool, Row};

use crate::domain::device::{Device, DeviceState, DeviceType};

pub async fn create_pool(url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new().max_connections(5).connect(url).await?;
    migrate(&pool).await?;
    log::info!("database: connected and schema ready");
    Ok(pool)
}

async fn migrate(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS devices (
            id          TEXT PRIMARY KEY,
            name        TEXT NOT NULL,
            device_type TEXT NOT NULL,
            state       TEXT NOT NULL DEFAULT 'off',
            room        TEXT,
            connected   BOOLEAN NOT NULL DEFAULT false,
            last_error  TEXT,
            brightness  SMALLINT NOT NULL DEFAULT 0,
            temperature DOUBLE PRECISION,
            created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn load_all_devices(pool: &PgPool) -> Result<Vec<Device>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT id, name, device_type, state, room, connected, \
         last_error, brightness, temperature \
         FROM devices ORDER BY name",
    )
    .fetch_all(pool)
    .await?;

    let devices = rows
        .iter()
        .filter_map(|row| {
            let device_type = parse_device_type(row.try_get("device_type").ok()?)?;
            let state_str: &str = row.try_get("state").ok()?;
            let state = if state_str == "on" { DeviceState::On } else { DeviceState::Off };
            let brightness: i16 = row.try_get("brightness").ok()?;
            Some(Device {
                id: row.try_get("id").ok()?,
                name: row.try_get("name").ok()?,
                device_type,
                state,
                room: row.try_get::<Option<String>, _>("room").ok()?,
                connected: row.try_get("connected").ok()?,
                last_error: row.try_get::<Option<String>, _>("last_error").ok()?,
                brightness: brightness as u8,
                temperature: row.try_get::<Option<f64>, _>("temperature").ok()?,
            })
        })
        .collect();

    Ok(devices)
}

pub async fn upsert_device(pool: &PgPool, device: &Device) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO devices
             (id, name, device_type, state, room, connected, last_error, brightness, temperature, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW())
         ON CONFLICT (id) DO UPDATE SET
             state       = EXCLUDED.state,
             room        = EXCLUDED.room,
             connected   = EXCLUDED.connected,
             last_error  = EXCLUDED.last_error,
             brightness  = EXCLUDED.brightness,
             temperature = EXCLUDED.temperature,
             updated_at  = NOW()",
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
    match s {
        "light" => Some(DeviceType::Light),
        "thermostat" => Some(DeviceType::Thermostat),
        "lock" => Some(DeviceType::Lock),
        "switch" => Some(DeviceType::Switch),
        "sensor" => Some(DeviceType::Sensor),
        _ => None,
    }
}

fn device_type_to_str(dt: &DeviceType) -> &'static str {
    match dt {
        DeviceType::Light => "light",
        DeviceType::Thermostat => "thermostat",
        DeviceType::Lock => "lock",
        DeviceType::Switch => "switch",
        DeviceType::Sensor => "sensor",
    }
}

fn state_to_str(s: &DeviceState) -> &'static str {
    match s { DeviceState::On => "on", DeviceState::Off => "off" }
}
