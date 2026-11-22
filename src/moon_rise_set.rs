use chrono::{DateTime, Duration, Local, NaiveDate, TimeZone, Utc};

const ZONE_TAB: &str = include_str!("../assets/zone1970.tab");

#[derive(Debug, Clone)]
pub struct MoonHorizonTimes {
    pub city: String,
    pub today_moonrise: String,
    pub today_moonset: String,
    pub tomorrow_moonrise: String,
    pub tomorrow_moonset: String,
    pub today_sunrise: String,
    pub today_sunset: String,
    pub tomorrow_sunrise: String,
    pub tomorrow_sunset: String,
}

impl Default for MoonHorizonTimes {
    fn default() -> Self {
        Self {
            city: "Unknown".to_string(),
            today_moonrise: "—".to_string(),
            today_moonset: "—".to_string(),
            tomorrow_moonrise: "—".to_string(),
            tomorrow_moonset: "—".to_string(),
            today_sunrise: "—".to_string(),
            today_sunset: "—".to_string(),
            tomorrow_sunrise: "—".to_string(),
            tomorrow_sunset: "—".to_string(),
        }
    }
}

pub fn compute_moon_rise_set() -> MoonHorizonTimes {
    let Ok(tz_name) = iana_time_zone::get_timezone() else {
        return MoonHorizonTimes::default();
    };

    let Some((lat, lon)) = lookup_zone_coords(&tz_name) else {
        return MoonHorizonTimes::default();
    };

    let city = tz_name
        .split('/')
        .last()
        .unwrap_or(&tz_name)
        .replace('_', " ");

    let today = Local::now().date_naive();
    let tomorrow = today + Duration::days(1);

    let (today_moonrise, today_moonset) = rise_set_for_local_date(today, lat, lon, true);
    let (tomorrow_moonrise, tomorrow_moonset) = rise_set_for_local_date(tomorrow, lat, lon, true);

    let (today_sunrise, today_sunset) = rise_set_for_local_date(today, lat, lon, false);
    let (tomorrow_sunrise, tomorrow_sunset) = rise_set_for_local_date(tomorrow, lat, lon, false);

    MoonHorizonTimes {
        city,
        today_moonrise,
        today_moonset,
        tomorrow_moonrise,
        tomorrow_moonset,
        today_sunrise,
        today_sunset,
        tomorrow_sunrise,
        tomorrow_sunset,
    }
}

fn rise_set_for_local_date(
    date: NaiveDate,
    lat_deg: f64,
    lon_deg: f64,
    is_moon: bool,
) -> (String, String) {
    let start_local = Local
        .from_local_datetime(&date.and_hms_opt(0, 0, 0).unwrap())
        .earliest()
        .unwrap_or_else(Local::now);

    // Moon center at horizon: +0.125° | Sun upper limb at horizon: -0.833°
    let h0 = if is_moon { 0.125 } else { -0.833 };

    let eval_alt = |utc: DateTime<Utc>| -> f64 {
        if is_moon {
            moon_altitude_deg(utc, lat_deg, lon_deg) - h0
        } else {
            sun_altitude_deg(utc, lat_deg, lon_deg) - h0
        }
    };

    let mut rise = "—".to_string();
    let mut set = "—".to_string();
    let mut prev_alt = eval_alt(start_local.with_timezone(&Utc));

    for step in 1..=288 {
        let t1 = start_local + Duration::minutes(step * 5);
        let alt = eval_alt(t1.with_timezone(&Utc));

        if prev_alt < 0.0 && alt >= 0.0 && rise == "—" {
            let frac = -prev_alt / (alt - prev_alt);
            let mins = ((step - 1) * 5) as f64 + frac * 5.0;
            let dt = start_local + Duration::seconds((mins * 60.0).round() as i64);
            rise = dt.format("%H:%M").to_string();
        } else if prev_alt >= 0.0 && alt < 0.0 && set == "—" {
            let frac = prev_alt / (prev_alt - alt);
            let mins = ((step - 1) * 5) as f64 + frac * 5.0;
            let dt = start_local + Duration::seconds((mins * 60.0).round() as i64);
            set = dt.format("%H:%M").to_string();
        }

        prev_alt = alt;
    }

    (rise, set)
}

fn sun_altitude_deg(utc: DateTime<Utc>, lat_deg: f64, lon_deg: f64) -> f64 {
    let jd = (utc.timestamp() as f64) / 86400.0 + 2440587.5;
    let d = jd - 2451545.0;

    // Mean anomaly and mean longitude of the Sun (Meeus Ch. 25)
    let g = (357.529 + 0.98560028 * d).to_radians();
    let q = 280.459 + 0.98564736 * d;
    let lon_ecl = (q + 1.915 * g.sin() + 0.020 * (2.0 * g).sin()).to_radians();
    let obl = (23.439 - 0.00000036 * d).to_radians();

    let ra = (obl.cos() * lon_ecl.sin()).atan2(lon_ecl.cos());
    let dec = (obl.sin() * lon_ecl.sin()).asin();

    let gmst_deg = (280.46061837 + 360.98564736629 * d).rem_euclid(360.0);
    let lst = (gmst_deg + lon_deg).to_radians();
    let ha = lst - ra;

    let lat = lat_deg.to_radians();
    let sin_alt = lat.sin() * dec.sin() + lat.cos() * dec.cos() * ha.cos();
    sin_alt.asin().to_degrees()
}

fn moon_altitude_deg(utc: DateTime<Utc>, lat_deg: f64, lon_deg: f64) -> f64 {
    let jd = (utc.timestamp() as f64) / 86400.0 + 2440587.5;
    let d = jd - 2451545.0;

    let l_prime = (218.316 + 13.176396 * d).to_radians();
    let m_moon = (134.963 + 13.064993 * d).to_radians();
    let f = (93.272 + 13.229350 * d).to_radians();

    let lon_ecl = l_prime + (6.289_f64.to_radians()) * m_moon.sin();
    let lat_ecl = (5.128_f64.to_radians()) * f.sin();
    let obl = (23.439 - 0.0000004 * d).to_radians();

    let ra = (lon_ecl.sin() * obl.cos() - lat_ecl.tan() * obl.sin()).atan2(lon_ecl.cos());
    let dec = (lat_ecl.sin() * obl.cos() + lat_ecl.cos() * obl.sin() * lon_ecl.sin()).asin();

    let gmst_deg = (280.46061837 + 360.98564736629 * d).rem_euclid(360.0);
    let lst = (gmst_deg + lon_deg).to_radians();
    let ha = lst - ra;

    let lat = lat_deg.to_radians();
    let sin_alt = lat.sin() * dec.sin() + lat.cos() * dec.cos() * ha.cos();
    sin_alt.asin().to_degrees()
}

fn lookup_zone_coords(target_tz: &str) -> Option<(f64, f64)> {
    for line in ZONE_TAB.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut cols = line.split('\t');
        let _countries = cols.next()?;
        let coords = cols.next()?;
        let tz = cols.next()?;
        if tz == target_tz {
            return parse_iso6709(coords);
        }
    }
    None
}

fn parse_iso6709(s: &str) -> Option<(f64, f64)> {
    let split_idx = s[1..].find(['+', '-'])? + 1;
    Some((parse_dms(&s[..split_idx], 2)?, parse_dms(&s[split_idx..], 3)?))
}

fn parse_dms(part: &str, deg_digits: usize) -> Option<f64> {
    let sign = if part.starts_with('-') { -1.0 } else { 1.0 };
    let digits = &part[1..];
    let deg: f64 = digits.get(0..deg_digits)?.parse().ok()?;
    let min: f64 = digits.get(deg_digits..deg_digits + 2)?.parse().ok()?;
    let sec: f64 = digits
        .get(deg_digits + 2..deg_digits + 4)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0);
    Some(sign * (deg + min / 60.0 + sec / 3600.0))
}
