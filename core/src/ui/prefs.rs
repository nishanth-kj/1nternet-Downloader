//! Window layout preferences (columns, sort, theme, panes), saved to
//! `%APPDATA%\InternetDownloader\ui.json`. Engine settings stay in `settings.json`.

use crate::config::Config;
use crate::ui::theme::{Accent, ThemeMode};
use iced::Length;
use serde::{Deserialize, Serialize};

/// A column of the download list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Column {
    FileName,
    Type,
    Size,
    Downloaded,
    Progress,
    Status,
    TimeLeft,
    TransferRate,
    Connections,
    LastTry,
    Added,
    SavePath,
    Url,
}

impl Column {
    pub const ALL: [Column; 13] = [
        Column::FileName,
        Column::Type,
        Column::Size,
        Column::Downloaded,
        Column::Progress,
        Column::Status,
        Column::TimeLeft,
        Column::TransferRate,
        Column::Connections,
        Column::LastTry,
        Column::Added,
        Column::SavePath,
        Column::Url,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Column::FileName => "File Name",
            Column::Type => "Type",
            Column::Size => "Size",
            Column::Downloaded => "Downloaded",
            Column::Progress => "Progress",
            Column::Status => "Status",
            Column::TimeLeft => "Time Left",
            Column::TransferRate => "Transfer Rate",
            Column::Connections => "Conn.",
            Column::LastTry => "Last Try",
            Column::Added => "Added",
            Column::SavePath => "Save To",
            Column::Url => "URL",
        }
    }

    pub fn width(self) -> Length {
        match self {
            Column::FileName => Length::FillPortion(5),
            Column::Type => Length::Fixed(92.0),
            Column::Size => Length::Fixed(76.0),
            Column::Downloaded => Length::Fixed(92.0),
            Column::Progress => Length::Fixed(110.0),
            Column::Status => Length::Fixed(88.0),
            Column::TimeLeft => Length::Fixed(80.0),
            Column::TransferRate => Length::Fixed(86.0),
            Column::Connections => Length::Fixed(52.0),
            Column::LastTry => Length::Fixed(110.0),
            Column::Added => Length::Fixed(118.0),
            Column::SavePath => Length::FillPortion(3),
            Column::Url => Length::FillPortion(4),
        }
    }

    /// File name can't be hidden; everything else is optional.
    pub fn hideable(self) -> bool {
        self != Column::FileName
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UiPrefs {
    pub theme: ThemeMode,
    pub accent: Accent,
    pub columns: Vec<Column>,
    pub sort: Column,
    pub sort_descending: bool,
    pub show_categories: bool,
    pub show_details: bool,
    /// Put new files in `<download folder>\<Category>` by default.
    pub category_subfolders: bool,
    /// Open the live download window when a new download starts.
    pub show_progress_on_start: bool,
    /// Show the file in Explorer when a download finishes.
    pub open_folder_when_done: bool,
    /// Ask before removing downloads from the list.
    pub confirm_remove: bool,
    /// Show the connections table in the download window.
    pub progress_details: bool,
    /// Show "Download complete" when a download finishes.
    pub show_complete_dialog: bool,
    /// Offer to download links copied to the clipboard.
    pub monitor_clipboard: bool,
}

impl Default for UiPrefs {
    fn default() -> Self {
        Self {
            theme: ThemeMode::System,
            accent: Accent::Green,
            columns: vec![
                Column::FileName,
                Column::Size,
                Column::Progress,
                Column::Status,
                Column::TimeLeft,
                Column::TransferRate,
                Column::Connections,
                Column::LastTry,
            ],
            sort: Column::Added,
            sort_descending: true,
            show_categories: true,
            show_details: true,
            category_subfolders: false,
            show_progress_on_start: true,
            open_folder_when_done: false,
            confirm_remove: true,
            progress_details: false,
            show_complete_dialog: true,
            monitor_clipboard: false,
        }
    }
}

impl UiPrefs {
    fn path() -> std::path::PathBuf {
        Config::app_data_dir().join("ui.json")
    }

    pub fn load() -> Self {
        let mut prefs: UiPrefs = std::fs::read_to_string(Self::path())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        prefs.sanitize();
        prefs
    }

    pub fn save(&self) {
        let path = Self::path();
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        match serde_json::to_string_pretty(self) {
            Ok(json) => {
                if let Err(e) = std::fs::write(&path, json) {
                    tracing::warn!("Could not save UI preferences to {}: {}", path.display(), e);
                }
            }
            Err(e) => tracing::warn!("Could not serialize UI preferences: {}", e),
        }
    }

    /// Keeps File Name first, drops duplicates, and keeps columns in canonical order.
    fn sanitize(&mut self) {
        let chosen = std::mem::take(&mut self.columns);
        self.columns = Column::ALL
            .iter()
            .copied()
            .filter(|c| *c == Column::FileName || chosen.contains(c))
            .collect();
    }

    pub fn toggle_column(&mut self, col: Column) {
        if !col.hideable() {
            return;
        }
        if self.columns.contains(&col) {
            self.columns.retain(|c| *c != col);
        } else {
            self.columns.push(col);
        }
        self.sanitize();
    }
}
