use moontool::moon::ToJSON;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct PhaseInfo {
    pub index: u8,
    pub name: String,
    pub icon: String,
}

#[derive(Debug, Clone, Deserialize)]
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
}

#[derive(Debug, Clone, Deserialize)]
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
pub struct MoonInfo {
    pub phase: MoonPhaseData,
    pub calendar: MoonCalendarData,
    // computed
    pub pct: f64,
    pub waxing: bool,
    pub day: usize,
    pub next_state: &'static str,
    pub time_remaining: String,
    pub current_phase_banner: String,
}

fn format_utc(s: &str) -> String {
    use chrono::{DateTime, Utc};
    s.parse::<DateTime<Utc>>()
        .map(|dt| dt.format("%A %d %b %Y %H:%M:%S").to_string())
        .unwrap_or_else(|_| s.to_string())
}

fn format_local(s: &str) -> String {
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

impl MoonInfo {
    pub fn compute() -> Self {
        let phase: moontool::moon::MoonPhase = moontool::moon::ForDateTime::now();
        let calendar: moontool::moon::MoonCalendar = moontool::moon::ForDateTime::now();

        let phase_data: MoonPhaseData =
            serde_json::from_str(&phase.to_json()).expect("failed to parse phase JSON");
        let calendar_data: MoonCalendarData =
            serde_json::from_str(&calendar.to_json()).expect("failed to parse calendar JSON");

        let pct = phase_data.fraction_illuminated * 100.0;
        let waxing = phase_data.fraction_of_lunation < 0.5;
        let day = (phase_data.age.round() as usize).clamp(1, 29);
        let next_state = if waxing {
            "Next Full Moon"
        } else {
            "Next New Moon"
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
            format!("in {}d {}h {}m", d, h, m)
        } else if h > 0 {
            format!("in {}h {}m", h, m)
        } else if m > 0 {
            format!("in {} minutes", m)
        } else {
            "now!".to_string()
        };

        let current_phase_banner = if pct <= 1.0 {
            "Current Moon Phase: === NEW MOON ===\n".to_string()
        } else if pct >= 99.0 {
            "Current Moon Phase: === FULL MOON ===\n".to_string()
        } else if pct >= 49.0 && pct <= 51.0 && waxing {
            "Current Moon Phase: === FIRST QUARTER ===\n".to_string()
        } else if pct >= 49.0 && pct <= 51.0 && !waxing {
            "Current Moon Phase: === LAST QUARTER ===\n".to_string()
        } else {
            String::new()
        };

        Self {
            phase: phase_data,
            calendar: calendar_data,
            pct,
            waxing,
            day,
            next_state,
            time_remaining,
            current_phase_banner,
        }
    }

    /// The main label shown in the popup.
    pub fn label(&self) -> String {
        format!(
            "{}{} {}.\nCurrent Moon size: {:.3}% {}",
            self.current_phase_banner,
            self.next_state,
            self.time_remaining,
            self.pct,
            if self.waxing {
                ", brightening(waxing)".to_string()
            } else {
                ", dimming(waning)".to_string()
            }
        )
    }

    /// The phase name from moontool (e.g. "Waxing Crescent").
    pub fn phase_name(&self) -> String {
        format!("{}", self.phase.phase.name)
    }

    /// Formatted calendar text (cleaned of special chars).
    pub fn calendar_text(&self) -> String {
        let cal: moontool::moon::MoonCalendar = moontool::moon::ForDateTime::now();
        let raw = format!("{}", cal);
        raw.chars()
            .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '\n' || *c == '\t')
            .collect()
    }

    pub fn phase_to_string(&self) -> String {
        let p = &self.phase;

        let total_minutes = (p.age * 24.0 * 60.0).round() as i64;
        let d = total_minutes / (24 * 60);
        let h = (total_minutes / 60) % 24;
        let m = total_minutes % 60;

        let age_str = if d > 0 {
            format!("{}d {}h {}m", d, h, m)
        } else if h > 0 {
            format!("{}h {}m", h, m)
        } else {
            format!("{} minutes", m)
        };

        format!(
            // "Julian Date:\t{:.6}\n\
            //  Timestamp:\t{}\n\
            "Age:                   \t{}\n\
             Phase Name:            \t{}\n\
             Ecliptic Longitude:    \t{:.2}°\n\
             Ecliptic Latitude:     \t{:.2}°\n\
             Parallax:              \t{:.2}°\n\
             Moon-Earth Distance:   \t{:.0} km, {:.1} Earth Radii:\n\
             Subtends:              \t{:.2}°",
            // p.julian_date,
            // p.timestamp,
            age_str,
            // p.fraction_of_lunation,
            // p.phase.index,
            p.phase.name,
            // p.phase.icon,
            // p.fraction_illuminated,
            p.ecliptic_longitude,
            p.ecliptic_latitude,
            p.parallax,
            p.distance_to_earth_km,
            p.distance_to_earth_earth_radii,
            p.subtends,
        )
    }

    pub fn calendar_to_string(&self) -> String {
        let c = &self.calendar;
        format!(
            // "Julian Date:\t{:.6}\n\
            // Timestamp:\t{}\n\
            "Local DateTime:\t{}\n\
                Last New Moon:\t{}\n\
                First Quarter:\t{}\n\
                Full Moon:\t{}\n\
                Last Quarter:\t{}\n\
                Next New Moon:\t{}",
            // c.julian_date,
            // c.timestamp,
            format_local(&c.utc_datetime),
            // c.lunation,
            format_local(&c.last_new_moon_utc),
            format_local(&c.first_quarter_utc),
            format_local(&c.full_moon_utc),
            format_local(&c.last_quarter_utc),
            format_local(&c.next_new_moon_utc),
        )
    }
}
