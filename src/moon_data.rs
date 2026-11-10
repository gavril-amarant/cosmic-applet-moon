use moontool::moon::ToJSON;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct PhaseInfo {
    pub index: u8,
    pub name: String,
    pub icon: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct MoonPhaseData {
    pub julian_date: f64,
    pub timestamp: i64,
    pub utc_datetime: String,
    pub age: f64,
    pub fraction_of_lunation: f64,
    pub phase: PhaseInfo,
    pub fraction_illuminated: f64,
    pub distance_to_earth_km: f64,
    pub subtends: f64,
    pub ecliptic_longitude: f64,
    pub ecliptic_latitude: f64,
    pub parallax: f64,
    pub distance_to_earth_earth_radii: f64,
    pub sun_ecliptic_longitude: f64,
    pub sun_distance_to_earth_km: f64,
    pub sun_distance_to_earth_astronomical_units: f64,
    pub sun_subtends: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct MoonCalendarData {
    pub julian_date: f64,
    pub timestamp: i64,
    pub utc_datetime: String,
    pub lunation: u32,
    pub last_new_moon: f64,
    pub last_new_moon_utc: String,
    pub first_quarter: f64,
    pub first_quarter_utc: String,
    pub full_moon: f64,
    pub full_moon_utc: String,
    pub last_quarter: f64,
    pub last_quarter_utc: String,
    pub next_new_moon: f64,
    pub next_new_moon_utc: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MoonInfo {
    pub phase: MoonPhaseData,
    pub calendar: MoonCalendarData,
    // computed
    pub illumination: f64,
    pub waxing: bool,
    pub day: usize,
    pub next_state: &'static str,
    pub time_remaining: String,
}
#[allow(dead_code)]
pub fn format_utc(s: &str) -> String {
    use chrono::{DateTime, Utc};
    s.parse::<DateTime<Utc>>()
        .map(|dt| dt.format("%A %d %b %Y %H:%M:%S").to_string())
        .unwrap_or_else(|_| s.to_string())
}

pub fn format_local(s: &str) -> String {
    use chrono::{DateTime, Local, Utc};
    s.parse::<DateTime<Utc>>()
        .map(|dt| {
            let local: DateTime<Local> = dt.with_timezone(&Local);
            format!(
                "{} ({})",
                local.format("%A, %d %b %Y, %H:%M:%S"),
                local.format("%Z")
            )
        })
        .unwrap_or_else(|_| s.to_string())
}

pub fn format_local_short(s: &str) -> String {
    use chrono::{DateTime, Local, Utc};
    s.parse::<DateTime<Utc>>()
        .map(|dt| {
            let local: DateTime<Local> = dt.with_timezone(&Local);
            format!(
                "{}",
                local.format("%A, %d %b %Y")
            )
        })
        .unwrap_or_else(|_| s.to_string())
}

impl MoonInfo {
    pub fn compute() -> Self {
        let phase: moontool::moon::MoonPhase = moontool::moon::ForDateTime::now();
        let calendar: moontool::moon::MoonCalendar = moontool::moon::ForDateTime::now();

        let phase_data: MoonPhaseData =
            serde_json::from_str(&phase.to_json()).expect("failed to parse phase JSON");
        let calendar_data: MoonCalendarData =
            serde_json::from_str(&calendar.to_json()).expect("failed to parse calendar JSON");

        let illumination = phase_data.fraction_illuminated * 100.0;
        let waxing = phase_data.fraction_of_lunation < 0.5;
        let day = (phase_data.age.round() as usize).clamp(1, 29);
        let next_state = if waxing {
            "Next Full Moon"
        } else {
            "Next New Moon"
        };

        let next_state_date = if waxing {
            format_local_short(&calendar_data.full_moon_utc)
        } else {
            format_local_short(&calendar_data.next_new_moon_utc)
        };

        let current_jd = calendar_data.julian_date;
        let target_jd = if waxing {
            calendar_data.full_moon
        } else {
            calendar_data.next_new_moon
        };
        let days_remaining = target_jd - current_jd;
        let total_minutes = (days_remaining * 24.0 * 60.0).round() as i64;
        let d = total_minutes / (24 * 60);
        let h = (total_minutes / 60) % 24;
        let m = total_minutes % 60;

        let time_remaining = if d > 0 {
            format!("in {}d {}h {}m on {}", d, h, m, next_state_date)
        } else if h > 0 {
            format!("in {}h {}m on {}", h, m, next_state_date)
        } else if m > 0 {
            format!("in {} minutes", m)
        } else {
            "now!".to_string()
        };

        Self {
            phase: phase_data,
            calendar: calendar_data,
            illumination,
            waxing,
            day,
            next_state,
            time_remaining,
        }
    }

}
