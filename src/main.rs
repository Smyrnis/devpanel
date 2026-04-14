mod app;
mod core;
mod messages;
mod tabs;

use app::App;
use iced::Theme;

static ICON_BYTES: &[u8] = include_bytes!("../devpanel.jpg");

fn load_window_icon() -> Option<iced::window::Icon> {
    use image::GenericImageView;
    let img = image::load_from_memory(ICON_BYTES).ok()?;
    let rgba = img.to_rgba8();
    let (w, h) = img.dimensions();
    iced::window::icon::from_rgba(rgba.into_raw(), w, h).ok()
}

fn make_fallback_icon() -> Option<iced::window::Icon> {
    let size: u32 = 32;
    let mut rgba = vec![0u8; (size * size * 4) as usize];
    for y in 0..size {
        for x in 0..size {
            let idx = ((y * size + x) * 4) as usize;
            rgba[idx + 3] = 0xFF;
            let dx = x as f32 - 15.5;
            let dy = y as f32 - 15.5;
            if dx * dx + dy * dy <= 13.0 * 13.0 {
                rgba[idx] = 0x33;
                rgba[idx + 1] = 0xBC;
                rgba[idx + 2] = 0xAC;
            }
        }
    }
    iced::window::icon::from_rgba(rgba, size, size).ok()
}

fn main() -> iced::Result {
    let icon = load_window_icon().or_else(make_fallback_icon);
    iced::application("DevPanel", App::update, App::view)
        .subscription(App::subscription)
        .theme(|_| Theme::Dark)
        .window(iced::window::Settings {
            size: iced::Size::new(1040.0, 660.0),
            min_size: Some(iced::Size::new(860.0, 560.0)),
            icon,
            ..Default::default()
        })
        .run_with(App::new)
}
