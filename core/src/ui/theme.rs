use iced::Color;

pub struct IdmColors;

impl IdmColors {
    pub const BACKGROUND: Color = Color::from_rgb(0.09, 0.10, 0.12);
    pub const SIDEBAR: Color = Color::from_rgb(0.12, 0.13, 0.16);
    pub const CARD: Color = Color::from_rgb(0.15, 0.16, 0.20);
    pub const CARD_SELECTED: Color = Color::from_rgb(0.18, 0.22, 0.32);
    pub const BORDER: Color = Color::from_rgb(0.22, 0.24, 0.28);
    pub const PRIMARY: Color = Color::from_rgb(0.20, 0.50, 0.95);
    pub const SUCCESS: Color = Color::from_rgb(0.15, 0.70, 0.40);
    pub const WARNING: Color = Color::from_rgb(0.95, 0.65, 0.15);
    pub const DANGER: Color = Color::from_rgb(0.90, 0.25, 0.25);
    pub const TEXT_PRIMARY: Color = Color::from_rgb(0.95, 0.96, 0.98);
    pub const TEXT_MUTED: Color = Color::from_rgb(0.60, 0.63, 0.68);
}
