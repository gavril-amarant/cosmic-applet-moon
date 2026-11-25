use crate::{location_utils::{get_observer_coords, resolve_tz_name}, moon_horizon::{MoonHorizonTimes, compute_moon_rise_set}, utils::format_age};
use chrono::{DateTime, TimeZone, Utc};

// ── Data structures (exact same fields as before) ────────────

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PhaseInfo {
    pub index: u8,
    pub name: String,
    pub icon: String,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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
    pub illumination: f64,
    pub growing: bool,
    pub day: usize,
    pub next_state: &'static str,
    pub next_state_tooltip: &'static str,
    pub time_remaining: String,
    pub moon_horizon: MoonHorizonTimes,
    pub earths_between_moon: f64,
    pub moons_between_moon: f64,
    pub earths_between_sun: f64,
    pub suns_between_sun: f64, 
    pub tz_name: String,   
}

// ── Computation ──────────────────────────────────────────────

impl MoonInfo {
    pub fn compute(manual_tz: Option<&str>) -> Self {
        const R_EARTH: f64 = 6_378.137;
        const R_MOON: f64 = 1_737.4;
        const R_SUN: f64 = 695_700.0;


        let now = Utc::now();
        let tz_name = resolve_tz_name(manual_tz);
        let (lat_deg, lon_deg) = get_observer_coords(manual_tz);

        let calendar_data = compute_calendar(now);
        let phase_data = compute_phase(now, &calendar_data, lat_deg, lon_deg);

        let illumination = phase_data.fraction_illuminated * 100.0;
        let growing = phase_data.fraction_of_lunation < 0.5;
        let day = (phase_data.age.round() as usize).clamp(1, 29);
        let next_state = if growing {
            "Full Moon in:"
        } else {
            "New Moon in:"
        };
        let next_state_tooltip = if growing {
            "Time left until 100% illumination"
        } else {
            "Time left until 0% illumination"
        };
        let current_jd = calendar_data.julian_date;
        let target_jd = if growing {
            calendar_data.full_moon
        } else {
            calendar_data.next_new_moon
        };

        let moon_horizon = compute_moon_rise_set(manual_tz);


        let moon_gap_km = (phase_data.distance_to_earth_km - R_EARTH - R_MOON).max(0.0);
        let earths_between_moon = moon_gap_km / (2.0 * R_EARTH);
        let moons_between_moon = moon_gap_km / (2.0 * R_MOON);

        let sun_gap_km = (phase_data.sun_distance_to_earth_km - R_EARTH - R_SUN).max(0.0);
        let earths_between_sun = sun_gap_km / (2.0 * R_EARTH);
        let suns_between_sun = sun_gap_km / (2.0 * R_SUN);


        Self {
            phase: phase_data,
            calendar: calendar_data,
            illumination,
            growing,
            day,
            next_state,
            next_state_tooltip,
            time_remaining: format_age(target_jd - current_jd),
            moon_horizon,
            earths_between_moon,
            moons_between_moon,
            earths_between_sun,
            suns_between_sun,
            tz_name,
        }
    }
}

// ── High-Precision Meeus Ephemeris Helpers ───────────────────

const EARTH_RADIUS_KM: f64 = 6378.137;
const AU_KM: f64 = 149_597_870.7;
const SYNODIC_MONTH: f64 = 29.530588861;
// ΔT ≈ TT - UTC in days (~69.2 seconds in 2026)
const DELTA_T_DAYS: f64 = 69.2 / 86400.0;

fn datetime_to_jd(dt: DateTime<Utc>) -> f64 {
    let secs = dt.timestamp() as f64 + (dt.timestamp_subsec_nanos() as f64) * 1e-9;
    secs / 86400.0 + 2440587.5
}

fn jd_to_datetime(jd: f64) -> DateTime<Utc> {
    let unix_secs = (jd - 2440587.5) * 86400.0;
    let secs = unix_secs.floor() as i64;
    let nanos = ((unix_secs - secs as f64) * 1e9).round() as u32;
    Utc.timestamp_opt(secs, nanos.min(999_999_999))
        .single()
        .unwrap_or_else(Utc::now)
}

fn jd_to_iso(jd: f64) -> String {
    jd_to_datetime(jd).to_rfc3339()
}

/// Helper: returns the next occurrence strictly after `jd_now` for `phase` (0=New, 1=FQ, 2=Full, 3=LQ).
fn next_phase_after(k_start: f64, phase: u8, jd_now: f64) -> f64 {
    let mut k = k_start;
    let mut jd = true_phase_jd(k, phase);
    while jd <= jd_now {
        k += 1.0;
        jd = true_phase_jd(k, phase);
    }
    jd
}

/// Computes exact New, First Quarter, Full, and Last Quarter JDs using Meeus Ch. 49.
fn compute_calendar(now: DateTime<Utc>) -> MoonCalendarData {
    let jd_now = datetime_to_jd(now);
    let year_approx = 2000.0 + (jd_now - 2451545.0) / 365.25;
    let mut k = ((year_approx - 2000.0) * 12.3685).floor() - 1.0;

    // Step forward until `k` is the lunation whose New Moon is `<= jd_now` (for accurate lunar age)
    let mut last_new = true_phase_jd(k, 0);
    while true_phase_jd(k + 1.0, 0) <= jd_now {
        k += 1.0;
        last_new = true_phase_jd(k, 0);
    }

    // Each upcoming phase independently advances to `k + 1.0` if it already occurred in lunation `k`
    let first_q = next_phase_after(k, 1, jd_now);
    let full = next_phase_after(k, 2, jd_now);
    let last_q = next_phase_after(k, 3, jd_now);
    let next_new = next_phase_after(k, 0, jd_now);

    // Meeus lunation number (Lunation 1 = Jan 17, 1923; Lunation 953 = Jan 6, 2000)
    let lunation = (k.round() as i64 + 953).max(0) as u32;

    MoonCalendarData {
        julian_date: jd_now,
        timestamp: now.timestamp(),
        utc_datetime: now.to_rfc3339(),
        lunation,
        last_new_moon: last_new,
        last_new_moon_utc: jd_to_iso(last_new),
        first_quarter: first_q,
        first_quarter_utc: jd_to_iso(first_q),
        full_moon: full,
        full_moon_utc: jd_to_iso(full),
        last_quarter: last_q,
        last_quarter_utc: jd_to_iso(last_q),
        next_new_moon: next_new,
        next_new_moon_utc: jd_to_iso(next_new),
    }
}

/// Meeus Chapter 49: True phase timestamp in UTC JD for lunation `k` and `phase` (0..=3).
fn true_phase_jd(k_base: f64, phase: u8) -> f64 {
    let k = k_base + (phase as f64) * 0.25;
    let t = k / 1236.85;
    let t2 = t * t;
    let t3 = t2 * t;
    let t4 = t3 * t;

    // Mean phase JDE
    let mut jde =
        2451550.09766 + SYNODIC_MONTH * k + 0.00015437 * t2 - 0.000000150 * t3 + 0.00000000073 * t4;

    let e = 1.0 - 0.002516 * t - 0.0000074 * t2;
    let e2 = e * e;

    let m = (2.5534 + 29.10535670 * k - 0.0000014 * t2 - 0.00000011 * t3).to_radians();
    let mp = (201.5643 + 385.81693528 * k + 0.0107582 * t2 + 0.00001238 * t3 - 0.000000058 * t4)
        .to_radians();
    let f = (160.7108 + 390.67050284 * k - 0.0016118 * t2 - 0.00000227 * t3 + 0.000000011 * t4)
        .to_radians();
    let omega = (124.7746 - 1.56375588 * k + 0.0020672 * t2 + 0.00000215 * t3).to_radians();

    let corr = match phase {
        0 => {
            // New Moon
            -0.40720 * mp.sin()
                + 0.17241 * e * m.sin()
                + 0.01608 * (2.0 * mp).sin()
                + 0.01039 * (2.0 * f).sin()
                + 0.00739 * e * (mp - m).sin()
                - 0.00514 * e * (mp + m).sin()
                + 0.00208 * e2 * (2.0 * m).sin()
                - 0.00111 * (mp - 2.0 * f).sin()
                - 0.00057 * (mp + 2.0 * f).sin()
                + 0.00056 * e * (2.0 * mp + m).sin()
                - 0.00042 * (3.0 * mp).sin()
                + 0.00042 * e * (m + 2.0 * f).sin()
                + 0.00038 * e * (m - 2.0 * f).sin()
                - 0.00024 * e * (2.0 * mp - m).sin()
                - 0.00017 * omega.sin()
                - 0.00007 * (mp + 2.0 * m).sin()
                + 0.00004 * (2.0 * mp - 2.0 * f).sin()
                + 0.00004 * (3.0 * m).sin()
                + 0.00003 * (mp + m - 2.0 * f).sin()
                + 0.00003 * (2.0 * mp + 2.0 * f).sin()
                - 0.00003 * (mp + m + 2.0 * f).sin()
                + 0.00003 * (mp - m + 2.0 * f).sin()
                - 0.00002 * (mp - m - 2.0 * f).sin()
                - 0.00002 * (3.0 * mp + m).sin()
                + 0.00002 * (4.0 * mp).sin()
        }
        2 => {
            // Full Moon
            -0.40614 * mp.sin()
                + 0.17302 * e * m.sin()
                + 0.01614 * (2.0 * mp).sin()
                + 0.01043 * (2.0 * f).sin()
                + 0.00734 * e * (mp - m).sin()
                - 0.00515 * e * (mp + m).sin()
                + 0.00209 * e2 * (2.0 * m).sin()
                - 0.00111 * (mp - 2.0 * f).sin()
                - 0.00057 * (mp + 2.0 * f).sin()
                + 0.00056 * e * (2.0 * mp + m).sin()
                - 0.00042 * (3.0 * mp).sin()
                + 0.00042 * e * (m + 2.0 * f).sin()
                + 0.00038 * e * (m - 2.0 * f).sin()
                - 0.00024 * e * (2.0 * mp - m).sin()
                - 0.00017 * omega.sin()
                - 0.00007 * (mp + 2.0 * m).sin()
                + 0.00004 * (2.0 * mp - 2.0 * f).sin()
                + 0.00004 * (3.0 * m).sin()
                + 0.00003 * (mp + m - 2.0 * f).sin()
                + 0.00003 * (2.0 * mp + 2.0 * f).sin()
                - 0.00003 * (mp + m + 2.0 * f).sin()
                + 0.00003 * (mp - m + 2.0 * f).sin()
                - 0.00002 * (mp - m - 2.0 * f).sin()
                - 0.00002 * (3.0 * mp + m).sin()
                + 0.00002 * (4.0 * mp).sin()
        }
        _ => {
            // First Quarter (1) and Last Quarter (3)
            let w_sign = if phase == 1 { 1.0 } else { -1.0 };
            let w = 0.00306 - 0.00038 * e * m.cos() + 0.00026 * mp.cos() - 0.00002 * (mp - m).cos()
                + 0.00002 * (mp + m).cos()
                + 0.00002 * (2.0 * f).cos();

            -0.62801 * mp.sin() + 0.17172 * e * m.sin() - 0.01183 * e * (mp + m).sin()
                + 0.00862 * (2.0 * mp).sin()
                + 0.00804 * (2.0 * f).sin()
                + 0.00454 * e * (mp - m).sin()
                + 0.00204 * e2 * (2.0 * m).sin()
                - 0.00180 * (mp - 2.0 * f).sin()
                - 0.00070 * (mp + 2.0 * f).sin()
                - 0.00040 * (3.0 * mp).sin()
                - 0.00034 * e * (2.0 * mp - m).sin()
                + 0.00032 * e * (m + 2.0 * f).sin()
                + 0.00032 * e * (m - 2.0 * f).sin()
                - 0.00028 * e2 * (mp + 2.0 * m).sin()
                + 0.00027 * e * (2.0 * mp + m).sin()
                - 0.00017 * omega.sin()
                - 0.00005 * (mp - m - 2.0 * f).sin()
                + 0.00004 * (2.0 * mp + 2.0 * f).sin()
                - 0.00004 * (mp + m + 2.0 * f).sin()
                + 0.00004 * (mp - 2.0 * m).sin()
                + 0.00003 * (mp + m - 2.0 * f).sin()
                + 0.00003 * (3.0 * m).sin()
                + 0.00002 * (2.0 * mp - 2.0 * f).sin()
                + 0.00002 * (mp - m + 2.0 * f).sin()
                - 0.00002 * (3.0 * mp + m).sin()
                + w_sign * w
        }
    };
    // Planetary & barycentric arguments (Meeus Ch. 49)
    let a1 = (299.77 + 0.107408 * k - 0.009173 * t2).to_radians();
    let a2 = (251.88 + 0.016321 * k).to_radians();
    let a3 = (251.83 + 26.651886 * k).to_radians();
    let a4 = (349.42 + 36.412478 * k).to_radians();
    let a5 = (84.66 + 18.206239 * k).to_radians();
    let a6 = (141.74 + 53.303771 * k).to_radians();
    let a7 = (207.14 + 2.453732 * k).to_radians();
    let a8 = (154.84 + 7.306860 * k).to_radians();
    let a9 = (34.52 + 27.261239 * k).to_radians();
    let a10 = (207.19 + 0.121824 * k).to_radians();
    let a11 = (291.34 + 1.844379 * k).to_radians();
    let a12 = (161.72 + 24.198154 * k).to_radians();
    let a13 = (239.56 + 25.513099 * k).to_radians();
    let a14 = (331.55 + 3.592518 * k).to_radians();
    let extra = 0.000325 * a1.sin()
        + 0.000165 * a2.sin()
        + 0.000164 * a3.sin()
        + 0.000126 * a4.sin()
        + 0.000110 * a5.sin()
        + 0.000062 * a6.sin()
        + 0.000060 * a7.sin()
        + 0.000056 * a8.sin()
        + 0.000047 * a9.sin()
        + 0.000042 * a10.sin()
        + 0.000040 * a11.sin()
        + 0.000037 * a12.sin()
        + 0.000035 * a13.sin()
        + 0.000023 * a14.sin();
    jde += corr + extra;
    // Convert Dynamical Time (JDE) to Universal Time (UTC JD)
    jde - DELTA_T_DAYS
}
/// Computes topocentric lunar & solar metrics using Meeus Chapters 25, 47, and 48.
fn compute_phase(
    now: DateTime<Utc>,
    cal: &MoonCalendarData,
    lat_deg: f64,
    lon_deg: f64,
) -> MoonPhaseData {
    let jd = cal.julian_date;
    let jde = jd + DELTA_T_DAYS;
    let t = (jde - 2451545.0) / 36525.0;
    let t2 = t * t;
    let t3 = t2 * t;
    let t4 = t3 * t;
    // ── Sun (Meeus Ch. 25) ───────────────────────────────────
    let l0 = (280.46646 + 36000.76983 * t + 0.0003032 * t2).rem_euclid(360.0);
    let m_sun = (357.52911 + 35999.05029 * t - 0.0001537 * t2).to_radians();
    let e_sun = 0.016708634 - 0.000042037 * t - 0.0000001267 * t2;
    let c_sun = (1.914602 - 0.004817 * t - 0.000014 * t2) * m_sun.sin()
        + (0.019993 - 0.000101 * t) * (2.0 * m_sun).sin()
        + 0.000289 * (3.0 * m_sun).sin();
    let sun_true_lon = (l0 + c_sun).rem_euclid(360.0);
    let sun_nu = m_sun + c_sun.to_radians();
    let mut sun_r_au = (1.000001018 * (1.0 - e_sun * e_sun)) / (1.0 + e_sun * sun_nu.cos());
    // Major planetary & lunar barycentric perturbations on Earth-Sun distance (in AU)
    let a_ven = (119.75 + 131.849 * t).to_radians();
    let b_ven = (225.328 + 35999.05 * t).to_radians();
    let c_jup = (72.56 + 20.186 * t).to_radians();
    let d_moon = (297.8501921 + 445267.1114034 * t).to_radians();
    sun_r_au += 0.00000543 * a_ven.sin()
        + 0.00001575 * b_ven.sin()
        + 0.00001627 * c_jup.sin()
        + 0.00003076 * d_moon.cos();
    let sun_dist_km = sun_r_au * AU_KM;
    let sun_subtends = 2.0 * (695_700.0_f64 / sun_dist_km).asin().to_degrees();
    // ── Moon (Meeus Ch. 47) ──────────────────────────────────
    let lp = (218.3164477 + 481267.88123421 * t - 0.0015786 * t2 + t3 / 538841.0 - t4 / 65194000.0)
        .rem_euclid(360.0);
    let d = (297.8501921 + 445267.1114034 * t - 0.0018819 * t2 + t3 / 545868.0 - t4 / 113065000.0)
        .to_radians();
    let m = (357.5291092 + 35999.0502909 * t - 0.0001536 * t2 + t3 / 24490000.0).to_radians();
    let mp = (134.9633964 + 477198.8675055 * t + 0.0087414 * t2 + t3 / 69699.0 - t4 / 14712000.0)
        .to_radians();
    let f = (93.2720950 + 483202.0175233 * t - 0.0036539 * t2 - t3 / 3526000.0 + t4 / 863310000.0)
        .to_radians();
    let e = 1.0 - 0.002516 * t - 0.0000074 * t2;
    let e2 = e * e;
    // Principal periodic terms for lunar longitude (10^-6 deg) and distance (meters)
    let sum_l = 6288774.0 * mp.sin()
        + 1274027.0 * (2.0 * d - mp).sin()
        + 658314.0 * (2.0 * d).sin()
        + 213618.0 * (2.0 * mp).sin()
        - 185116.0 * e * m.sin()
        - 114332.0 * (2.0 * f).sin()
        + 58793.0 * (2.0 * d - 2.0 * mp).sin()
        + 57066.0 * e * (2.0 * d - m - mp).sin()
        + 53322.0 * (2.0 * d + mp).sin()
        + 45758.0 * e * (2.0 * d - m).sin()
        - 40923.0 * e * (m - mp).sin()
        - 34720.0 * d.sin()
        - 30383.0 * e * (m + mp).sin()
        + 15327.0 * (2.0 * d - 2.0 * f).sin()
        - 12528.0 * (mp + 2.0 * f).sin()
        + 10980.0 * (mp - 2.0 * f).sin()
        + 10675.0 * (4.0 * d - mp).sin()
        + 10034.0 * (3.0 * mp).sin()
        + 8548.0 * (4.0 * d - 2.0 * mp).sin()
        - 7888.0 * e * (2.0 * d + m - mp).sin()
        - 6766.0 * e * (2.0 * d + m).sin()
        - 5163.0 * (d - mp).sin()
        + 4987.0 * e * (d + m).sin()
        + 4036.0 * e * (2.0 * d - m + mp).sin();
    let sum_r = -20905355.0 * mp.cos()
        - 3699111.0 * (2.0 * d - mp).cos()
        - 2955968.0 * (2.0 * d).cos()
        - 569925.0 * (2.0 * mp).cos()
        + 48888.0 * e * m.cos()
        - 3149.0 * (2.0 * f).cos()
        + 246158.0 * (2.0 * d - 2.0 * mp).cos()
        - 152138.0 * e * (2.0 * d - m - mp).cos()
        - 170733.0 * (2.0 * d + mp).cos()
        - 204586.0 * e * (2.0 * d - m).cos()
        - 129620.0 * e * (m - mp).cos()
        + 108743.0 * d.cos()
        + 104755.0 * e * (m + mp).cos()
        + 10321.0 * (2.0 * d - 2.0 * f).cos()
        + 79661.0 * (mp - 2.0 * f).cos()
        - 34782.0 * (4.0 * d - mp).cos()
        - 23210.0 * (3.0 * mp).cos()
        - 21636.0 * (4.0 * d - 2.0 * mp).cos()
        + 24208.0 * e * (2.0 * d + m - mp).cos()
        + 30824.0 * e * (2.0 * d + m).cos()
        - 8379.0 * (d - mp).cos()
        - 16675.0 * e * (d + m).cos()
        - 12831.0 * e * (2.0 * d - m + mp).cos()
        - 10445.0 * (2.0 * d + 2.0 * mp).cos()
        - 11650.0 * (4.0 * d).cos()
        + 14403.0 * (2.0 * d - 3.0 * mp).cos()
        - 7003.0 * e * (m - 2.0 * mp).cos()
        + 10056.0 * e2 * (2.0 * d - 2.0 * m - mp).cos()
        + 6322.0 * (d + mp).cos()
        - 9884.0 * e2 * (2.0 * d - 2.0 * m).cos();
    let sum_b = 5128122.0 * f.sin()
        + 280602.0 * (mp + f).sin()
        + 277693.0 * (mp - f).sin()
        + 173237.0 * (2.0 * d - f).sin()
        + 55413.0 * (2.0 * d - mp + f).sin()
        + 46271.0 * (2.0 * d - mp - f).sin()
        + 32573.0 * (2.0 * d + f).sin()
        + 17198.0 * (2.0 * mp + f).sin()
        + 9266.0 * (2.0 * d + mp - f).sin()
        + 8822.0 * (2.0 * mp - f).sin()
        + 8216.0 * e * (2.0 * d - m - f).sin()
        + 4324.0 * (2.0 * d - 2.0 * mp - f).sin()
        + 4200.0 * (2.0 * d + mp + f).sin();
    let moon_lon = (lp + sum_l / 1_000_000.0).rem_euclid(360.0);
    let moon_lat = sum_b / 1_000_000.0;
    let geocentric_dist_km = 385_000.56 + sum_r / 1000.0;
    // ── Topocentric Distance Correction (Meeus Ch. 40) ───────
    let obl = (23.439291 - 0.0130042 * t).to_radians();
    let lam = moon_lon.to_radians();
    let beta = moon_lat.to_radians();
    let ra = (lam.sin() * obl.cos() - beta.tan() * obl.sin()).atan2(lam.cos());
    let dec = (beta.sin() * obl.cos() + beta.cos() * obl.sin() * lam.sin()).asin();
    let d_ut = jd - 2451545.0;
    let gmst_deg = (280.46061837 + 360.98564736629 * d_ut + 0.000387933 * t2 - t3 / 38_710_000.0)
        .rem_euclid(360.0);
    let ha = (gmst_deg + lon_deg).to_radians() - ra;
    let lat = lat_deg.to_radians();
    let sin_alt = lat.sin() * dec.sin() + lat.cos() * dec.cos() * ha.cos();
    let sin_pi = EARTH_RADIUS_KM / geocentric_dist_km;
    // Law of cosines on Earth-Observer-Moon triangle
    let topocentric_dist_km =
        geocentric_dist_km * (1.0 - 2.0 * sin_pi * sin_alt + sin_pi * sin_pi).sqrt();
    let parallax_deg = sin_pi.asin().to_degrees();
    let _subtends_deg = 2.0 * (1737.4_f64 / topocentric_dist_km).asin().to_degrees();
    // ── Illumination & Phase Angle (Meeus Ch. 48) ────────────
    let sun_lon_rad = sun_true_lon.to_radians();
    let cos_psi = beta.cos() * (lam - sun_lon_rad).cos();
    let psi = cos_psi.clamp(-1.0, 1.0).acos();
    let phase_angle = (sun_dist_km * psi.sin()).atan2(geocentric_dist_km - sun_dist_km * cos_psi);
    let fraction_illuminated = (1.0 + phase_angle.cos()) * 0.5;
    let age = (jd - cal.last_new_moon).max(0.0);
    let lunation_len = (cal.next_new_moon - cal.last_new_moon).max(1.0);
    let fraction_of_lunation = (age / lunation_len).clamp(0.0, 1.0);
    let phase = classify_phase(fraction_of_lunation, fraction_illuminated);


    MoonPhaseData {
        julian_date: jd,
        timestamp: now.timestamp(),
        utc_datetime: now.to_rfc3339(),
        age,
        fraction_of_lunation,
        phase,
        fraction_illuminated,
        distance_to_earth_km: geocentric_dist_km,
        subtends: 2.0 * (1737.4_f64 / geocentric_dist_km).asin().to_degrees(),
        ecliptic_longitude: moon_lon,
        ecliptic_latitude: moon_lat,
        parallax: parallax_deg,
        distance_to_earth_earth_radii: geocentric_dist_km / EARTH_RADIUS_KM,
        sun_ecliptic_longitude: sun_true_lon,
        sun_distance_to_earth_km: sun_dist_km,
        sun_distance_to_earth_astronomical_units: sun_r_au,
        sun_subtends,
    }
}
fn classify_phase(frac: f64, illum: f64) -> PhaseInfo {
    // Primary phases only trigger right around the exact quarter/new/full peaks
    let (idx, name, icon) = if illum <= 0.015 || frac < 0.015 || frac > 0.985 {
        (0, "New Moon", "🌑")
    } else if (frac - 0.25).abs() <= 0.015 {
        (2, "First Quarter", "🌓")
    } else if illum >= 0.985 || (frac - 0.50).abs() <= 0.015 {
        (4, "Full Moon", "🌕")
    } else if (frac - 0.75).abs() <= 0.015 {
        (6, "Last Quarter", "🌗")
    } else if frac < 0.25 {
        (1, "Waxing Crescent", "🌒")
    } else if frac < 0.50 {
        (3, "Waxing Gibbous", "🌔")
    } else if frac < 0.75 {
        (5, "Waning Gibbous", "🌖")
    } else {
        (7, "Waning Crescent", "🌘")
    };

    PhaseInfo {
        index: idx,
        name: name.to_string(),
        icon: icon.to_string(),
    }
}

