mod moon_data;
use moon_data::{MoonInfo, format_local};   

use std::time::Duration;
use cosmic::app::Core;
use cosmic::iced::{
    platform_specific::shell::commands::popup::{get_popup},
    window::Id,
    Limits,
};
use cosmic::{Action, Element, Task};

const APP_ID: &str = "com.example.CosmicAppletMoon";

#[derive(Debug, Clone)]
pub enum Msg {
    Tick,
    TogglePopup,
    PopupClosed(Id),
}

macro_rules! moon_icons {
    ($day:expr, $($n:expr => $file:expr),*) => {
        match $day {
            $($n => cosmic::iced::widget::image::Handle::from_bytes(
                include_bytes!($file).to_vec()
            ),)*
            _ => cosmic::iced::widget::image::Handle::from_bytes(
                include_bytes!("../assets/icons/moon-1-NM.png").to_vec()
            ),
        }
    };
}

fn field_row<'a>(label: &'a str, value: String) -> cosmic::Element<'a, Msg> {
    cosmic::widget::row![
        cosmic::widget::text(label).width(cosmic::iced::Length::Fixed(113.0)).size(10.0),
        cosmic::widget::text(value).size(10.0),
    ]
    .spacing(8)
    .into()
}    

fn get_moon_phase_image(day: usize) -> cosmic::iced::widget::image::Handle {
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

pub struct MoonApplet {
    core: Core,
    popup: Option<Id>,
    info: MoonInfo,
    icon: cosmic::iced::widget::image::Handle,
}

impl cosmic::Application for MoonApplet {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Msg;

    const APP_ID: &'static str = APP_ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Action<Msg>>) {
        let info = MoonInfo::compute();
        let icon = get_moon_phase_image(info.day);
        (
            Self {
                core,
                popup: None,
                info,
                icon,
            },
            Task::none(),
        )
    }

    fn update(&mut self, msg: Self::Message) -> Task<Action<Msg>> {
        match msg {
            Msg::Tick => {
                self.info = MoonInfo::compute();
                self.icon = get_moon_phase_image(self.info.day);
            }

            Msg::TogglePopup => {
                let new_id = Id::unique();
                self.popup.replace(new_id);
                let mut popup_settings = self.core.applet.get_popup_settings(
                    self.core.main_window_id().unwrap(),
                    new_id,
                    Some((420, 400)),
                    None,
                    None,
                );
                popup_settings.positioner.size_limits = Limits::NONE
                    .min_width(200.0)
                    .min_height(80.0)
                    .max_width(600.0)
                    .max_height(500.0);
                return get_popup(popup_settings);
            }
            Msg::PopupClosed(id) => {
                if self.popup == Some(id) {
                    self.popup = None;
                }
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let row: cosmic::Element<'_, Msg> = cosmic::iced::widget::row![
            cosmic::widget::image(self.icon.clone())
                .width(24)
                .height(24),
            // cosmic::iced::widget::text(&self.label).size(13.0),
        ]
        .spacing(6)
        .align_y(cosmic::iced::Alignment::Center)
        .into();

        self.core
            .applet
            .autosize_window(
                cosmic::widget::button::custom(row)
                    .class(cosmic::theme::Button::AppletIcon)
                    .on_press(Msg::TogglePopup),
            )
            .into()
    }

    fn view_window(&self, id: Id) -> Element<'_, Self::Message> {

        let p = &self.info.phase;
        let c = &self.info.calendar;

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

        if Some(id) == self.popup {
            let content = cosmic::iced::widget::column![
                cosmic::iced::widget::container(
                    cosmic::iced::widget::image(self.icon.clone()).width(48).height(48)
                )
                .width(cosmic::iced::Length::Fill)
                .align_x(cosmic::iced::Alignment::Center),    
                cosmic::iced::widget::rule::horizontal(2),                
                field_row("Moon Phase:", self.info.phase.phase.name.clone()),
                field_row("Illumination:", format!("{:.3}%", self.info.illumination)),
                field_row("Moon Age:", age_str),
                field_row(self.info.next_state, self.info.time_remaining.clone()),   
                cosmic::iced::widget::rule::horizontal(2),
                field_row("Next Full Moon:", format_local(&c.full_moon_utc)),
                field_row("Next New Moon:", format_local(&c.next_new_moon_utc)),
                field_row("Next First Quarter:", format_local(&c.first_quarter_utc)),
                field_row("Next Last Quarter:", format_local(&c.last_quarter_utc)),
                field_row("Local DateTime:", format_local(&c.utc_datetime)),  
                cosmic::iced::widget::rule::horizontal(2),
                field_row("Earth-Moon Distance:", format!("{:.0} km, {:.1} Earth Radii", p.distance_to_earth_km, p.distance_to_earth_earth_radii)),
                field_row("Earth-Sun Distance:", format!("{:.0} km, {:.2} au", p.sun_distance_to_earth_km, p.sun_distance_to_earth_astronomical_units)),

                // Calendar fields
             
            ]
            .spacing(2)
            .padding(8);

            self.core
                .applet
                .popup_container(content)
                .max_width(580.0)
                .into()
        } else {
            cosmic::iced::widget::text("").into()
        }
    }

    fn subscription(&self) -> cosmic::iced::Subscription<Self::Message> {
        cosmic::iced::time::every(Duration::from_secs(60)).map(|_| Msg::Tick)
    }
}

fn main() -> cosmic::iced::Result {
    cosmic::applet::run::<MoonApplet>(())
}
#[cfg(test)]
mod tests {
    // use moontool::moon::{ForDateTime, ToJSON};

    // use super::moon_data::MoonInfo;

    #[test]
    #[allow(dead_code)]
    fn test_output() {
        // let phase = MoonPhase::now();
        // let json = phase.to_json();
        // println!("\n=== PHASE JSON {} ===", json);

        // let calendar: MoonCalendar = ForDateTime::now();
        // let cal_json = calendar.to_json();
        // println!("\n=== CAL JSON {} ===", cal_json);
        
        // let info = MoonInfo::compute();

        // println!("\n======== APPLET TEST OUTPUT =======");
        // println!("\n===Header:==========================\n");
        // println!("{}", info.label());
        // println!("\n===MoonDay:========================\n");
        // println!("{}", info.day);
        // println!("\n===PhaseToString:==========================\n");
        // println!("{}", info.phase_to_string());
        // println!("\n===PhaseValue:==========================\n");
        // println!("{}", info.phase.phase.name);
        // println!("\n===CalendarToString:=======================\n");
        // println!("{}", info.calendar_to_string());
        // println!("\n=== APPLET TEST OUTPUT END ===\n");
    }


}
