use cosmic::app::Core;
use cosmic::iced::window::Id;
use cosmic::{Action, Element, Task};
use crate::moon_data::MoonInfo;
use crate::view::{get_moon_phase_image, view_panel, view_popup};

const APP_ID: &str = "com.example.CosmicAppletMoon";

pub struct MoonApplet {
    pub core: Core,
    pub popup: Option<Id>,
    pub info: MoonInfo,
    pub icon: cosmic::iced::widget::image::Handle,
}

#[derive(Debug, Clone)]
pub enum Msg {
    Tick,
    TogglePopup,
    #[allow(dead_code)]
    PopupClosed(Id),
}

impl cosmic::Application for MoonApplet {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Msg;

    const APP_ID: &'static str = APP_ID;

    fn core(&self) -> &Core { &self.core }
    fn core_mut(&mut self) -> &mut Core { &mut self.core }

    fn init(core: Core, _flags: ()) -> (Self, Task<Action<Msg>>) {
        let info = MoonInfo::compute();
        let icon = get_moon_phase_image(info.day);
        (Self { core, popup: None, info, icon }, Task::none())
    }

    fn view(&self) -> Element<'_, Msg> {
        self.core.applet.autosize_window(view_panel(&self.icon)).into()
    }

    fn view_window(&self, id: Id) -> Element<'_, Msg> {
        if Some(id) == self.popup {
            self.core.applet
                .popup_container(view_popup(&self.info, &self.icon))
                .max_width(580.0)
                .into()
        } else {
            cosmic::widget::text("").into()
        }
    }

    fn update(&mut self, msg: Msg) -> Task<Action<Msg>> {
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
                popup_settings.positioner.size_limits = cosmic::iced::Limits::NONE
                    .min_width(200.0)
                    .min_height(80.0)
                    .max_width(600.0)
                    .max_height(500.0);
                return cosmic::iced::platform_specific::shell::commands::popup::get_popup(popup_settings);
            }
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

}   