use cosmic::iced::widget::image::Handle;
use chrono::{DateTime, Local, Utc};


pub const APP_ID: &str = "io.github.gavril_amarant.cosmic-applet-moon";
pub const APP_VERSION: &str = "v1.0.2";
pub const APP_HOMEPAGE: &str = "https://github.com/gavril-amarant/cosmic-applet-moon?tab=readme-ov-file#readme-ov-file";

// ── Icon loading ─────────────────────────────────────────────

macro_rules! moon_icon_bytes {
    ($day:expr, $($n:expr => $file:expr),*) => {
        match $day {
            $($n => include_bytes!($file).as_slice(),)*
            _ => include_bytes!("../assets/icons/moon-1-NM.png").as_slice(),
        }
    };
}

pub fn get_moon_phase_bytes(day: usize) -> &'static [u8] {
    moon_icon_bytes!(day,
        1 => "../assets/icons/moon-1-NM.png",
        2 => "../assets/icons/moon-2.png",
        3 => "../assets/icons/moon-3.png",
        4 => "../assets/icons/moon-4.png",
        5 => "../assets/icons/moon-5.png",
        6 => "../assets/icons/moon-6.png",
        7 => "../assets/icons/moon-7.png",
        8 => "../assets/icons/moon-8-FQ.png",
        9 => "../assets/icons/moon-9.png",
        10 => "../assets/icons/moon-10.png",
        11 => "../assets/icons/moon-11.png",
        12 => "../assets/icons/moon-12.png",
        13 => "../assets/icons/moon-13.png",
        14 => "../assets/icons/moon-14.png",
        15 => "../assets/icons/moon-15-FM.png",
        16 => "../assets/icons/moon-16.png",
        17 => "../assets/icons/moon-17.png",
        18 => "../assets/icons/moon-18.png",
        19 => "../assets/icons/moon-19.png",
        20 => "../assets/icons/moon-20.png",
        21 => "../assets/icons/moon-21.png",
        22 => "../assets/icons/moon-22-LQ.png",
        23 => "../assets/icons/moon-23.png",
        24 => "../assets/icons/moon-24.png",
        25 => "../assets/icons/moon-25.png",
        26 => "../assets/icons/moon-26.png",
        27 => "../assets/icons/moon-27.png",
        28 => "../assets/icons/moon-28.png",
        29 => "../assets/icons/moon-29.png"
    )
}

pub fn get_moon_phase_image(day: usize) -> Handle {
    Handle::from_bytes(get_moon_phase_bytes(day).to_vec())
}

// ── Formatting helpers ───────────────────────────────────────

pub fn format_age(days: f64) -> String {
    let total_minutes = (days * 24.0 * 60.0).round() as i64;
    let d = total_minutes / (24 * 60);
    let h = (total_minutes / 60) % 24;
    let m = total_minutes % 60;
    if d > 0 {
        format!("{}d {}h {}m", d, h, m)
    } else if h > 0 {
        format!("{}h {}m", h, m)
    } else if m > 0 {
        format!("{} minutes", m)
    } else {
        "now!".to_string()
    }
}
pub fn format_in_tz(s: &str, tz_name: &str) -> String {
    let tz: chrono_tz::Tz = tz_name.parse().unwrap_or(chrono_tz::Tz::UTC);
    s.parse::<DateTime<Utc>>()
        .map(|dt| {
            let local = dt.with_timezone(&tz);
            format!("{} ({})", local.format("%A, %d %b %Y, %H:%M:%S"), local.format("%Z"))
        })
        .unwrap_or_else(|_| s.to_string())
}

#[allow(dead_code)]
pub fn format_local(s: &str) -> String {
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