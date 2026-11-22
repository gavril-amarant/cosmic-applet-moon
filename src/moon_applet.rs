use cosmic::app::Core;
use cosmic::iced::Limits;
use cosmic::iced::window::Id;
use cosmic::{Action, Element, Task};
use crate::moon_data::MoonInfo;
use crate::view::{get_moon_phase_image, view_panel, view_popup};
use cosmic::surface::action::{app_popup, destroy_popup};

const APP_ID: &str = "io.github.gavril_amarant.CosmicAppletMoon";

#[derive(Debug, Clone)]
pub enum Msg {
    Tick,
    TogglePopup,
    // Surface(cosmic::surface::Action<Msg>),
    PopupClosed(Id),
    ToggleCityPicker,
    CitySearchChanged(String),
    SelectCity(Option<String>),
    ToggleNotifications(bool),
    SelectNotifyDays(usize),
    SelectNotifyHours(usize),
}

pub struct MoonApplet {
    pub core: Core,
    pub popup: Option<Id>,
    pub info: MoonInfo,
    pub icon: cosmic::iced::widget::image::Handle,
    pub config_handler: Option<cosmic::cosmic_config::Config>,
    pub config: crate::config::MoonConfig,
    pub city_picker_open: bool,
    pub city_search: String,
    pub all_cities: Vec<crate::moon_horizon::CityEntry>,    
}




impl cosmic::Application for MoonApplet {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Msg;

    const APP_ID: &'static str = APP_ID;

    fn core(&self) -> &Core { &self.core }
    fn core_mut(&mut self) -> &mut Core { &mut self.core }

    fn init(core: Core, _flags: ()) -> (Self, Task<Action<Msg>>) {
        let (config_handler, config) = crate::config::MoonConfig::load(APP_ID);
        let info = MoonInfo::compute(config.manual_tz.as_deref());
        let icon = get_moon_phase_image(info.day);
        let all_cities = crate::moon_horizon::all_cities();
        

        let mut app = Self {
            core,
            popup: None,
            info,
            icon,
            config_handler,
            config,
            city_picker_open: false,
            city_search: String::new(),
            all_cities,
        };
        let notify_task = app.check_moon_notifications();

        (app, notify_task)
    }



    fn view(&self) -> Element<'_, Msg> {
        self.core.applet.autosize_window(view_panel(&self.icon)).into()
    }

    fn view_window(&self, id: Id) -> Element<'_, Msg> {
        if Some(id) == self.popup {
            self.core.applet
                .popup_container(view_popup(
                    &self.info,
                    &self.icon,
                    self.city_picker_open,
                    &self.city_search,
                    &self.all_cities,
                    self.config.manual_tz.is_some(),
                    self.config.notifications_enabled,
                    self.config.notify_days,
                    self.config.notify_hours,
                ))
                .max_width(580.0)
                .into()
        } else {
            cosmic::widget::text("").into()
        }
    }



    fn update(&mut self, msg: Msg) -> Task<Action<Msg>> {
        match msg {
            Msg::SelectNotifyDays(days_idx) => {
                let days = (days_idx as u32).min(14);
                self.config.notify_days = days;
                self.config.last_notified_event = None;
                if let Some(h) = self.config_handler.as_ref() {
                    let _ = self.config.set_notify_days(h, days);
                    let _ = self.config.set_last_notified_event(h, None);
                }
                return self.check_moon_notifications();
            }
            Msg::SelectNotifyHours(hours_idx) => {
                let hours = (hours_idx as u32).min(24);
                self.config.notify_hours = hours;
                self.config.last_notified_event = None;
                if let Some(h) = self.config_handler.as_ref() {
                    let _ = self.config.set_notify_hours(h, hours);
                    let _ = self.config.set_last_notified_event(h, None);
                }
                return self.check_moon_notifications();
            }            
            Msg::Tick => {
                self.info = MoonInfo::compute(self.config.manual_tz.as_deref());
                self.icon = get_moon_phase_image(self.info.day);
                return self.check_moon_notifications();
            }
            Msg::ToggleCityPicker => {
                self.city_picker_open = !self.city_picker_open;
                self.city_search.clear();
            }
            Msg::CitySearchChanged(query) => {
                self.city_search = query;
            }
            Msg::SelectCity(tz_opt) => {
                self.config.manual_tz = tz_opt;
                self.config.save(self.config_handler.as_ref());
                self.city_picker_open = false;
                self.city_search.clear();
                self.info = MoonInfo::compute(self.config.manual_tz.as_deref());
                self.icon = get_moon_phase_image(self.info.day);
            }      
            Msg::ToggleNotifications(enabled) => {
                self.config.notifications_enabled = enabled;
                if let Some(h) = self.config_handler.as_ref() {
                    let _ = self.config.set_notifications_enabled(h, enabled);
                }
                if enabled { return self.check_moon_notifications(); }
            }

            Msg::TogglePopup => {

                if let Some(p) = self.popup.take() {
                    return cosmic::surface::surface_task(destroy_popup(p));
                }

                return cosmic::surface::surface_task(app_popup(

                    |_| Default::default(),
                    |app: &mut Self| {
                        let new_id = Id::unique();
                        app.popup = Some(new_id);

                        let mut popup_settings = app.core.applet.get_popup_settings(
                            app.core.main_window_id().unwrap(),
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

                        popup_settings
                    },
                    None,
                ));
            }
            // Msg::Surface(action) => {
            //     return cosmic::surface::surface_task(action);
            // }   
            Msg::PopupClosed(id) => {
                if self.popup == Some(id) {
                    self.popup = None;
                }
            }
        }
        Task::none()
    }

    fn subscription(&self) -> cosmic::iced::Subscription<Msg> {
        cosmic::iced::time::every(std::time::Duration::from_secs(60)).map(|_| Msg::Tick)
    }   

    fn on_close_requested(&self, id: Id) -> Option<Msg> {
        Some(Msg::PopupClosed(id))
    }
    
}   

impl MoonApplet {
    fn check_moon_notifications(&mut self) -> Task<Action<Msg>> {
        let total_lead_hours = (self.config.notify_days * 24 + self.config.notify_hours) as f64;
        if !self.config.notifications_enabled || total_lead_hours <= 0.0 {
            return Task::none();
        }

        let now_jd = self.info.calendar.julian_date;
        let c = &self.info.calendar;

        let candidates = [
            ("Full Moon", c.full_moon, &c.full_moon_utc),
            ("New Moon", c.next_new_moon, &c.next_new_moon_utc),
        ];

        for (label, target_jd, utc_str) in candidates {
            let hours_until = (target_jd - now_jd) * 24.0;

            if hours_until > 0.0 && hours_until <= total_lead_hours {
                let event_key = format!(
                    "{}:{}:{}:{}",
                    label, utc_str, self.config.notify_days, self.config.notify_hours
                );

                if self.config.last_notified_event.as_deref() != Some(&event_key) {
                    let local_time = crate::moon_data::format_in_tz(utc_str, &self.info.tz_name);
                    let summary = format!("Upcoming {}", label);
                    let body = format!(
                        "{} occurs in {} ({}).",
                        label,
                        crate::moon_data::format_age(target_jd - now_jd),
                        local_time
                    );

                    self.config.set_notified(self.config_handler.as_ref(), event_key);

                    return Task::future(async move {
                        send_desktop_notification(summary, body).await;
                        Action::None
                    });
                }
            }
        }

        Task::none()
    }


}

async fn send_desktop_notification(summary: String, body: String) {
    let Ok(conn) = zbus::Connection::session().await else {
        return;
    };

    let empty_actions: Vec<&str> = Vec::new();
    let empty_hints: std::collections::HashMap<&str, zbus::Variant<'_>> =
        std::collections::HashMap::new();

    let _ = conn
        .call_method(
            Some("org.freedesktop.Notifications"),
            "/org/freedesktop/Notifications",
            Some("org.freedesktop.Notifications"),
            "Notify",
            &(
                "Cosmic Moon Applet",
                0u32,
                "weather-clear-night",
                summary.as_str(),
                body.as_str(),
                empty_actions,
                empty_hints,
                -1i32,
            ),
        )
        .await;
}
