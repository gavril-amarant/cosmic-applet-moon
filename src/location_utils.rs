
const ZONE_TAB: &str = include_str!("../assets/zone1970.tab");
const ISO3166_TAB: &str = include_str!("../assets/iso3166.tab");



#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CityEntry {
    pub tz_name: String,
    pub city: String,
    pub country: String,
    pub display_label: String, // e.g. "Bucharest, Romania"
}

pub fn all_cities() -> Vec<CityEntry> {
    let mut list: Vec<CityEntry> = ZONE_TAB
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let mut cols = line.split('\t');
            let countries = cols.next()?;
            let _coords = cols.next()?;
            let tz = cols.next()?;

            let primary_cc = countries.split(',').next().unwrap_or("");
            let country = country_name(primary_cc).to_string();
            let city = tz.split('/').last().unwrap_or(tz).replace('_', " ");
            let display_label = format!("{}, {}", city, country);

            Some(CityEntry {
                tz_name: tz.to_string(),
                city,
                country,
                display_label,
            })
        })
        .collect();

    list.sort_by(|a, b| a.display_label.cmp(&b.display_label));
    list
}

pub fn country_name(code: &str) -> &str {
    for line in ISO3166_TAB.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        // Format: CC \t Country Name
        if let Some((cc, name)) = line.split_once('\t') {
            if cc == code {
                return name;
            }
        }
    }
    code
}
#[allow(dead_code)]
pub fn resolve_timezone(manual_tz: Option<&str>) -> String {
    manual_tz
        .map(str::to_string)
        .or_else(|| iana_time_zone::get_timezone().ok())
        .unwrap_or_else(|| "UTC".to_string())
}


pub fn resolve_tz_name(manual_tz: Option<&str>) -> String {
    manual_tz
        .map(str::to_string)
        .or_else(|| iana_time_zone::get_timezone().ok())
        .unwrap_or_else(|| "UTC".to_string())
}

pub fn get_observer_coords(manual_tz: Option<&str>) -> (f64, f64) {
    let tz_name = resolve_tz_name(manual_tz);
    lookup_zone_coords(&tz_name).unwrap_or((0.0, 0.0))
}

pub fn lookup_country_for_tz(target_tz: &str) -> String {
    for line in ZONE_TAB.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut cols = line.split('\t');
        if let (Some(countries), Some(_coords), Some(tz)) = (cols.next(), cols.next(), cols.next()) {
            if tz == target_tz {
                let cc = countries.split(',').next().unwrap_or("");
                return country_name(cc).to_string();
            }
        }
    }
    "Unknown".to_string()
}

pub fn lookup_zone_coords(target_tz: &str) -> Option<(f64, f64)> {
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

pub fn parse_iso6709(s: &str) -> Option<(f64, f64)> {
    let split_idx = s[1..].find(['+', '-'])? + 1;
    Some((parse_dms(&s[..split_idx], 2)?, parse_dms(&s[split_idx..], 3)?))
}


pub fn parse_dms(part: &str, deg_digits: usize) -> Option<f64> {
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