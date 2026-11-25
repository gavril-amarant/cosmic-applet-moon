mod moon_applet;
mod astronomy;
mod moon_horizon;
mod view;
mod config;
mod utils;
mod location_utils;

fn main() -> cosmic::iced::Result {
    cosmic::applet::run::<moon_applet::MoonApplet>(())
}   