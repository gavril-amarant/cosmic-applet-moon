mod moon_applet;
mod moon_data;
mod moon_horizon;
mod view;
mod config;

fn main() -> cosmic::iced::Result {
    cosmic::applet::run::<moon_applet::MoonApplet>(())
}   