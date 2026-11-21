use cosmic::iced::widget::image::Handle;
use crate::moon_data::{format_age, format_local, MoonInfo};
use crate::moon_applet::Msg;

// ── Icon loading ─────────────────────────────────────────────

macro_rules! moon_icons {
    ($day:expr, $($n:expr => $file:expr),*) => {
        match $day {
            $($n => Handle::from_bytes(include_bytes!($file).to_vec()),)*
            _ => Handle::from_bytes(include_bytes!("../assets/icons/moon-1-NM.png").to_vec()),
        }
    };
}

pub fn get_moon_phase_image(day: usize) -> Handle {
    moon_icons!(day,
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
        .map(|(label, dt)| field_row(*label, format_local(dt)).into())
        .collect()
}

pub fn view_popup<'a>(info: &'a MoonInfo, icon: &'a Handle) -> cosmic::Element<'a, Msg> {
    let p = &info.phase;
    let c = &info.calendar;
    let event_rows = build_event_rows(info);

    let content = cosmic::iced::widget::column![
        cosmic::iced::widget::row![
            cosmic::widget::image(icon.clone()).width(48).height(48),
            cosmic::iced::widget::column![
                field_row_short("Moon Phase:", info.phase.phase.name.clone()),
                field_row_short("Illumination:", format!("{:.1}%", info.illumination)),
                field_row_short("Moon Age:", format_age(p.age)),
                field_row_short(info.next_state, info.time_remaining.clone()),
            ]
            .spacing(2),
        ]
        .spacing(8)
        .align_y(cosmic::iced::Alignment::Center),
        cosmic::iced::widget::rule::horizontal(2),
        cosmic::iced::widget::column(event_rows).spacing(2),
        cosmic::iced::widget::rule::horizontal(2),
        field_row(
            "Earth-Moon Distance:",
            format!(
                "{:.0} km, {:.1} Earth Radii",
                p.distance_to_earth_km, p.distance_to_earth_earth_radii
            )
        ),
        field_row(
            "Earth-Sun Distance:",
            format!(
                "{:.0} km, {:.2} au",
                p.sun_distance_to_earth_km, p.sun_distance_to_earth_astronomical_units
            )
        ),
        cosmic::iced::widget::rule::horizontal(2),
        field_row("Local DateTime:", format_local(&c.utc_datetime)),
    ]
    .spacing(2)
    .padding(8);

    content.into()
}
