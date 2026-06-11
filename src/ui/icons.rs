use iced::widget::container;
use iced::{Color, Element, Length};

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum Icon {
    Apache,
    Check,
    ChevronDown,
    ChevronRight,
    Code,
    Config,
    Copy,
    Dashboard,
    Database,
    Editor,
    External,
    Folder,
    Globe,
    Host,
    Info,
    Key,
    Lock,
    Minus,
    Php,
    Play,
    Plus,
    Refresh,
    Repo,
    Search,
    Server,
    Shield,
    Stop,
    Terminal,
    Tools,
    Trash,
    Unlock,
}

impl Icon {
    fn name(self) -> &'static str {
        match self {
            Self::Apache => "server",
            Self::Check => "check",
            Self::ChevronDown => "chevron-down",
            Self::ChevronRight => "chevron-right",
            Self::Code => "code",
            Self::Config => "gear",
            Self::Copy => "copy",
            Self::Dashboard => "gauge-high",
            Self::Database => "database",
            Self::Editor => "pen-to-square",
            Self::External => "arrow-up-right-from-square",
            Self::Folder => "folder-open",
            Self::Globe => "globe",
            Self::Host => "network-wired",
            Self::Info => "circle-info",
            Self::Key => "key",
            Self::Lock => "lock",
            Self::Minus => "minus",
            Self::Php => "file-code",
            Self::Play => "play",
            Self::Plus => "plus",
            Self::Refresh => "rotate",
            Self::Repo => "code-branch",
            Self::Search => "magnifying-glass",
            Self::Server => "desktop",
            Self::Shield => "shield-halved",
            Self::Stop => "square",
            Self::Terminal => "terminal",
            Self::Tools => "screwdriver-wrench",
            Self::Trash => "trash",
            Self::Unlock => "unlock",
        }
    }
}

pub fn solid<'a, Message: 'a>(icon: Icon, size: f32, color: Color) -> Element<'a, Message> {
    iced_font_awesome::fa_icon_solid(icon.name())
        .size(size)
        .color(color)
        .into()
}

pub fn solid_box<'a, Message: 'a>(
    icon: Icon,
    size: f32,
    color: Color,
    box_size: f32,
) -> Element<'a, Message> {
    container(solid(icon, size, color))
        .width(Length::Fixed(box_size))
        .height(Length::Fixed(box_size))
        .center_x(Length::Fixed(box_size))
        .center_y(Length::Fixed(box_size))
        .into()
}
