use cosmic::iced::widget::image::Handle;
use crate::astronomy::{MoonInfo};
use crate::utils::{APP_HOMEPAGE, APP_VERSION, format_age, format_in_tz};
use crate::moon_applet::Msg;
use crate::location_utils::CityEntry;



// ── Reusable widgets ─────────────────────────────────────────

fn field_row<'a>(label: &'a str, value: String, width: f32) -> cosmic::Element<'a, Msg> {
    cosmic::widget::row![
        cosmic::widget::text(label)
            .width(cosmic::iced::Length::Fixed(width))
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
        ("Next Half Moon (first):", &c.first_quarter_utc),
        ("Next Half Moon (last):", &c.last_quarter_utc),
    ];

    events.retain(|(_, s)| *s > now_str.as_str());
    events.sort_by(|a, b| a.1.cmp(b.1));

    events
        .iter()
        .map(|(label, dt)| field_row(*label, format_in_tz(dt, &info.tz_name), 113.0).into())
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
    let hour_options: Vec<String> = (0..=23)
        .map(|h| if h == 1 { "1 hr".to_string() } else { format!("{} hrs", h) })
        .collect();

    let growing_waxing_str = if info.growing {
        "(increasing)"
    } else {
        "(decreasing)"
    };

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
                    .size(10.0),   
                cosmic::iced::widget::rule::horizontal(2),
                cosmic::widget::text(" ")
                    .width(cosmic::iced::Length::Fill)
                    .align_x(cosmic::iced::Alignment::Center)
                    .size(10.0),                  
                field_row("Illumination:", format!("{:.1}% {}", info.illumination, growing_waxing_str), 70.0),
                cosmic::widget::tooltip(
                    field_row("Moon Age:", format_age(p.age), 70.0),
                    cosmic::widget::text("Time passed since the last New Moon").size(10.0),
                    cosmic::widget::tooltip::Position::Top,
                ),                
                cosmic::widget::tooltip(
                    field_row(info.next_state, info.time_remaining.clone(), 70.0),
                    cosmic::widget::text(info.next_state_tooltip).size(10.0),
                    cosmic::widget::tooltip::Position::Top,
                )
                // .style(|_theme| cosmic::widget::popover::Style {
                //     background: Some(cosmic::iced::Background::Color(
                //         cosmic::iced::Color::from_rgb(0.15, 0.15, 0.15),
                //     )),
                //     text_color: Some(cosmic::iced::Color::WHITE),
                //     ..Default::default()
                // })
                
                // .style(|theme| {
                //     let cosmic = theme.cosmic();
                //     cosmic::widget::popover::Style {
                //         background: Some(cosmic::iced::Background::Color(
                //             cosmic::iced::Color::from(cosmic.background(true).base),
                //         )),
                //         text_color: Some(cosmic::iced::Color::from(cosmic.background(true).on)),
                //         ..Default::default()
                //     }
                // }),
            ]
        ]
        .spacing(8)
        .align_y(cosmic::iced::Alignment::Center),
        cosmic::iced::widget::rule::horizontal(2),
        cosmic::iced::widget::row![
            cosmic::iced::widget::column![
                field_row("Moonrise:",format!("{}", mh.today_moonrise), 113.0),
                field_row("Moonset:",format!("{}", mh.today_moonset), 113.0),  
                cosmic::iced::widget::rule::horizontal(1),  
                field_row("Moonrise Tomorrow:",format!("{}", mh.tomorrow_moonrise), 113.0),
                field_row("Moonset Tomorrow:",format!("{}", mh.tomorrow_moonset), 113.0), 
            ].width(150),
            cosmic::widget::divider::vertical::default(),
            cosmic::iced::widget::column![
                field_row("Sunrise:",format!("{}", mh.today_sunrise), 113.0),
                field_row("Sunset:",format!("{}", mh.today_sunset), 113.0),    
                cosmic::iced::widget::rule::horizontal(1),
                field_row("Sunrise Tomorrow:",format!("{}", mh.tomorrow_sunrise), 113.0),
                field_row("Sunset Tomorrow:",format!("{}", mh.tomorrow_sunset), 113.0), 
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
            ), 113.0
        ),
        field_row(
            "Earth-Sun Distance:",
            format!(
                "{:.1} Suns | {:.1} Earths | {:.0} km",
                &info.suns_between_sun, 
                &info.earths_between_sun, p.sun_distance_to_earth_km 
            ), 113.0
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
        field_row("Local DateTime:", format_in_tz(&c.utc_datetime, &info.tz_name), 113.0),
        cosmic::iced::widget::rule::horizontal(2),
        cosmic::widget::row![
            cosmic::widget::checkbox(notifications_enabled)
                .on_toggle(Msg::ToggleNotifications).size(10),
            cosmic::widget::text(" Remind me before New/Full Moon with:")
                .size(10.0),
            cosmic::widget::dropdown(
                day_options,
                Some((notify_days as usize).min(14)),
                Msg::SelectNotifyDays,
            ).text_size(10.0),
            cosmic::widget::dropdown(
                hour_options,
                Some((notify_hours as usize).min(23)),
                Msg::SelectNotifyHours,
            ).text_size(10.0),
        ].align_y(cosmic::iced::Alignment::Center),
        cosmic::iced::widget::rule::horizontal(2),        
        cosmic::widget::row![
            cosmic::iced::widget::Space::new()
                .width(cosmic::iced::Length::Fill),
            cosmic::widget::divider::vertical::default(),
            cosmic::widget::button::link("homepage")
                .on_press(Msg::OpenUrl(APP_HOMEPAGE.to_string()))
                .padding(0)
                .font_size(8),
            cosmic::widget::divider::vertical::default(),
            cosmic::widget::text(APP_VERSION)
                .size(8.0),
        ]
        .spacing(8)
        .align_y(cosmic::iced::Alignment::Center)
    ]
    .spacing(2)
    .padding(8);

    content.into()
}
