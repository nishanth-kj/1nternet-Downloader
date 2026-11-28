use iced::Color;
use serde::{Deserialize, Serialize};

/// Which visual style the main window uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ThemeMode {
    /// Follow the Windows "app mode" (light or dark) setting (default).
    #[default]
    System,
    /// Light, dense, Windows-classic look.
    Classic,
    Dark,
}

impl ThemeMode {
    pub const ALL: [ThemeMode; 3] = [ThemeMode::System, ThemeMode::Classic, ThemeMode::Dark];

    /// The concrete look to paint: System becomes Classic or Dark.
    pub fn resolve(self) -> ThemeMode {
        match self {
            ThemeMode::System => {
                if system_prefers_dark() {
                    ThemeMode::Dark
                } else {
                    ThemeMode::Classic
                }
            }
            other => other,
        }
    }
}

impl std::fmt::Display for ThemeMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ThemeMode::System => "Same as Windows",
            ThemeMode::Classic => "Classic (light)",
            ThemeMode::Dark => "Dark",
        })
    }
}

/// Reads `AppsUseLightTheme` (0 = dark) from the current user's personalization settings.
#[cfg(windows)]
pub fn system_prefers_dark() -> bool {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    std::process::Command::new("reg")
        .args([
            "query",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize",
            "/v",
            "AppsUseLightTheme",
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("0x0"))
        .unwrap_or(false)
}

#[cfg(not(windows))]
pub fn system_prefers_dark() -> bool {
    false
}

/// Highlight colour for selection, progress bars, buttons and links.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Accent {
    #[default]
    Green,
    Blue,
    Teal,
    Purple,
    Orange,
    Red,
    Graphite,
}

impl Accent {
    pub const ALL: [Accent; 7] =
        [Accent::Green, Accent::Blue, Accent::Teal, Accent::Purple, Accent::Orange, Accent::Red, Accent::Graphite];

    /// The accent tuned for a light or a dark background.
    pub fn color(self, dark: bool) -> Color {
        match (self, dark) {
            (Accent::Green, false) => rgb(0x10, 0x7C, 0x10),
            (Accent::Green, true) => rgb(0x5C, 0xC8, 0x5C),
            (Accent::Blue, false) => rgb(0x1F, 0x6F, 0xD1),
            (Accent::Blue, true) => rgb(0x4C, 0xA0, 0xFF),
            (Accent::Teal, false) => rgb(0x00, 0x7C, 0x80),
            (Accent::Teal, true) => rgb(0x3C, 0xC4, 0xC4),
            (Accent::Purple, false) => rgb(0x6B, 0x46, 0xC1),
            (Accent::Purple, true) => rgb(0xA7, 0x8B, 0xFA),
            (Accent::Orange, false) => rgb(0xC2, 0x5E, 0x00),
            (Accent::Orange, true) => rgb(0xF5, 0x9E, 0x3B),
            (Accent::Red, false) => rgb(0xC4, 0x2B, 0x1C),
            (Accent::Red, true) => rgb(0xF0, 0x6A, 0x60),
            (Accent::Graphite, false) => rgb(0x4A, 0x55, 0x62),
            (Accent::Graphite, true) => rgb(0xA8, 0xB2, 0xBD),
        }
    }
}

impl std::fmt::Display for Accent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Accent::Green => "Green",
            Accent::Blue => "Blue",
            Accent::Teal => "Teal",
            Accent::Purple => "Purple",
            Accent::Orange => "Orange",
            Accent::Red => "Red",
            Accent::Graphite => "Graphite",
        })
    }
}

/// Every color the UI paints with. Widgets capture a copy, so styles stay `'static`.
#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub background: Color,
    pub panel: Color,
    pub toolbar: Color,
    pub sidebar: Color,
    pub header: Color,
    pub row: Color,
    pub row_alt: Color,
    pub hover: Color,
    pub selection: Color,
    pub selection_border: Color,
    pub border: Color,
    pub text: Color,
    pub muted: Color,
    /// Download colour: progress bars, "Downloading" status and transfer rate only.
    pub accent: Color,
    pub on_accent: Color,
    /// Neutral high-contrast colour for primary buttons, focus rings and checkboxes.
    pub strong: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
    pub track: Color,
    pub overlay: Color,
}

const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
}

impl Palette {
    /// Base palette for the mode, recoloured with the chosen accent.
    pub fn new(mode: ThemeMode, accent: Accent) -> Self {
        let dark = mode.resolve() == ThemeMode::Dark;
        let mut p = Self::for_mode(mode);
        p.accent = accent.color(dark);
        p.on_accent = if dark { rgb(0x10, 0x14, 0x10) } else { rgb(0xFF, 0xFF, 0xFF) };
        p
    }

    /// Neutral greys used for selection, hover and primary buttons in each mode.
    fn neutrals(dark: bool) -> (Color, Color, Color, Color) {
        // (selection, selection_border, hover, strong)
        if dark {
            (rgb(0x3A, 0x3E, 0x44), rgb(0x62, 0x68, 0x70), rgb(0x2E, 0x31, 0x35), rgb(0xE6, 0xE8, 0xEB))
        } else {
            (rgb(0xD5, 0xD9, 0xDF), rgb(0x9A, 0xA1, 0xAB), rgb(0xEB, 0xED, 0xF0), rgb(0x2B, 0x2F, 0x36))
        }
    }

    pub fn for_mode(mode: ThemeMode) -> Self {
        let dark = mode.resolve() == ThemeMode::Dark;
        let (selection, selection_border, hover, strong) = Self::neutrals(dark);
        let mut p = Self::base(mode);
        p.selection = selection;
        p.selection_border = selection_border;
        p.hover = hover;
        p.strong = strong;
        p
    }

    fn base(mode: ThemeMode) -> Self {
        match mode.resolve() {
            ThemeMode::Classic => Self {
                background: rgb(0xFF, 0xFF, 0xFF),
                panel: rgb(0xF3, 0xF4, 0xF6),
                toolbar: rgb(0xEC, 0xEF, 0xF3),
                sidebar: rgb(0xF7, 0xF9, 0xFC),
                header: rgb(0xE6, 0xE9, 0xEE),
                row: rgb(0xFF, 0xFF, 0xFF),
                row_alt: rgb(0xF6, 0xF8, 0xFB),
                hover: rgb(0xE5, 0xF1, 0xFB),
                selection: rgb(0xCC, 0xE4, 0xFF),
                selection_border: rgb(0x7F, 0xB2, 0xF0),
                border: rgb(0xC9, 0xCE, 0xD6),
                text: rgb(0x1B, 0x1F, 0x24),
                muted: rgb(0x5F, 0x67, 0x70),
                accent: rgb(0x1F, 0x6F, 0xD1),
                on_accent: rgb(0xFF, 0xFF, 0xFF),
                strong: rgb(0x2B, 0x2F, 0x36),
                success: rgb(0x1E, 0x8E, 0x3E),
                warning: rgb(0xB0, 0x6D, 0x0B),
                danger: rgb(0xC6, 0x28, 0x28),
                track: rgb(0xE1, 0xE5, 0xEB),
                overlay: Color { a: 0.35, ..rgb(0x10, 0x14, 0x1A) },
            },
            _ => Self {
                background: rgb(0x20, 0x20, 0x20),
                panel: rgb(0x2B, 0x2B, 0x2B),
                toolbar: rgb(0x2B, 0x2B, 0x2B),
                sidebar: rgb(0x25, 0x25, 0x25),
                header: rgb(0x2D, 0x2D, 0x2D),
                row: rgb(0x20, 0x20, 0x20),
                row_alt: rgb(0x25, 0x25, 0x25),
                hover: rgb(0x33, 0x38, 0x40),
                selection: rgb(0x26, 0x4F, 0x78),
                selection_border: rgb(0x3C, 0x78, 0xB4),
                border: rgb(0x3D, 0x3D, 0x3D),
                text: rgb(0xF0, 0xF0, 0xF0),
                muted: rgb(0xA0, 0xA0, 0xA0),
                accent: rgb(0x4C, 0xA0, 0xFF),
                on_accent: rgb(0x0B, 0x12, 0x1C),
                strong: rgb(0xE6, 0xE8, 0xEB),
                success: rgb(0x4C, 0xC2, 0x5E),
                warning: rgb(0xE0, 0xA8, 0x30),
                danger: rgb(0xF2, 0x5C, 0x54),
                track: rgb(0x38, 0x38, 0x38),
                overlay: Color { a: 0.55, ..rgb(0x00, 0x00, 0x00) },
            },
        }
    }
}

impl Palette {
    /// An iced theme built from this palette, so checkboxes, dropdowns, scrollbars and
    /// text editors match the rest of the window.
    pub fn iced_theme(&self, dark: bool) -> iced::Theme {
        iced::Theme::custom(
            if dark { "1IDM Dark".into() } else { "1IDM Light".into() },
            iced::theme::Palette {
                background: self.background,
                text: self.text,
                primary: self.strong,
                success: self.success,
                danger: self.danger,
            },
        )
    }
}
