mod moon_applet;
mod moon_data;
mod moon_rise_set;
mod view;

fn main() -> cosmic::iced::Result {
    cosmic::applet::run::<moon_applet::MoonApplet>(())
}   