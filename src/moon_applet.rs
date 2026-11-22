use cosmic::app::Core;
use cosmic::iced::Limits;
use cosmic::iced::window::Id;
use cosmic::{Action, Element, Task};
use crate::moon_data::MoonInfo;
use crate::view::{get_moon_phase_image, view_panel, view_popup};
use cosmic::surface::action::{app_popup, destroy_popup};


#[derive(Debug, Clone)]
pub enum Msg {
    Tick,
    TogglePopup,
    // Surface(cosmic::surface::Action<Msg>),
    PopupClosed(Id),
    ToggleCityPicker,
    CitySearchChanged(String),
    SelectCity(Option<String>),
}

const APP_ID: &str = "com.example.CosmicAppletMoon";

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

        (
            Self {
                core,
                popup: None,
                info,
                icon,
                config_handler,
                config,
                city_picker_open: false,
                city_search: String::new(),
                all_cities,
            },
            Task::none(),
        )
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
                ))
                .max_width(580.0)
                .into()
        } else {
            cosmic::widget::text("").into()
        }
    }


    fn update(&mut self, msg: Msg) -> Task<Action<Msg>> {
        match msg {
            Msg::Tick => {
                self.info = MoonInfo::compute(self.config.manual_tz.as_deref());
                self.icon = get_moon_phase_image(self.info.day);
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

