use cosmic::iced::widget::image::Handle;
use crate::moon_data::{format_age, format_in_tz, MoonInfo};
use crate::moon_applet::Msg;
use crate::moon_horizon::CityEntry;

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

// ── Reusable widgets ─────────────────────────────────────────

fn field_row<'a>(label: &'a str, value: String) -> cosmic::Element<'a, Msg> {
    cosmic::widget::row![
        cosmic::widget::text(label)
            .width(cosmic::iced::Length::Fixed(113.0))
            .size(10.0),
        cosmic::widget::text(value).size(10.0),
    ]
    .spacing(8)
    .into()
}

fn field_row_short<'a>(label: &'a str, value: String) -> cosmic::Element<'a, Msg> {
    cosmic::widget::row![
        cosmic::widget::text(label)
            .width(cosmic::iced::Length::Fixed(70.0))
            .size(10.0),
        cosmic::widget::text(value).size(10.0),
    ]
    .spacing(8)
    .into()
}

// ── Panel (icon in the bar) ──────────────────────────────────

pub fn view_panel(icon: &Handle) -> cosmic::Element<'_, Msg> {
    let row: cosmic::Element<'_, Msg> =
        cosmic::widget::row![cosmic::widget::image(icon.clone()).width(24).height(24),]
            .spacing(6)
            .align_y(cosmic::iced::Alignment::Center)
            .into();

    cosmic::widget::button::custom(row)
        .class(cosmic::theme::Button::AppletIcon)
        .on_press(Msg::TogglePopup)
        .into()
}

// ── Popup ────────────────────────────────────────────────────

fn build_event_rows(info: &MoonInfo) -> Vec<cosmic::Element<'_, Msg>> {
    let now_str = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let c = &info.calendar;

    let mut events: Vec<(&str, &str)> = vec![
        ("Next New Moon:", &c.next_new_moon_utc),
        ("Next Full Moon:", &c.full_moon_utc),
        ("Next First Quarter:", &c.first_quarter_utc),
        ("Next Last Quarter:", &c.last_quarter_utc),
    ];

    events.retain(|(_, s)| *s > now_str.as_str());
    events.sort_by(|a, b| a.1.cmp(b.1));

    events
        .iter()
        .map(|(label, dt)| field_row(*label, format_in_tz(dt, &info.tz_name)).into())
        .collect()
}

fn build_city_picker<'a>(
    open: bool,
    search: &'a str,
    all_cities: &'a [CityEntry],
) -> cosmic::Element<'a, Msg> {
    if !open {
        return cosmic::iced::widget::Space::new().into();
    }

    let query = search.trim().to_lowercase();
    let mut rows: Vec<cosmic::Element<'a, Msg>> = vec![
        cosmic::widget::button::text("Auto (OS Timezone)")
            .on_press(Msg::SelectCity(None))
            .width(cosmic::iced::Length::Fill)
            .into(),
    ];

    for entry in all_cities
        .iter()
        .filter(|c| query.is_empty() || c.display_label.to_lowercase().contains(&query))
        .take(30)
    {
        rows.push(
            cosmic::widget::button::text(&entry.display_label)
                .on_press(Msg::SelectCity(Some(entry.tz_name.clone())))
                .width(cosmic::iced::Length::Fill)
                .into(),
        );
    }

    cosmic::widget::column![
        cosmic::widget::text_input("Search city or country...", search)
            .on_input(Msg::CitySearchChanged),
        cosmic::widget::scrollable(cosmic::widget::column(rows).spacing(2))
            .height(cosmic::iced::Length::Fixed(140.0)),
    ]
    .spacing(4)
    .padding([4, 0])
    .into()
}

pub fn view_popup<'a>(
    info: &'a MoonInfo,
    icon: &'a Handle,
    city_picker_open: bool,
    city_search: &'a str,
    all_cities: &'a [CityEntry],
    is_manual_city: bool,
    notifications_enabled: bool,
    notify_days: u32,
    notify_hours: u32,
) -> cosmic::Element<'a, Msg> {

    let p = &info.phase;
    let c = &info.calendar;
    let event_rows = build_event_rows(info);
    let mh = &info.moon_horizon;

    let day_options: Vec<String> = (0..=14)
        .map(|d| if d == 1 { "1 day".to_string() } else { format!("{} days", d) })
        .collect();
    let hour_options: Vec<String> = (0..=24)
        .map(|h| if h == 1 { "1 hr".to_string() } else { format!("{} hrs", h) })
        .collect();


    let content = cosmic::iced::widget::column![
        cosmic::iced::widget::row![
            cosmic::widget::image(icon.clone()).width(150).height(150),
            cosmic::widget::divider::vertical::default(),
            cosmic::iced::widget::column![
                cosmic::widget::text(info.phase.phase.name.clone())
                    .width(cosmic::iced::Length::Fill)
                    .align_x(cosmic::iced::Alignment::Center)
                    .size(20.0),
                cosmic::widget::text(" ")
                    .width(cosmic::iced::Length::Fill)
                    .align_x(cosmic::iced::Alignment::Center)
                    .size(20.0),                    
                cosmic::iced::widget::rule::horizontal(2),
                cosmic::widget::text(" ")
                    .width(cosmic::iced::Length::Fill)
                    .align_x(cosmic::iced::Alignment::Center)
                    .size(10.0),                  
                field_row_short("Illumination:", format!("{:.1}%", info.illumination)),
                field_row_short("Moon Age:", format_age(p.age)),
                field_row_short(info.next_state, info.time_remaining.clone()),
            ]
        ]
        .spacing(8)
        .align_y(cosmic::iced::Alignment::Center),
        cosmic::iced::widget::rule::horizontal(2),

        cosmic::iced::widget::row![
            cosmic::iced::widget::column![
                field_row("Moonrise Today:",format!("{}", mh.today_moonrise)),
                field_row("Moonset Today:",format!("{}", mh.today_moonset)),    
                field_row("Moonrise Tomorrow:",format!("{}", mh.tomorrow_moonrise)),
                field_row("Moonset Tomorrow:",format!("{}", mh.tomorrow_moonset)), 
            ].width(150),
            cosmic::widget::divider::vertical::default(),
            cosmic::iced::widget::column![
                field_row("Sunrise Today:",format!("{}", mh.today_sunrise)),
                field_row("Sunset Today:",format!("{}", mh.today_sunset)),    
                field_row("Sunrise Tomorrow:",format!("{}", mh.tomorrow_sunrise)),
                field_row("Sunset Tomorrow:",format!("{}", mh.tomorrow_sunset)), 
            ],
        ]        
        .spacing(8)
        .align_y(cosmic::iced::Alignment::Center),
        cosmic::iced::widget::rule::horizontal(2),
        cosmic::iced::widget::column(event_rows).spacing(2),
        cosmic::iced::widget::rule::horizontal(2),
        field_row(
            "Earth-Moon Distance:",
            format!(
                "{:.1} Moons | {:.1} Earths | {:.0} km",
                &info.moons_between_moon, 
                &info.earths_between_moon,
                p.distance_to_earth_km,
            )
        ),
        field_row(
            "Earth-Sun Distance:",
            format!(
                "{:.1} Suns | {:.1} Earths | {:.0} km",
                &info.suns_between_sun, 
                &info.earths_between_sun, p.sun_distance_to_earth_km 
            )
        ),
        cosmic::iced::widget::rule::horizontal(2),
        cosmic::widget::row![
            cosmic::widget::text("Your Location:")
                .width(cosmic::iced::Length::Fixed(113.0))
                .size(10.0),
            cosmic::widget::text(if is_manual_city {
                mh.location_label.clone()
            } else {
                format!("{} (Auto)", mh.location_label)
            })
            // .width(cosmic::iced::Length::Fill)
            .size(10.0),
            cosmic::widget::button::custom(
                cosmic::widget::text(if city_picker_open { "(Close Location)" } else { "(Change Location)" })
                    .size(10.0)
                    .font(cosmic::font::bold())
            )
            .class(cosmic::theme::Button::Link)
            .padding(0)
            .on_press(Msg::ToggleCityPicker),
        ]
        .spacing(8)
        .align_y(cosmic::iced::Alignment::Center),
        build_city_picker(city_picker_open, city_search, all_cities),
        field_row("Local DateTime:", format_in_tz(&c.utc_datetime, &info.tz_name)),
        cosmic::iced::widget::rule::horizontal(2),
        cosmic::widget::row![
            cosmic::widget::checkbox(notifications_enabled)
                .on_toggle(Msg::ToggleNotifications),
            cosmic::widget::text("Notify me of New/Full Moon before:")
                .size(10.0),
            cosmic::widget::dropdown(
                day_options,
                Some((notify_days as usize).min(14)),
                Msg::SelectNotifyDays,
            ),
            cosmic::widget::dropdown(
                hour_options,
                Some((notify_hours as usize).min(24)),
                Msg::SelectNotifyHours,
            ),
        ]
        .spacing(6)
        .align_y(cosmic::iced::Alignment::Center),
    ]
    .spacing(2)
    .padding(8);

    content.into()
}
