use chrono::{DateTime, Duration, NaiveDate, TimeZone, Utc};
use chrono_tz::Tz;

use crate::location_utils::{lookup_country_for_tz, lookup_zone_coords, resolve_tz_name};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MoonHorizonTimes {
    pub city: String,
    pub country: String,
    pub location_label: String,    
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
            country: "Unknown".to_string(),
            location_label: "Unknown".to_string(),
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



pub fn compute_moon_rise_set(manual_tz: Option<&str>) -> MoonHorizonTimes {
    let tz_name = resolve_tz_name(manual_tz);
    let Ok(tz) = tz_name.parse::<Tz>() else {
        return MoonHorizonTimes::default();
    };
    let Some((lat, lon)) = lookup_zone_coords(&tz_name) else {
        return MoonHorizonTimes::default();
    };

    let city = tz_name.split('/').last().unwrap_or(&tz_name).replace('_', " ");
    let country = lookup_country_for_tz(&tz_name);
    let location_label = format!("{}, {}", city, country);

    let today = Utc::now().with_timezone(&tz).date_naive();
    let tomorrow = today + Duration::days(1);

    let (today_moonrise, today_moonset) = rise_set_for_local_date(today, tz, lat, lon, true);
    let (tomorrow_moonrise, tomorrow_moonset) = rise_set_for_local_date(tomorrow, tz, lat, lon, true);
    let (today_sunrise, today_sunset) = rise_set_for_local_date(today, tz, lat, lon, false);
    let (tomorrow_sunrise, tomorrow_sunset) = rise_set_for_local_date(tomorrow, tz, lat, lon, false);

    MoonHorizonTimes {
        city,
        country,
        location_label,
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
    tz: Tz,
    lat_deg: f64,
    lon_deg: f64,
    is_moon: bool,
) -> (String, String) {
    let start_local = tz
        .from_local_datetime(&date.and_hms_opt(0, 0, 0).unwrap())
        .earliest()
        .unwrap_or_else(|| Utc::now().with_timezone(&tz));

    // Topocentric upper limb at horizon:
    // Refraction (-0.5667°) - average lunar semidiameter (-0.2725°) ≈ -0.839°
    let h0 = if is_moon { -0.839 } else { -0.833 };


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
    // 1. Julian Day (UT) and Julian Centuries (TT)
    // Use timestamp_millis() for sub-second precision
    let jd_ut = (utc.timestamp_millis() as f64) / 86_400_000.0 + 2440587.5;
    let d_ut = jd_ut - 2451545.0;

    // Approximate Delta-T (~69.2s in 2026) to convert UT to Terrestrial Time (TT)
    let delta_t_days = 69.2 / 86400.0;
    let t = (d_ut + delta_t_days) / 36525.0;
    let t2 = t * t;
    let t3 = t2 * t;
    let t4 = t3 * t;

    // 2. Fundamental Lunar & Solar Arguments (Meeus Ch. 47, in degrees -> radians)
    let l_prime = (218.3164477 + 481267.88123421 * t - 0.0015786 * t2 + t3 / 538841.0
        - t4 / 65194000.0)
        .rem_euclid(360.0)
        .to_radians(); // Moon's mean longitude

    let d = (297.8501921 + 445267.1114034 * t - 0.0018819 * t2 + t3 / 545868.0
        - t4 / 113065000.0)
        .rem_euclid(360.0)
        .to_radians(); // Mean elongation of the Moon

    let m = (357.5291092 + 35999.0502909 * t - 0.0001536 * t2 + t3 / 24490000.0)
        .rem_euclid(360.0)
        .to_radians(); // Sun's mean anomaly

    let m_prime = (134.9633964 + 477198.8675055 * t + 0.0087414 * t2 + t3 / 69699.0
        - t4 / 14712000.0)
        .rem_euclid(360.0)
        .to_radians(); // Moon's mean anomaly

    let f = (93.2720950 + 483202.0175233 * t - 0.0036539 * t2 - t3 / 3526000.0
        + t4 / 863310000.0)
        .rem_euclid(360.0)
        .to_radians(); // Moon's argument of latitude

    // Eccentricity of Earth's orbit around the Sun
    let e = 1.0 - 0.002516 * t - 0.0000074 * t2;
    let e2 = e * e;

    // 3. Periodic Terms for Longitude (Sigma_l) and Distance (Sigma_r)
    // Coefficients from Meeus Table 47.A (top ~20 terms give < 0.01° accuracy)
    let lon_terms: &[(f64, f64, f64, f64, f64, f64)] = &[
        // (D, M, M', F, Sigma_l [0.000001 deg], Sigma_r [0.001 km])
        (0.0, 0.0, 1.0, 0.0, 6288774.0, -20905355.0),
        (2.0, 0.0, -1.0, 0.0, 1274027.0, -3699111.0),
        (2.0, 0.0, 0.0, 0.0, 658314.0, -2955968.0),
        (0.0, 0.0, 2.0, 0.0, 213618.0, -569925.0),
        (0.0, 1.0, 0.0, 0.0, -185116.0, 48888.0),
        (0.0, 0.0, 0.0, 2.0, -114332.0, -3149.0),
        (2.0, 0.0, -2.0, 0.0, 58793.0, 246158.0),
        (2.0, -1.0, -1.0, 0.0, 57066.0, -152138.0),
        (2.0, 0.0, 1.0, 0.0, 53322.0, -170733.0),
        (2.0, -1.0, 0.0, 0.0, 45758.0, -204586.0),
        (0.0, 1.0, -1.0, 0.0, -40923.0, -129620.0),
        (1.0, 0.0, 0.0, 0.0, -34720.0, 108743.0),
        (0.0, 1.0, 1.0, 0.0, -30383.0, 104755.0),
        (2.0, 0.0, 0.0, -2.0, 15327.0, 10321.0),
        (0.0, 0.0, 1.0, 2.0, -12528.0, 0.0),
        (0.0, 0.0, 1.0, -2.0, 10980.0, 79661.0),
        (4.0, 0.0, -1.0, 0.0, 10675.0, -34782.0),
        (0.0, 0.0, 3.0, 0.0, 10034.0, -23210.0),
        (4.0, 0.0, -2.0, 0.0, 8548.0, -21636.0),
        (2.0, 1.0, -1.0, 0.0, -7888.0, 24208.0),
    ];

    let mut sigma_l = 0.0;
    let mut sigma_r = 0.0;
    for &(cd, cm, cmp, cf, sl, sr) in lon_terms {
        let arg = cd * d + cm * m + cmp * m_prime + cf * f;
        let e_factor = match cm.abs() as i32 {
            1 => e,
            2 => e2,
            _ => 1.0,
        };
        sigma_l += sl * e_factor * arg.sin();
        sigma_r += sr * e_factor * arg.cos();
    }

    // 4. Periodic Terms for Latitude (Sigma_b) from Meeus Table 47.B
    let lat_terms: &[(f64, f64, f64, f64, f64)] = &[
        // (D, M, M', F, Sigma_b [0.000001 deg])
        (0.0, 0.0, 0.0, 1.0, 5128122.0),
        (0.0, 0.0, 1.0, 1.0, 280602.0),
        (0.0, 0.0, 1.0, -1.0, 277693.0),
        (2.0, 0.0, 0.0, -1.0, 173237.0),
        (2.0, 0.0, -1.0, 1.0, 55413.0),
        (2.0, 0.0, -1.0, -1.0, 46271.0),
        (2.0, 0.0, 0.0, 1.0, 32573.0),
        (0.0, 0.0, 2.0, 1.0, 17198.0),
        (2.0, 0.0, 1.0, -1.0, 9266.0),
        (0.0, 0.0, 2.0, -1.0, 8822.0),
        (2.0, -1.0, 0.0, -1.0, 8216.0),
        (2.0, 0.0, -2.0, -1.0, 4324.0),
    ];

    let mut sigma_b = 0.0;
    for &(cd, cm, cmp, cf, sb) in lat_terms {
        let arg = cd * d + cm * m + cmp * m_prime + cf * f;
        let e_factor = match cm.abs() as i32 {
            1 => e,
            2 => e2,
            _ => 1.0,
        };
        sigma_b += sb * e_factor * arg.sin();
    }

    // Geocentric ecliptic coordinates & distance (km)
    let lon_ecl = l_prime + (sigma_l / 1_000_000.0).to_radians();
    let lat_ecl = (sigma_b / 1_000_000.0).to_radians();
    let dist_km = 385000.56 + (sigma_r / 1000.0);

    // Mean obliquity of the ecliptic (Meeus eq. 22.2)
    let obl = (23.4392911 - 0.0130042 * t - 1.64e-7 * t2 + 5.04e-7 * t3).to_radians();

    // 5. Geocentric Equatorial Coordinates (RA, Dec)
    let ra = (lon_ecl.sin() * obl.cos() - lat_ecl.tan() * obl.sin()).atan2(lon_ecl.cos());
    let dec = (lat_ecl.sin() * obl.cos() + lat_ecl.cos() * obl.sin() * lon_ecl.sin()).asin();

    // 6. Greenwich Mean Sidereal Time (GMST) & Geocentric Hour Angle (Meeus eq. 12.4)
    let t_ut = d_ut / 36525.0;
    let gmst_deg = (280.46061837
        + 360.98564736629 * d_ut
        + 0.000387933 * t_ut * t_ut
        - (t_ut * t_ut * t_ut) / 38710000.0)
        .rem_euclid(360.0);

    let lst = (gmst_deg + lon_deg).to_radians();
    let ha = lst - ra;

    // 7. Topocentric Parallax Correction (Meeus Ch. 40, eq. 40.2 & 40.3)
    // Equatorial horizontal parallax: sin(pi) = R_earth / dist_km
    let sin_pi = 6378.14 / dist_km;

    // Geocentric latitude constants (WGS84 flattening f = 1 / 298.257223563)
    let lat = lat_deg.to_radians();
    let f_flat = 1.0 / 298.257223563;
    let u = ((1.0 - f_flat) * lat.tan()).atan();
    let rho_sin_phi = (1.0 - f_flat) * u.sin();
    let rho_cos_phi = u.cos();

    let delta_ra = (-rho_cos_phi * sin_pi * ha.sin())
        .atan2(dec.cos() - rho_cos_phi * sin_pi * ha.cos());

    let ha_topo = ha - delta_ra;
    let dec_topo = ((dec.sin() - rho_sin_phi * sin_pi) * delta_ra.cos())
        .atan2(dec.cos() - rho_cos_phi * sin_pi * ha.cos());

    // 8. Topocentric Altitude
    let sin_alt = lat.sin() * dec_topo.sin() + lat.cos() * dec_topo.cos() * ha_topo.cos();
    sin_alt.asin().to_degrees()
}




