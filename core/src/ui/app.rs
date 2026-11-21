use crate::config::{Config, Settings};
use crate::model::{DownloadId, DownloadSnapshot, DownloadStatus};
use crate::service::QueueManager;
use crate::ui::theme::IdmColors;
use iced::widget::{
    button, column, container, horizontal_space, progress_bar, row, scrollable, text, text_input, vertical_space,
};
use iced::{Alignment, Element, Length, Subscription, Task, Theme};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    All,
    Downloading,
    Queued,
    Completed,
    Paused,
    Failed,
    Torrents,
    Settings,
}

#[derive(Debug, Clone)]
pub enum Message {
    SelectCategory(Category),
    SelectDownload(DownloadId),
    SearchChanged(String),
    AddUrlChanged(String),
    ToggleAddDialog(bool),
    SubmitAddDownload,
    PauseDownload(DownloadId),
    ResumeDownload(DownloadId),
    DeleteDownload(DownloadId),
    PauseAll,
    ResumeAll,
    SettingsDownloadDirChanged(String),
    SettingsConcurrentChanged(String),
    SettingsSegmentsChanged(String),
    SaveSettings,
    SnapshotReceived(DownloadSnapshot),
    Tick,
}

pub struct IcedApp {
    queue_manager: Arc<QueueManager>,
    downloads: Vec<DownloadSnapshot>,
    selected_category: Category,
    selected_id: Option<DownloadId>,
    search_query: String,
    add_url_input: String,
    show_add_dialog: bool,
    settings: Settings,
    status_filter_counts: (usize, usize, usize, usize, usize, usize, usize),
}

impl IcedApp {
    pub fn new(queue_manager: Arc<QueueManager>) -> (Self, Task<Message>) {
        let config = Config::load();
        let settings = config.get();
        let initial_downloads = queue_manager.get_all_snapshots();

        let mut app = Self {
            queue_manager,
            downloads: initial_downloads,
            selected_category: Category::All,
            selected_id: None,
            search_query: String::new(),
            add_url_input: String::new(),
            show_add_dialog: false,
            settings,
            status_filter_counts: (0, 0, 0, 0, 0, 0, 0),
        };
        app.update_counts();
        (app, Task::none())
    }

    pub fn title(&self) -> String {
        let active = self
            .downloads
            .iter()
            .filter(|d| d.status == DownloadStatus::Downloading)
            .count();

        if active > 0 {
            format!("1nternet Downloader (1IDM) — {} Active Downloads", active)
        } else {
            "1nternet Downloader (1IDM) — Fast & Resilient Acceleration".to_string()
        }
    }

    pub fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn update_counts(&mut self) {
        let total = self.downloads.len();
        let downloading = self
            .downloads
            .iter()
            .filter(|d| d.status == DownloadStatus::Downloading)
            .count();
        let queued = self
            .downloads
            .iter()
            .filter(|d| d.status == DownloadStatus::Queued)
            .count();
        let completed = self
            .downloads
            .iter()
            .filter(|d| d.status == DownloadStatus::Completed)
            .count();
        let paused = self
            .downloads
            .iter()
            .filter(|d| d.status == DownloadStatus::Paused)
            .count();
        let failed = self
            .downloads
            .iter()
            .filter(|d| d.status == DownloadStatus::Error)
            .count();
        let torrents = self.downloads.iter().filter(|d| d.is_torrent).count();

        self.status_filter_counts = (total, downloading, queued, completed, paused, failed, torrents);
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SelectCategory(cat) => {
                self.selected_category = cat;
            }
            Message::SelectDownload(id) => {
                self.selected_id = Some(id);
            }
            Message::SearchChanged(q) => {
                self.search_query = q;
            }
            Message::AddUrlChanged(u) => {
                self.add_url_input = u;
            }
            Message::ToggleAddDialog(show) => {
                self.show_add_dialog = show;
                if !show {
                    self.add_url_input.clear();
                }
            }
            Message::SubmitAddDownload => {
                let url = self.add_url_input.trim().to_string();
                if !url.is_empty() {
                    let qm = Arc::clone(&self.queue_manager);
                    let url_clone = url.clone();
                    crate::service::spawn(async move {
                        let _ = qm.add_download(&url_clone, None, true).await;
                    });
                    self.show_add_dialog = false;
                    self.add_url_input.clear();
                }
            }
            Message::PauseDownload(id) => {
                self.queue_manager.pause_download(id);
            }
            Message::ResumeDownload(id) => {
                self.queue_manager.resume_download(id);
            }
            Message::DeleteDownload(id) => {
                self.queue_manager.delete_download(id, false);
                self.downloads.retain(|d| d.id != id);
                if self.selected_id == Some(id) {
                    self.selected_id = None;
                }
                self.update_counts();
            }
            Message::PauseAll => {
                self.queue_manager.pause_all();
            }
            Message::ResumeAll => {
                self.queue_manager.resume_all();
            }
            Message::SettingsDownloadDirChanged(dir) => {
                self.settings.download_dir = dir;
            }
            Message::SettingsConcurrentChanged(val) => {
                if let Ok(num) = val.parse::<u32>() {
                    self.settings.max_concurrent_downloads = num;
                }
            }
            Message::SettingsSegmentsChanged(val) => {
                if let Ok(num) = val.parse::<u32>() {
                    self.settings.max_segments = num;
                }
            }
            Message::SaveSettings => {
                let config = Config::load();
                let current = self.settings.clone();
                let _ = config.update(|s| *s = current);
            }
            Message::SnapshotReceived(snap) => {
                if let Some(pos) = self.downloads.iter().position(|d| d.id == snap.id) {
                    self.downloads[pos] = snap;
                } else {
                    self.downloads.push(snap);
                }
                self.update_counts();
            }
            Message::Tick => {
                self.downloads = self.queue_manager.get_all_snapshots();
                self.update_counts();
            }
        }
        Task::none()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let sub = self.queue_manager.subscribe();
        Subscription::batch([
            iced::time::every(std::time::Duration::from_millis(250)).map(|_| Message::Tick),
            Subscription::run_with_id(
                "queue_events",
                futures::stream::unfold(sub, |mut sub| async move {
                    loop {
                        match sub.recv().await {
                            Ok(snap) => return Some((Message::SnapshotReceived(snap), sub)),
                            Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                            Err(tokio::sync::broadcast::error::RecvError::Closed) => return None,
                        }
                    }
                }),
            ),
        ])
    }

    pub fn view(&self) -> Element<'_, Message> {
        if self.show_add_dialog {
            return self.view_add_dialog();
        }

        let sidebar = self.view_sidebar();
        let content: Element<'_, Message> = match self.selected_category {
            Category::Settings => self.view_settings(),
            _ => self.view_main_content(),
        };

        row![sidebar, content].into()
    }

    fn view_sidebar(&self) -> Element<'_, Message> {
        let (total, dl, queued, done, paused, failed, torrents) = self.status_filter_counts;

        let make_item = |label: &str, cat: Category, count: usize| {
            let is_selected = self.selected_category == cat;
            let title_text = format!("{} ({})", label, count);

            let btn = button(text(title_text).size(14))
                .width(Length::Fill)
                .padding([8, 12])
                .on_press(Message::SelectCategory(cat));

            if is_selected {
                btn.style(|_t, _s| button::Style {
                    background: Some(IdmColors::CARD_SELECTED.into()),
                    text_color: IdmColors::PRIMARY,
                    border: iced::border::rounded(6),
                    ..Default::default()
                })
            } else {
                btn.style(|_t, _s| button::Style {
                    background: None,
                    text_color: IdmColors::TEXT_MUTED,
                    ..Default::default()
                })
            }
        };

        let title_header = container(
            column![
                text("1nternet Downloader").size(18).color(IdmColors::TEXT_PRIMARY),
                text("v1.0.0 (Rust + Iced)").size(11).color(IdmColors::TEXT_MUTED),
            ]
            .spacing(2),
        )
        .padding(14);

        let nav = column![
            make_item("All Downloads", Category::All, total),
            make_item("Downloading", Category::Downloading, dl),
            make_item("Queued", Category::Queued, queued),
            make_item("Completed", Category::Completed, done),
            make_item("Paused", Category::Paused, paused),
            make_item("Failed", Category::Failed, failed),
            make_item("Torrents", Category::Torrents, torrents),
            vertical_space().height(10),
            button(text("⚙ Settings").size(14))
                .width(Length::Fill)
                .padding([8, 12])
                .on_press(Message::SelectCategory(Category::Settings))
                .style(|_t, _s| button::Style {
                    background: None,
                    text_color: IdmColors::TEXT_MUTED,
                    ..Default::default()
                }),
        ]
        .spacing(4);

        container(column![title_header, nav].spacing(8))
            .width(Length::Fixed(220.0))
            .height(Length::Fill)
            .style(|_t| container::Style {
                background: Some(IdmColors::SIDEBAR.into()),
                border: iced::border::rounded(0),
                ..Default::default()
            })
            .into()
    }

    fn view_main_content(&self) -> Element<'_, Message> {
        let toolbar = self.view_toolbar();
        let table = self.view_downloads_table();
        let details = self.view_details_pane();

        column![toolbar, table, details]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn view_toolbar(&self) -> Element<'_, Message> {
        row![
            button(text("+ Add Download").size(14))
                .padding([8, 14])
                .on_press(Message::ToggleAddDialog(true)),
            button(text("▶ Resume All").size(14))
                .padding([8, 14])
                .on_press(Message::ResumeAll),
            button(text("⏸ Pause All").size(14))
                .padding([8, 14])
                .on_press(Message::PauseAll),
            horizontal_space(),
            text_input("Search downloads...", &self.search_query)
                .on_input(Message::SearchChanged)
                .width(Length::Fixed(240.0))
                .padding(8),
        ]
        .padding(12)
        .spacing(10)
        .align_y(Alignment::Center)
        .into()
    }

    fn view_downloads_table(&self) -> Element<'_, Message> {
        let filtered: Vec<&DownloadSnapshot> = self
            .downloads
            .iter()
            .filter(|d| {
                if !self.search_query.is_empty()
                    && !d.filename.to_lowercase().contains(&self.search_query.to_lowercase())
                {
                    return false;
                }
                match self.selected_category {
                    Category::All => true,
                    Category::Downloading => d.status == DownloadStatus::Downloading,
                    Category::Queued => d.status == DownloadStatus::Queued,
                    Category::Completed => d.status == DownloadStatus::Completed,
                    Category::Paused => d.status == DownloadStatus::Paused,
                    Category::Failed => d.status == DownloadStatus::Error,
                    Category::Torrents => d.is_torrent,
                    Category::Settings => false,
                }
            })
            .collect();

        let header = container(
            row![
                text("Filename").width(Length::FillPortion(4)).size(13).color(IdmColors::TEXT_MUTED),
                text("Size").width(Length::FillPortion(2)).size(13).color(IdmColors::TEXT_MUTED),
                text("Progress").width(Length::FillPortion(3)).size(13).color(IdmColors::TEXT_MUTED),
                text("Speed").width(Length::FillPortion(2)).size(13).color(IdmColors::TEXT_MUTED),
                text("Status").width(Length::FillPortion(2)).size(13).color(IdmColors::TEXT_MUTED),
                text("Actions").width(Length::Fixed(140.0)).size(13).color(IdmColors::TEXT_MUTED),
            ]
            .spacing(8)
            .padding([8, 12]),
        )
        .style(|_t| container::Style {
            background: Some(IdmColors::CARD.into()),
            border: iced::border::rounded(4),
            ..Default::default()
        });

        let mut list_col = column![header].spacing(4).padding(8);

        for snap in filtered {
            let is_selected = self.selected_id == Some(snap.id);
            let size_str = format_bytes(snap.total_bytes);
            let speed_str = if snap.status == DownloadStatus::Downloading {
                format!("{}/s", format_bytes(snap.speed_bytes_per_sec as u64))
            } else {
                "-".to_string()
            };

            let status_badge = match snap.status {
                DownloadStatus::Downloading => text("Downloading").color(IdmColors::PRIMARY),
                DownloadStatus::Completed => text("Completed").color(IdmColors::SUCCESS),
                DownloadStatus::Paused => text("Paused").color(IdmColors::WARNING),
                DownloadStatus::Error => text("Error").color(IdmColors::DANGER),
                DownloadStatus::Queued => text("Queued").color(IdmColors::TEXT_MUTED),
            };

            let action_btn = match snap.status {
                DownloadStatus::Downloading => button(text("⏸").size(12))
                    .padding([4, 8])
                    .on_press(Message::PauseDownload(snap.id)),
                DownloadStatus::Paused | DownloadStatus::Error | DownloadStatus::Queued => button(text("▶").size(12))
                    .padding([4, 8])
                    .on_press(Message::ResumeDownload(snap.id)),
                DownloadStatus::Completed => button(text("✓").size(12)).padding([4, 8]),
            };

            let delete_btn = button(text("🗑").size(12))
                .padding([4, 8])
                .on_press(Message::DeleteDownload(snap.id));

            let item_row = button(
                row![
                    text(&snap.filename).width(Length::FillPortion(4)).size(13),
                    text(size_str).width(Length::FillPortion(2)).size(13),
                    column![
                        progress_bar(0.0..=100.0, snap.progress_percent),
                        text(format!("{:.1}%", snap.progress_percent)).size(10).color(IdmColors::TEXT_MUTED)
                    ]
                    .width(Length::FillPortion(3)),
                    text(speed_str).width(Length::FillPortion(2)).size(13),
                    container(status_badge).width(Length::FillPortion(2)),
                    row![action_btn, delete_btn].spacing(6).width(Length::Fixed(140.0)),
                ]
                .spacing(8)
                .align_y(Alignment::Center)
                .padding([6, 10]),
            )
            .width(Length::Fill)
            .on_press(Message::SelectDownload(snap.id))
            .style(move |_t, _s| button::Style {
                background: Some(if is_selected {
                    IdmColors::CARD_SELECTED.into()
                } else {
                    IdmColors::CARD.into()
                }),
                border: iced::border::rounded(4),
                ..Default::default()
            });

            list_col = list_col.push(item_row);
        }

        scrollable(list_col).height(Length::FillPortion(3)).into()
    }

    fn view_details_pane(&self) -> Element<'_, Message> {
        let selected = self
            .selected_id
            .and_then(|id| self.downloads.iter().find(|d| d.id == id));

        let content = if let Some(snap) = selected {
            let mut seg_col = column![
                text("Active Dynamic Segments & Worker Streams")
                    .size(13)
                    .color(IdmColors::TEXT_PRIMARY)
            ].spacing(6);

            if snap.segments.is_empty() {
                seg_col = seg_col.push(text("Single stream active").size(12).color(IdmColors::TEXT_MUTED));
            } else {
                for seg in &snap.segments {
                    let seg_prog = seg.progress_ratio() * 100.0;
                    let seg_line = row![
                        text(format!("Worker #{}", seg.segment_id)).size(11).width(Length::Fixed(80.0)),
                        progress_bar(0.0..=100.0, seg_prog).width(Length::FillPortion(3)),
                        text(format!("{:.1}% ({}..{})", seg_prog, format_bytes(seg.start_offset), format_bytes(seg.end_offset)))
                            .size(11)
                            .color(IdmColors::TEXT_MUTED)
                            .width(Length::FillPortion(2)),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center);

                    seg_col = seg_col.push(seg_line);
                }
            }

            let meta = column![
                text(format!("URL: {}", snap.url)).size(12).color(IdmColors::TEXT_MUTED),
                text(format!("Save path: {}", snap.destination)).size(12).color(IdmColors::TEXT_MUTED),
                text(format!(
                    "Transferred: {} of {} ({:.1}%)",
                    format_bytes(snap.downloaded_bytes),
                    format_bytes(snap.total_bytes),
                    snap.progress_percent
                ))
                .size(12)
                .color(IdmColors::TEXT_MUTED),
            ]
            .spacing(4);

            column![meta, vertical_space().height(6), seg_col].spacing(6)
        } else {
            column![text("Select a download above to inspect active segments and real-time details")
                .size(13)
                .color(IdmColors::TEXT_MUTED)]
        };

        container(content)
            .padding(14)
            .width(Length::Fill)
            .height(Length::FillPortion(2))
            .style(|_t| container::Style {
                background: Some(IdmColors::SIDEBAR.into()),
                border: iced::border::rounded(0),
                ..Default::default()
            })
            .into()
    }

    fn view_add_dialog(&self) -> Element<'_, Message> {
        container(
            column![
                text("Add New Download").size(20).color(IdmColors::PRIMARY),
                vertical_space().height(10),
                text("Enter HTTP/HTTPS URL or Magnet Link:").size(14),
                text_input("https://...", &self.add_url_input)
                    .on_input(Message::AddUrlChanged)
                    .padding(10)
                    .width(Length::Fixed(480.0)),
                vertical_space().height(14),
                row![
                    button(text("Start Download").size(14))
                        .padding([8, 16])
                        .on_press(Message::SubmitAddDownload),
                    button(text("Cancel").size(14))
                        .padding([8, 16])
                        .on_press(Message::ToggleAddDialog(false)),
                ]
                .spacing(10),
            ]
            .spacing(8)
            .padding(24),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    }

    fn view_settings(&self) -> Element<'_, Message> {
        column![
            text("Settings & Acceleration Engine Configuration").size(20).color(IdmColors::PRIMARY),
            vertical_space().height(16),
            text("Default Download Folder:").size(14),
            text_input("C:\\Downloads", &self.settings.download_dir)
                .on_input(Message::SettingsDownloadDirChanged)
                .padding(8),
            vertical_space().height(10),
            text("Max Concurrent Active Downloads:").size(14),
            text_input("3", &self.settings.max_concurrent_downloads.to_string())
                .on_input(Message::SettingsConcurrentChanged)
                .padding(8),
            vertical_space().height(10),
            text("Dynamic Segments per Download (IDM Work-Stealing Workers):").size(14),
            text_input("8", &self.settings.max_segments.to_string())
                .on_input(Message::SettingsSegmentsChanged)
                .padding(8),
            vertical_space().height(16),
            button(text("Save Settings").size(14))
                .padding([10, 20])
                .on_press(Message::SaveSettings),
        ]
        .padding(24)
        .spacing(8)
        .width(Length::Fill)
        .into()
    }
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
