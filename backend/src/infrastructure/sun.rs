use chrono::NaiveTime;

use crate::domain::automation::SunEvent;

/// Returns the configured time for a sun event, read from env vars.
/// Defaults: SUNRISE_TIME=06:00, SUNSET_TIME=18:00 (suitable for Indonesia).
pub fn sun_event_time(event: &SunEvent) -> NaiveTime {
    let var = match event {
        SunEvent::Sunrise => "SUNRISE_TIME",
        SunEvent::Sunset  => "SUNSET_TIME",
    };
    let default = match event {
        SunEvent::Sunrise => "06:00",
        SunEvent::Sunset  => "18:00",
    };
    let raw = std::env::var(var).unwrap_or_else(|_| default.to_string());
    NaiveTime::parse_from_str(&raw, "%H:%M").unwrap_or_else(|_| {
        NaiveTime::parse_from_str(default, "%H:%M").unwrap()
    })
}
