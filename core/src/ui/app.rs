//! Main window state and message handling. Rendering lives in `view.rs` and `dialogs.rs`.

use crate::config::{Config, Settings};
use crate::config::system::{
    MAX_CONCURRENT_DOWNLOADS, MAX_RETRIES_LIMIT, MAX_SEGMENTS, MIN_CONCURRENT_DOWNLOADS, MIN_RETRIES, MIN_SEGMENTS,
};
use crate::model::{DownloadId, DownloadSnapshot, DownloadStatus};
use crate::service::QueueManager;
use crate::ui::format::{self, FileCategory};
use crate::ui::prefs::{Column, UiPrefs};
use crate::ui::theme::{Accent, Palette, ThemeMode};
use crate::ui::torrent_ui::{CreateTorrentForm, FilePriority, PieceSize, TorrentDraft, TorrentOpt, TorrentTab};
use crate::service::TorrentDetails;
use crate::ui::tray::{Tray, TrayAction};
use iced::keyboard::{self, key::Named, Key, Modifiers};
use iced::widget::{text_editor, text_input};
use iced::{event, mouse, window, Event, Point, Subscription, Task, Theme};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

const DOUBLE_CLICK: Duration = Duration::from_millis(450);

/// Left-hand tree selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filter {
    All,
    Category(FileCategory),
    Unfinished,
    Finished,
    Downloading,
    Queued,
    Paused,
    Failed,
    Torrents,
}

impl Filter {
    pub fn matches(self, d: &DownloadSnapshot) -> bool {
        match self {
            Filter::All => true,
            Filter::Category(c) => FileCategory::from_filename(&d.filename) == c,
            Filter::Unfinished => d.status != DownloadStatus::Completed,
            Filter::Finished => d.status == DownloadStatus::Completed,
            Filter::Downloading => d.status == DownloadStatus::Downloading,
            Filter::Queued => d.status == DownloadStatus::Queued,
            Filter::Paused => d.status == DownloadStatus::Paused,
            Filter::Failed => d.status == DownloadStatus::Error,
            Filter::Torrents => d.is_torrent,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuKind {
    Tasks,
    File,
    Downloads,
    Torrent,
    View,
    Help,
}

impl MenuKind {
    pub const ALL: [MenuKind; 6] = [MenuKind::Tasks, MenuKind::File, MenuKind::Downloads, MenuKind::Torrent, MenuKind::View, MenuKind::Help];

    pub fn label(self) -> &'static str {
        match self {
            MenuKind::Tasks => "Tasks",
            MenuKind::File => "File",
            MenuKind::Downloads => "Downloads",
            MenuKind::Torrent => "Torrent",
            MenuKind::View => "View",
            MenuKind::Help => "Help",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailsTab {
    General,
    Connections,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionsTab {
    General,
    Connection,
    Torrent,
    Appearance,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Dialog {
    AddUrl,
    Batch,
    Options,
    ConfirmDelete(Vec<DownloadId>),
    Properties(DownloadId),
    Complete(DownloadId),
    TorrentDetails(DownloadId),
    CreateTorrent,
    About,
}

/// Per-row facts the engine snapshot doesn't carry, tracked while the app runs.
#[derive(Debug, Clone, Copy)]
pub struct RowMeta {
    pub added_at: i64,
    pub last_try_at: Option<i64>,
    pub last_status: DownloadStatus,
}

#[derive(Debug, Clone, Default)]
pub struct AddForm {
    pub url: String,
    pub filename: String,
    pub filename_edited: bool,
    pub folder: String,
    pub folder_edited: bool,
    pub start_now: bool,
    pub category_subfolder: bool,
    /// Category picked by hand; None means "detect from the file name".
    pub category: Option<FileCategory>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct OptionsDraft {
    pub download_dir: String,
    pub max_concurrent: String,
    pub max_segments: String,
    pub max_retries: String,
    pub speed_limit: String,
    pub theme: ThemeMode,
    pub accent: Accent,
    pub category_subfolders: bool,
    pub show_progress_on_start: bool,
    pub open_folder_when_done: bool,
    pub confirm_remove: bool,
    pub start_with_windows: bool,
    pub close_to_tray: bool,
    pub start_minimized: bool,
    pub show_complete_dialog: bool,
    pub monitor_clipboard: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    SnapshotReceived(DownloadSnapshot),
    CursorMoved(Point),
    ModifiersChanged(Modifiers),
    KeyPressed(Key, Modifiers),

    SelectFilter(Filter),
    ToggleAllExpanded,
    SearchChanged(String),
    FocusSearch,
    RowPressed(DownloadId),
    SelectAll,
    RowRightPressed(DownloadId),
    SortBy(Column),

    OpenMenu(MenuKind),
    CloseMenus,

    ResumeSelected,
    PauseSelected,
    ResumeAll,
    PauseAll,
    RequestDeleteSelected,
    RequestDeleteCompleted,
    DeleteWithFile(bool),
    ConfirmDelete,
    OpenFile,
    OpenFolder,
    CopyUrls,
    ShowProperties,
    PauseOne(DownloadId),
    ResumeOne(DownloadId),
    RemoveOne(DownloadId),

    ToggleColumn(Column),
    ToggleCategories,
    ToggleDetails,
    SetTheme(ThemeMode),
    SetDetailsTab(DetailsTab),

    OpenAddDialog,
    ClipboardForAdd(Option<String>),
    PasteIntoAdd,
    AddUrlChanged(String),
    AddFilenameChanged(String),
    AddFolderChanged(String),
    AddStartNow(bool),
    AddCategorySubfolder(bool),
    AddCategoryPicked(FileCategory),
    SubmitAdd,
    AddLater,
    ToggleProgressDetails,
    DeleteDontAsk(bool),
    CompleteDontShow(bool),
    ClipboardPoll(Option<String>),

    OpenBatchDialog,
    BatchFromClipboard,
    ClipboardForBatch(Option<String>),
    BatchAction(text_editor::Action),
    BatchStartNow(bool),
    SubmitBatch,

    OpenOptions,
    OpenAppearance,
    SetOptionsTab(OptionsTab),
    OptDownloadDir(String),
    OptConcurrent(String),
    OptSegments(String),
    OptRetries(String),
    OptSpeedLimit(String),
    OptTheme(ThemeMode),
    OptAccent(Accent),
    SetAccent(Accent),
    OptCategorySubfolders(bool),
    OptShowProgress(bool),
    OptOpenFolderDone(bool),
    OptConfirmRemove(bool),
    OptStartWithWindows(bool),
    OptCloseToTray(bool),
    OptStartMinimized(bool),
    WindowCloseRequested(window::Id),
    ShowWindow,
    OptShowComplete(bool),
    OptMonitorClipboard(bool),
    SaveOptions,

    OpenAbout,
    // torrents
    OpenTorrentDetails(DownloadId, TorrentTab),
    TorrentTab(TorrentTab),
    TorrentFilePriority(i32, FilePriority),
    TorrentAllFiles(bool),
    TorrentSequential(DownloadId, bool),
    TorrentRecheck(DownloadId),
    TorrentReannounce(DownloadId),
    TrackerInput(String),
    TrackerAdd,
    OpenCreateTorrent,
    CtSource(String),
    CtOutput(String),
    CtTrackers(text_editor::Action),
    CtWebSeeds(String),
    CtComment(String),
    CtPiece(PieceSize),
    CtPrivate(bool),
    CtSeed(bool),
    CtSubmit,
    CtDone(Result<String, String>),
    OptTorrent(TorrentOpt),
    OpenTorrentSettings,
    CloseDialog,
    Exit,
}

pub struct IcedApp {
    pub(super) queue_manager: Arc<QueueManager>,
    pub(super) downloads: Vec<DownloadSnapshot>,
    pub(super) meta: HashMap<DownloadId, RowMeta>,
    pub(super) filter: Filter,
    pub(super) all_expanded: bool,
    pub(super) search: String,
    pub(super) selected: Vec<DownloadId>,
    pub(super) anchor: Option<DownloadId>,
    pub(super) last_click: Option<(DownloadId, Instant)>,
    pub(super) prefs: UiPrefs,
    pub(super) palette: Palette,
    pub(super) settings: Settings,
    pub(super) dialog: Option<Dialog>,
    pub(super) menu: Option<MenuKind>,
    pub(super) context_menu: Option<Point>,
    pub(super) cursor: Point,
    pub(super) modifiers: Modifiers,
    pub(super) details_tab: DetailsTab,
    pub(super) add: AddForm,
    pub(super) batch: text_editor::Content,
    pub(super) batch_start_now: bool,
    pub(super) options_tab: OptionsTab,
    pub(super) options: OptionsDraft,
    pub(super) delete_with_file: bool,
    pub(super) status_note: Option<String>,
    /// The look actually painted (System resolved to Classic or Dark).
    pub(super) resolved_theme: ThemeMode,
    /// False while loading the initial list, so old downloads don't pop up windows.
    started: bool,
    ticks: u32,
    pub(super) iced_theme: Theme,
    title_bar_dark: Option<bool>,
    pub(super) delete_dont_ask: bool,
    pub(super) complete_dont_show: bool,
    last_clip: Option<String>,
    clip_baseline: bool,
    pub(super) torrent_tab: TorrentTab,
    pub(super) torrent_view: Option<(DownloadId, TorrentDetails)>,
    pub(super) tracker_input: String,
    pub(super) create_form: CreateTorrentForm,
    pub(super) torrent_draft: TorrentDraft,
    tray: Option<Tray>,
    window_hidden: bool,
}

pub(super) fn search_input_id() -> text_input::Id {
    text_input::Id::new("main-search")
}

pub(super) fn add_url_input_id() -> text_input::Id {
    text_input::Id::new("add-url")
}

fn now() -> i64 {
    chrono::Utc::now().timestamp()
}

impl IcedApp {
    pub fn new(queue_manager: Arc<QueueManager>) -> (Self, Task<Message>) {
        let settings = Config::load().get();
        let prefs = UiPrefs::load();
        let mut app = Self {
            queue_manager,
            downloads: Vec::new(),
            meta: HashMap::new(),
            filter: Filter::All,
            all_expanded: true,
            search: String::new(),
            selected: Vec::new(),
            anchor: None,
            last_click: None,
            palette: Palette::new(prefs.theme, prefs.accent),
            prefs,
            settings,
            dialog: None,
            menu: None,
            context_menu: None,
            cursor: Point::ORIGIN,
            modifiers: Modifiers::default(),
            details_tab: DetailsTab::General,
            add: AddForm::default(),
            batch: text_editor::Content::new(),
            batch_start_now: true,
            options_tab: OptionsTab::General,
            options: OptionsDraft::default(),
            delete_with_file: false,
            status_note: None,
            resolved_theme: ThemeMode::Classic,
            started: false,
            ticks: 0,
            iced_theme: Theme::Light,
            title_bar_dark: None,
            delete_dont_ask: false,
            complete_dont_show: false,
            last_clip: None,
            clip_baseline: false,
            torrent_tab: TorrentTab::General,
            torrent_view: None,
            tracker_input: String::new(),
            create_form: CreateTorrentForm::default(),
            torrent_draft: TorrentDraft::from_settings(crate::config::TorrentSettings::default()),
            tray: None,
            window_hidden: false,
        };
        app.apply_theme();
        let snaps = app.queue_manager.get_all_snapshots();
        app.ingest(snaps);
        app.started = true;
        let dark = app.resolved_theme == ThemeMode::Dark;
        app.title_bar_dark = Some(dark);
        app.tray = Tray::new();
        tracing::info!("System tray icon: {}", if app.tray.is_some() { "ready" } else { "unavailable" });
        let mut startup = vec![sync_title_bar(dark)];
        if app.prefs.start_minimized && app.tray.is_some() {
            app.window_hidden = true;
            startup.push(set_window_hidden(true));
        }
        (app, Task::batch(startup))
    }

    pub fn title(&self) -> String {
        let active: Vec<&DownloadSnapshot> =
            self.downloads.iter().filter(|d| d.status == DownloadStatus::Downloading).collect();
        if active.is_empty() {
            "1nternet Downloader".to_string()
        } else {
            let speed: f64 = active.iter().map(|d| d.speed_bytes_per_sec).sum();
            format!("1nternet Downloader — {} active, {}", active.len(), format::rate(speed).trim())
        }
    }

    pub fn theme(&self) -> Theme {
        self.iced_theme.clone()
    }

    // ---- data -------------------------------------------------------------------------

    /// Replaces the list with the engine's current view and updates per-row metadata.
    fn ingest(&mut self, snaps: Vec<DownloadSnapshot>) {
        let t = now();
        for s in &snaps {
            self.note_status(s, t);
        }
        self.meta.retain(|id, _| snaps.iter().any(|s| s.id == *id));
        self.selected.retain(|id| snaps.iter().any(|s| s.id == *id));
        self.downloads = snaps;
    }

    fn note_status(&mut self, s: &DownloadSnapshot, t: i64) {
        let is_new = !self.meta.contains_key(&s.id);
        let m = self.meta.entry(s.id).or_insert(RowMeta {
            added_at: t,
            last_try_at: None,
            last_status: s.status,
        });
        if s.status == DownloadStatus::Downloading && (m.last_status != DownloadStatus::Downloading || m.last_try_at.is_none()) {
            m.last_try_at = Some(t);
        }
        let just_finished = !is_new && m.last_status != DownloadStatus::Completed && s.status == DownloadStatus::Completed;
        m.last_status = s.status;

        if !self.started {
            return;
        }
        if is_new && self.prefs.show_progress_on_start && self.dialog.is_none() {
            self.selected = vec![s.id];
            self.anchor = Some(s.id);
            self.dialog = Some(Dialog::Properties(s.id));
        }
        if just_finished {
            self.status_note = Some(format!("Finished: {}", s.filename));
            let showing_this = matches!(self.dialog, Some(Dialog::Properties(id)) if id == s.id);
            if self.prefs.show_complete_dialog && (self.dialog.is_none() || showing_this) {
                self.complete_dont_show = false;
                self.dialog = Some(Dialog::Complete(s.id));
            }
            if self.prefs.open_folder_when_done {
                open_path(&s.destination, true);
            }
        }
    }

    fn refresh_torrent_view(&mut self, id: DownloadId) {
        self.torrent_view = self.queue_manager.torrent_details(id).map(|d| (id, d));
    }

    fn apply_theme(&mut self) {
        self.resolved_theme = self.prefs.theme.resolve();
        self.palette = Palette::new(self.resolved_theme, self.prefs.accent);
        self.iced_theme = self.palette.iced_theme(self.resolved_theme == ThemeMode::Dark);
    }

    pub(super) fn find(&self, id: DownloadId) -> Option<&DownloadSnapshot> {
        self.downloads.iter().find(|d| d.id == id)
    }

    pub(super) fn count(&self, f: Filter) -> usize {
        self.downloads.iter().filter(|d| f.matches(d)).count()
    }

    /// Rows shown in the list: filter, search, then sort.
    pub(super) fn visible(&self) -> Vec<&DownloadSnapshot> {
        let q = self.search.trim().to_lowercase();
        let mut rows: Vec<&DownloadSnapshot> = self
            .downloads
            .iter()
            .filter(|d| self.filter.matches(d))
            .filter(|d| q.is_empty() || d.filename.to_lowercase().contains(&q) || d.url.to_lowercase().contains(&q))
            .collect();

        let meta = &self.meta;
        let added = |d: &DownloadSnapshot| meta.get(&d.id).map(|m| m.added_at).unwrap_or(0);
        let tried = |d: &DownloadSnapshot| meta.get(&d.id).and_then(|m| m.last_try_at).unwrap_or(0);
        rows.sort_by(|a, b| {
            use std::cmp::Ordering;
            let o = match self.prefs.sort {
                Column::FileName => a.filename.to_lowercase().cmp(&b.filename.to_lowercase()),
                Column::Type => FileCategory::from_filename(&a.filename)
                    .label()
                    .cmp(FileCategory::from_filename(&b.filename).label()),
                Column::Size => a.total_bytes.cmp(&b.total_bytes),
                Column::Downloaded => a.downloaded_bytes.cmp(&b.downloaded_bytes),
                Column::Progress => a.progress_percent.partial_cmp(&b.progress_percent).unwrap_or(Ordering::Equal),
                Column::Status => (a.status as i32).cmp(&(b.status as i32)),
                Column::TimeLeft => a.eta_seconds.unwrap_or(u64::MAX).cmp(&b.eta_seconds.unwrap_or(u64::MAX)),
                Column::TransferRate => a
                    .speed_bytes_per_sec
                    .partial_cmp(&b.speed_bytes_per_sec)
                    .unwrap_or(Ordering::Equal),
                Column::Connections => a.segment_count.cmp(&b.segment_count),
                Column::LastTry => tried(a).cmp(&tried(b)),
                Column::Added => added(a).cmp(&added(b)),
                Column::SavePath => a.destination.cmp(&b.destination),
                Column::Url => a.url.cmp(&b.url),
            }
            .then(a.id.cmp(&b.id));
            if self.prefs.sort_descending {
                o.reverse()
            } else {
                o
            }
        });
        rows
    }

    pub(super) fn selected_snaps(&self) -> Vec<&DownloadSnapshot> {
        self.selected.iter().filter_map(|id| self.find(*id)).collect()
    }

    fn close_overlays(&mut self) {
        self.menu = None;
        self.context_menu = None;
    }

    fn set_note(&mut self, note: impl Into<String>) {
        self.status_note = Some(note.into());
    }

    fn default_folder_for(&self, filename: &str, subfolder: bool) -> String {
        let base = PathBuf::from(&self.settings.download_dir);
        if subfolder {
            let cat = self.add.category.unwrap_or_else(|| FileCategory::from_filename(filename));
            base.join(cat.label()).to_string_lossy().to_string()
        } else {
            base.to_string_lossy().to_string()
        }
    }

    fn refresh_add_suggestions(&mut self) {
        if !self.add.filename_edited {
            let is_torrent = crate::service::TorrentManager::is_torrent_url(&self.add.url);
            self.add.filename = if format::is_supported_link(&self.add.url) && !is_torrent {
                crate::service::network::NetworkClient::extract_filename_from_url(self.add.url.trim())
            } else {
                String::new()
            };
        }
        if !self.add.folder_edited {
            let name = self.add.filename.clone();
            self.add.folder = self.default_folder_for(&name, self.add.category_subfolder);
        }
    }

    // ---- update -----------------------------------------------------------------------

    pub fn update(&mut self, message: Message) -> Task<Message> {
        let task = self.update_inner(message);
        let dark = self.resolved_theme == ThemeMode::Dark;
        if self.title_bar_dark != Some(dark) {
            self.title_bar_dark = Some(dark);
            return Task::batch([task, sync_title_bar(dark)]);
        }
        task
    }

    fn update_inner(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                let snaps = self.queue_manager.get_all_snapshots();
                self.ingest(snaps);
                let tooltip = self.title();
                let actions = match self.tray.as_mut() {
                    Some(t) => {
                        t.set_tooltip(&tooltip);
                        t.poll()
                    }
                    None => Vec::new(),
                };
                if !actions.is_empty() {
                    let mut tasks = Vec::new();
                    for act in actions {
                        tasks.push(match act {
                            TrayAction::Show => self.update_inner(Message::ShowWindow),
                            TrayAction::AddUrl => Task::batch([
                                self.update_inner(Message::ShowWindow),
                                self.update_inner(Message::OpenAddDialog),
                            ]),
                            TrayAction::PauseAll => self.update_inner(Message::PauseAll),
                            TrayAction::ResumeAll => self.update_inner(Message::ResumeAll),
                            TrayAction::Exit => self.update_inner(Message::Exit),
                        });
                    }
                    return Task::batch(tasks);
                }
                // Follow Windows light/dark changes (checked every ~5 s).
                self.ticks = self.ticks.wrapping_add(1);
                if self.prefs.theme == ThemeMode::System && self.ticks % 20 == 0 {
                    self.apply_theme();
                }
                if let Some(Dialog::TorrentDetails(id)) = self.dialog {
                    if self.ticks % 4 == 0 {
                        self.refresh_torrent_view(id);
                    }
                }
                // Clipboard monitor: check about once a second while no dialog is open.
                if self.prefs.monitor_clipboard && self.ticks % 4 == 0 && self.dialog.is_none() {
                    return iced::clipboard::read().map(Message::ClipboardPoll);
                }
            }
            Message::ClipboardPoll(clip) => {
                let link = clip.as_deref().map(format::extract_links).and_then(|l| l.into_iter().next());
                if !self.clip_baseline {
                    // Whatever was on the clipboard when monitoring started isn't "new".
                    self.clip_baseline = true;
                    self.last_clip = link;
                } else if link.is_some() && link != self.last_clip {
                    self.last_clip = link.clone();
                    let known = link.as_ref().is_some_and(|l| self.downloads.iter().any(|d| &d.url == l));
                    if !known && self.dialog.is_none() {
                        return self.update_inner(Message::OpenAddDialog);
                    }
                }
            }
            Message::SnapshotReceived(snap) => {
                self.note_status(&snap, now());
                match self.downloads.iter().position(|d| d.id == snap.id) {
                    Some(i) => self.downloads[i] = snap,
                    None => self.downloads.push(snap),
                }
            }
            Message::CursorMoved(p) => self.cursor = p,
            Message::ModifiersChanged(m) => self.modifiers = m,
            Message::KeyPressed(key, mods) => return self.on_key(key, mods),

            Message::SelectFilter(f) => {
                self.filter = f;
                self.close_overlays();
            }
            Message::ToggleAllExpanded => self.all_expanded = !self.all_expanded,
            Message::SearchChanged(q) => self.search = q,
            Message::FocusSearch => {
                self.close_overlays();
                return text_input::focus(search_input_id());
            }
            Message::RowPressed(id) => {
                self.close_overlays();
                self.select_row(id);
                let is_double = matches!(self.last_click, Some((last, at)) if last == id && at.elapsed() < DOUBLE_CLICK);
                if is_double && !self.modifiers.control() && !self.modifiers.shift() {
                    self.last_click = None;
                    if self.queue_manager.is_torrent(id) {
                        return self.update_inner(Message::OpenTorrentDetails(id, TorrentTab::General));
                    }
                    self.dialog = Some(Dialog::Properties(id));
                } else {
                    self.last_click = Some((id, Instant::now()));
                }
            }
            Message::SelectAll => {
                self.close_overlays();
                self.selected = self.visible().iter().map(|d| d.id).collect();
            }
            Message::RowRightPressed(id) => {
                self.menu = None;
                if !self.selected.contains(&id) {
                    self.selected = vec![id];
                    self.anchor = Some(id);
                }
                self.context_menu = Some(self.cursor);
            }
            Message::SortBy(col) => {
                if self.prefs.sort == col {
                    self.prefs.sort_descending = !self.prefs.sort_descending;
                } else {
                    self.prefs.sort = col;
                    self.prefs.sort_descending = matches!(
                        col,
                        Column::Size | Column::Downloaded | Column::Progress | Column::TransferRate | Column::LastTry | Column::Added
                    );
                }
                self.prefs.save();
            }

            Message::OpenMenu(kind) => {
                self.context_menu = None;
                self.menu = if self.menu == Some(kind) { None } else { Some(kind) };
            }
            Message::CloseMenus => self.close_overlays(),

            Message::ResumeSelected => {
                self.close_overlays();
                let ids: Vec<DownloadId> = self
                    .selected_snaps()
                    .iter()
                    .filter(|d| d.status != DownloadStatus::Completed && d.status != DownloadStatus::Downloading)
                    .map(|d| d.id)
                    .collect();
                for id in &ids {
                    self.queue_manager.resume_download(*id);
                }
                self.set_note(format!("Resumed {} download(s)", ids.len()));
            }
            Message::PauseSelected => {
                self.close_overlays();
                let ids: Vec<DownloadId> = self
                    .selected_snaps()
                    .iter()
                    .filter(|d| d.status == DownloadStatus::Downloading)
                    .map(|d| d.id)
                    .collect();
                for id in &ids {
                    self.queue_manager.pause_download(*id);
                }
                self.set_note(format!("Paused {} download(s)", ids.len()));
            }
            Message::ResumeAll => {
                self.close_overlays();
                self.queue_manager.resume_all();
                self.set_note("Resumed all unfinished downloads");
            }
            Message::PauseAll => {
                self.close_overlays();
                self.queue_manager.pause_all();
                self.set_note("Paused all downloads");
            }
            Message::RequestDeleteSelected => {
                self.close_overlays();
                if !self.selected.is_empty() {
                    self.delete_with_file = false;
                    self.delete_dont_ask = false;
                    self.dialog = Some(Dialog::ConfirmDelete(self.selected.clone()));
                    if !self.prefs.confirm_remove {
                        return self.update(Message::ConfirmDelete);
                    }
                }
            }
            Message::RequestDeleteCompleted => {
                self.close_overlays();
                let ids: Vec<DownloadId> = self
                    .downloads
                    .iter()
                    .filter(|d| d.status == DownloadStatus::Completed)
                    .map(|d| d.id)
                    .collect();
                if ids.is_empty() {
                    self.set_note("No completed downloads to remove");
                } else {
                    self.delete_with_file = false;
                    self.dialog = Some(Dialog::ConfirmDelete(ids));
                }
            }
            Message::DeleteWithFile(b) => self.delete_with_file = b,
            Message::ConfirmDelete => {
                if self.delete_dont_ask {
                    self.prefs.confirm_remove = false;
                    self.prefs.save();
                }
                if let Some(Dialog::ConfirmDelete(ids)) = self.dialog.take() {
                    for id in &ids {
                        self.queue_manager.delete_download(*id, self.delete_with_file);
                    }
                    self.downloads.retain(|d| !ids.contains(&d.id));
                    self.meta.retain(|id, _| !ids.contains(id));
                    self.selected.retain(|id| !ids.contains(id));
                    self.set_note(format!(
                        "Removed {} item(s){}",
                        ids.len(),
                        if self.delete_with_file { " and their files" } else { "" }
                    ));
                }
            }
            Message::OpenFile => {
                self.close_overlays();
                if let Some(Dialog::Complete(id) | Dialog::Properties(id)) = self.dialog {
                    self.selected = vec![id];
                }
                if let Some(d) = self.selected_snaps().first() {
                    let path = d.destination.clone();
                    if d.status == DownloadStatus::Completed && std::path::Path::new(&path).exists() {
                        open_path(&path, false);
                    } else {
                        self.set_note("The file isn't finished yet");
                    }
                }
            }
            Message::OpenFolder => {
                self.close_overlays();
                if let Some(Dialog::Complete(id) | Dialog::Properties(id)) = self.dialog {
                    self.selected = vec![id];
                }
                if let Some(d) = self.selected_snaps().first() {
                    let path = d.destination.clone();
                    open_path(&path, true);
                }
            }
            Message::CopyUrls => {
                self.close_overlays();
                let urls: Vec<String> = self.selected_snaps().iter().map(|d| d.url.clone()).collect();
                if !urls.is_empty() {
                    self.set_note(format!("Copied {} address(es)", urls.len()));
                    return iced::clipboard::write(urls.join("\n"));
                }
            }
            Message::PauseOne(id) => self.queue_manager.pause_download(id),
            Message::ResumeOne(id) => self.queue_manager.resume_download(id),
            Message::RemoveOne(id) => {
                self.delete_with_file = false;
                self.delete_dont_ask = false;
                self.dialog = Some(Dialog::ConfirmDelete(vec![id]));
            }
            Message::ShowProperties => {
                self.close_overlays();
                if let Some(id) = self.selected.first().copied() {
                    if self.queue_manager.is_torrent(id) {
                        return self.update_inner(Message::OpenTorrentDetails(id, TorrentTab::General));
                    }
                    self.dialog = Some(Dialog::Properties(id));
                }
            }

            Message::ToggleColumn(c) => {
                self.prefs.toggle_column(c);
                self.prefs.save();
            }
            Message::ToggleCategories => {
                self.prefs.show_categories = !self.prefs.show_categories;
                self.prefs.save();
                self.close_overlays();
            }
            Message::ToggleDetails => {
                self.prefs.show_details = !self.prefs.show_details;
                self.prefs.save();
                self.close_overlays();
            }
            Message::SetTheme(mode) => {
                self.prefs.theme = mode;
                self.apply_theme();
                self.prefs.save();
                self.close_overlays();
            }
            Message::SetDetailsTab(t) => self.details_tab = t,

            Message::OpenAddDialog => {
                self.close_overlays();
                self.add = AddForm {
                    start_now: true,
                    category_subfolder: self.prefs.category_subfolders,
                    ..AddForm::default()
                };
                self.refresh_add_suggestions();
                self.dialog = Some(Dialog::AddUrl);
                // Like most download managers: prefill from the clipboard when it holds a link.
                return Task::batch([
                    iced::clipboard::read().map(Message::ClipboardForAdd),
                    text_input::focus(add_url_input_id()),
                ]);
            }
            Message::ClipboardForAdd(clip) => {
                if self.dialog == Some(Dialog::AddUrl) && self.add.url.trim().is_empty() {
                    if let Some(link) = clip.as_deref().map(format::extract_links).and_then(|l| l.into_iter().next()) {
                        self.add.url = link;
                        self.refresh_add_suggestions();
                    }
                }
            }
            Message::PasteIntoAdd => {
                return iced::clipboard::read().map(|clip| {
                    Message::AddUrlChanged(
                        clip.as_deref()
                            .map(format::extract_links)
                            .and_then(|l| l.into_iter().next())
                            .unwrap_or_default(),
                    )
                });
            }
            Message::AddUrlChanged(u) => {
                self.add.url = u;
                self.add.error = None;
                self.refresh_add_suggestions();
            }
            Message::AddFilenameChanged(f) => {
                self.add.filename = f;
                self.add.filename_edited = true;
                if !self.add.folder_edited {
                    let name = self.add.filename.clone();
                    self.add.folder = self.default_folder_for(&name, self.add.category_subfolder);
                }
            }
            Message::AddFolderChanged(f) => {
                self.add.folder = f;
                self.add.folder_edited = true;
            }
            Message::AddStartNow(b) => self.add.start_now = b,
            Message::AddCategoryPicked(c) => {
                self.add.category = Some(c);
                self.add.category_subfolder = true;
                self.add.folder_edited = false;
                self.refresh_add_suggestions();
            }
            Message::AddLater => {
                self.add.start_now = false;
                let t = self.update_inner(Message::SubmitAdd);
                self.add.start_now = true;
                return t;
            }
            Message::ToggleProgressDetails => {
                self.prefs.progress_details = !self.prefs.progress_details;
                self.prefs.save();
            }
            Message::DeleteDontAsk(b) => self.delete_dont_ask = b,
            Message::CompleteDontShow(b) => self.complete_dont_show = b,
            Message::AddCategorySubfolder(b) => {
                self.add.category_subfolder = b;
                self.add.folder_edited = false;
                self.refresh_add_suggestions();
            }
            Message::SubmitAdd => {
                let url = self.add.url.trim().to_string();
                if !format::is_supported_link(&url) {
                    self.add.error = Some("Enter an http(s):// address or a magnet: link.".into());
                    return Task::none();
                }
                let name = self.add.filename.trim().to_string();
                if name.contains(['/', '\\', ':', '*', '?', '"', '<', '>', '|']) {
                    self.add.error = Some("The file name can't contain \\ / : * ? \" < > |".into());
                    return Task::none();
                }
                let folder = self.add.folder.trim().to_string();
                if folder.is_empty() {
                    self.add.error = Some("Choose a folder to save to.".into());
                    return Task::none();
                }
                let dupe = self.downloads.iter().any(|d| d.url == url);
                let qm = Arc::clone(&self.queue_manager);
                let start = self.add.start_now;
                let override_name = if name.is_empty() { None } else { Some(name) };
                let url_for_task = url.clone();
                crate::service::spawn(async move {
                    if let Err(e) = qm
                        .add_download_as(&url_for_task, Some(PathBuf::from(folder)), override_name, start)
                        .await
                    {
                        tracing::warn!("Adding download {} failed: {}", url_for_task, e);
                    }
                });
                self.dialog = None;
                self.set_note(if dupe {
                    "Added (this address was already in the list)".to_string()
                } else if start {
                    "Download started".to_string()
                } else {
                    "Added to the list".to_string()
                });
            }

            Message::OpenBatchDialog => {
                self.close_overlays();
                self.batch = text_editor::Content::new();
                self.dialog = Some(Dialog::Batch);
            }
            Message::BatchFromClipboard => {
                self.close_overlays();
                return iced::clipboard::read().map(Message::ClipboardForBatch);
            }
            Message::ClipboardForBatch(clip) => {
                let links = clip.as_deref().map(format::extract_links).unwrap_or_default();
                if links.is_empty() {
                    self.set_note("The clipboard has no http(s) or magnet links");
                } else {
                    self.batch = text_editor::Content::with_text(&links.join("\n"));
                    self.dialog = Some(Dialog::Batch);
                }
            }
            Message::BatchAction(action) => self.batch.perform(action),
            Message::BatchStartNow(b) => self.batch_start_now = b,
            Message::SubmitBatch => {
                let links = format::extract_links(&self.batch.text());
                if links.is_empty() {
                    return Task::none();
                }
                let qm = Arc::clone(&self.queue_manager);
                let start = self.batch_start_now;
                let n = links.len();
                crate::service::spawn(async move {
                    for link in links {
                        if let Err(e) = qm.add_download(&link, None, start).await {
                            tracing::warn!("Adding download {} failed: {}", link, e);
                        }
                    }
                });
                self.dialog = None;
                self.set_note(format!("Added {} download(s)", n));
            }

            Message::OpenOptions => {
                self.close_overlays();
                let s = &self.settings;
                self.options = OptionsDraft {
                    download_dir: s.download_dir.clone(),
                    max_concurrent: s.max_concurrent_downloads.to_string(),
                    max_segments: s.max_segments.to_string(),
                    max_retries: s.max_retries.to_string(),
                    speed_limit: s.speed_limit_k_bps.to_string(),
                    theme: self.prefs.theme,
                    accent: self.prefs.accent,
                    category_subfolders: self.prefs.category_subfolders,
                    show_progress_on_start: self.prefs.show_progress_on_start,
                    open_folder_when_done: self.prefs.open_folder_when_done,
                    confirm_remove: self.prefs.confirm_remove,
                    start_with_windows: startup::is_enabled(),
                    close_to_tray: self.prefs.close_to_tray,
                    start_minimized: self.prefs.start_minimized,
                    show_complete_dialog: self.prefs.show_complete_dialog,
                    monitor_clipboard: self.prefs.monitor_clipboard,
                    error: None,
                };
                self.torrent_draft = TorrentDraft::from_settings(self.queue_manager.torrent_settings());
                self.dialog = Some(Dialog::Options);
            }
            Message::OptTorrent(m) => self.torrent_draft.apply(m),
            Message::OpenTorrentSettings => {
                let t = self.update_inner(Message::OpenOptions);
                self.options_tab = OptionsTab::Torrent;
                return t;
            }
            Message::OpenAppearance => {
                let t = self.update_inner(Message::OpenOptions);
                self.options_tab = OptionsTab::Appearance;
                return t;
            }
            Message::SetOptionsTab(t) => self.options_tab = t,
            Message::OptDownloadDir(v) => self.options.download_dir = v,
            Message::OptConcurrent(v) => self.options.max_concurrent = v,
            Message::OptSegments(v) => self.options.max_segments = v,
            Message::OptRetries(v) => self.options.max_retries = v,
            Message::OptSpeedLimit(v) => self.options.speed_limit = v,
            Message::OptTheme(t) => self.options.theme = t,
            Message::OptAccent(c) => self.options.accent = c,
            Message::SetAccent(c) => {
                self.prefs.accent = c;
                self.apply_theme();
                self.prefs.save();
                self.close_overlays();
            }
            Message::OptCategorySubfolders(b) => self.options.category_subfolders = b,
            Message::OptShowProgress(b) => self.options.show_progress_on_start = b,
            Message::OptOpenFolderDone(b) => self.options.open_folder_when_done = b,
            Message::OptConfirmRemove(b) => self.options.confirm_remove = b,
            Message::OptStartWithWindows(b) => self.options.start_with_windows = b,
            Message::OptCloseToTray(b) => self.options.close_to_tray = b,
            Message::OptStartMinimized(b) => self.options.start_minimized = b,
            Message::OptShowComplete(b) => self.options.show_complete_dialog = b,
            Message::OptMonitorClipboard(b) => self.options.monitor_clipboard = b,
            Message::SaveOptions => match self.options_to_settings().and_then(|s| self.torrent_draft.to_settings().map(|t| (s, t))) {
                Ok((new_settings, torrent_settings)) => {
                    if let Err(e) = self.queue_manager.update_torrent_settings(torrent_settings) {
                        self.options.error = Some(format!("Couldn't save torrent settings: {}", e));
                        return Task::none();
                    }
                    let to_save = new_settings.clone();
                    if let Err(e) = Config::load().update(|s| *s = to_save) {
                        self.options.error = Some(format!("Couldn't save settings: {}", e));
                        return Task::none();
                    }
                    self.queue_manager.update_settings(new_settings.clone());
                    self.settings = new_settings;
                    self.prefs.theme = self.options.theme;
                    self.prefs.accent = self.options.accent;
                    self.prefs.category_subfolders = self.options.category_subfolders;
                    self.prefs.show_progress_on_start = self.options.show_progress_on_start;
                    self.prefs.open_folder_when_done = self.options.open_folder_when_done;
                    self.prefs.confirm_remove = self.options.confirm_remove;
                    self.prefs.show_complete_dialog = self.options.show_complete_dialog;
                    self.prefs.close_to_tray = self.options.close_to_tray;
                    self.prefs.start_minimized = self.options.start_minimized;
                    if self.options.monitor_clipboard && !self.prefs.monitor_clipboard {
                        self.clip_baseline = false;
                    }
                    self.prefs.monitor_clipboard = self.options.monitor_clipboard;
                    if self.options.start_with_windows != startup::is_enabled() {
                        if let Err(e) = startup::set_enabled(self.options.start_with_windows) {
                            self.options.error = Some(format!("Couldn't change the startup setting: {}", e));
                            return Task::none();
                        }
                    }
                    self.apply_theme();
                    self.prefs.save();
                    self.dialog = None;
                    self.set_note("Options saved");
                }
                Err(e) => self.options.error = Some(e),
            },

            Message::OpenTorrentDetails(id, tab) => {
                self.close_overlays();
                self.selected = vec![id];
                self.torrent_tab = tab;
                self.tracker_input.clear();
                self.refresh_torrent_view(id);
                self.dialog = Some(Dialog::TorrentDetails(id));
            }
            Message::TorrentTab(t) => self.torrent_tab = t,
            Message::TorrentFilePriority(index, prio) => {
                if let Some(Dialog::TorrentDetails(id)) = self.dialog {
                    self.queue_manager.set_torrent_file_priority(id, index, prio.value());
                    self.refresh_torrent_view(id);
                }
            }
            Message::TorrentAllFiles(on) => {
                if let Some(Dialog::TorrentDetails(id)) = self.dialog {
                    let files: Vec<i32> = self
                        .torrent_view
                        .as_ref()
                        .map(|(_, d)| d.files.iter().map(|f| f.index).collect())
                        .unwrap_or_default();
                    for i in files {
                        self.queue_manager.set_torrent_file_priority(id, i, if on { 4 } else { 0 });
                    }
                    self.refresh_torrent_view(id);
                }
            }
            Message::TorrentSequential(id, on) => {
                self.close_overlays();
                self.queue_manager.set_torrent_sequential(id, on);
                self.refresh_torrent_view(id);
                self.set_note(if on { "Downloading pieces in order" } else { "Downloading rarest pieces first" });
            }
            Message::TorrentRecheck(id) => {
                self.close_overlays();
                self.queue_manager.force_torrent_recheck(id);
                self.set_note("Checking downloaded data…");
            }
            Message::TorrentReannounce(id) => {
                self.close_overlays();
                self.queue_manager.force_torrent_reannounce(id);
                self.set_note("Contacting trackers…");
            }
            Message::TrackerInput(v) => self.tracker_input = v,
            Message::TrackerAdd => {
                let url = self.tracker_input.trim().to_string();
                let valid = ["udp://", "http://", "https://", "wss://"].iter().any(|s| url.starts_with(s));
                if let (true, Some(Dialog::TorrentDetails(id))) = (valid, self.dialog.clone()) {
                    self.queue_manager.add_torrent_tracker(id, &url);
                    self.tracker_input.clear();
                    self.refresh_torrent_view(id);
                    self.set_note("Tracker added");
                } else if !valid {
                    self.set_note("A tracker address starts with udp://, http://, https:// or wss://");
                }
            }
            Message::OpenCreateTorrent => {
                self.close_overlays();
                self.create_form = CreateTorrentForm::default();
                self.dialog = Some(Dialog::CreateTorrent);
            }
            Message::CtSource(v) => {
                if !self.create_form.output_edited {
                    let base = v.trim().trim_end_matches(['\\', '/']);
                    self.create_form.output = if base.is_empty() { String::new() } else { format!("{}.torrent", base) };
                }
                self.create_form.source = v;
                self.create_form.error = None;
            }
            Message::CtOutput(v) => {
                self.create_form.output = v;
                self.create_form.output_edited = true;
            }
            Message::CtTrackers(act) => self.create_form.trackers.perform(act),
            Message::CtWebSeeds(v) => self.create_form.web_seeds = v,
            Message::CtComment(v) => self.create_form.comment = v,
            Message::CtPiece(p) => self.create_form.piece = p,
            Message::CtPrivate(b) => self.create_form.private = b,
            Message::CtSeed(b) => self.create_form.seed = b,
            Message::CtSubmit => {
                let c = &mut self.create_form;
                let source = PathBuf::from(c.source.trim());
                let output = PathBuf::from(c.output.trim());
                if !source.exists() {
                    c.error = Some("That file or folder doesn't exist.".into());
                    return Task::none();
                }
                c.busy = true;
                c.error = None;
                c.done = None;
                let split = |s: &str| s.lines().map(str::trim).filter(|l| !l.is_empty()).map(str::to_string).collect::<Vec<_>>();
                let trackers = split(&c.trackers.text());
                let web_seeds: Vec<String> = c.web_seeds.split_whitespace().map(str::to_string).collect();
                let comment = c.comment.clone();
                let piece = c.piece.kib();
                let private = c.private;
                return Task::perform(
                    async move {
                        let out = output.clone();
                        let job = crate::service::get_runtime().spawn_blocking(move || {
                            crate::service::TorrentManager::create_torrent_file(
                                &source, &out, &trackers, &web_seeds, &comment, piece, private,
                            )
                        });
                        match job.await {
                            Ok(Ok(())) => Ok(output.to_string_lossy().into_owned()),
                            Ok(Err(e)) => Err(e),
                            Err(e) => Err(e.to_string()),
                        }
                    },
                    Message::CtDone,
                );
            }
            Message::CtDone(result) => {
                self.create_form.busy = false;
                match result {
                    Ok(path) => {
                        self.create_form.done = Some(path.clone());
                        self.set_note(format!("Created {}", path));
                        if self.create_form.seed {
                            let folder = PathBuf::from(self.create_form.source.trim())
                                .parent()
                                .map(|p| p.to_path_buf());
                            let qm = Arc::clone(&self.queue_manager);
                            crate::service::spawn(async move {
                                if let Err(e) = qm.add_download_as(&path, folder, None, true).await {
                                    tracing::warn!("Seeding the new torrent failed: {}", e);
                                }
                            });
                        }
                    }
                    Err(e) => self.create_form.error = Some(e),
                }
            }
            Message::OpenAbout => {
                self.close_overlays();
                self.dialog = Some(Dialog::About);
            }
            Message::CloseDialog => {
                if matches!(self.dialog, Some(Dialog::Complete(_))) && self.complete_dont_show {
                    self.prefs.show_complete_dialog = false;
                    self.prefs.save();
                }
                self.dialog = None;
                self.close_overlays();
            }
            Message::WindowCloseRequested(id) => {
                tracing::info!("Close requested for window {:?} (close to tray: {})", id, self.prefs.close_to_tray);
                if self.prefs.close_to_tray && self.tray.is_some() {
                    self.close_overlays();
                    self.window_hidden = true;
                    return set_window_hidden(true);
                }
                return self.update_inner(Message::Exit);
            }
            Message::ShowWindow => {
                self.window_hidden = false;
                return set_window_hidden(false);
            }
            Message::Exit => {
                // Keep torrents' progress for the next start.
                self.queue_manager.save_torrent_state();
                return iced::exit();
            }
        }
        Task::none()
    }

    fn options_to_settings(&self) -> Result<Settings, String> {
        fn parse(label: &str, v: &str, min: u32, max: u32) -> Result<u32, String> {
            match v.trim().parse::<u32>() {
                Ok(n) if (min..=max).contains(&n) => Ok(n),
                _ => Err(format!("{} must be a whole number from {} to {}.", label, min, max)),
            }
        }
        let o = &self.options;
        let dir = o.download_dir.trim();
        if dir.is_empty() {
            return Err("Choose a default download folder.".into());
        }
        let speed = match o.speed_limit.trim() {
            "" => 0,
            v => v
                .parse::<i64>()
                .ok()
                .filter(|n| *n >= 0)
                .ok_or_else(|| "Speed limit must be 0 (unlimited) or a positive number of KB/s.".to_string())?,
        };
        let mut s = self.settings.clone();
        s.download_dir = dir.to_string();
        s.max_concurrent_downloads =
            parse("Simultaneous downloads", &o.max_concurrent, MIN_CONCURRENT_DOWNLOADS, MAX_CONCURRENT_DOWNLOADS)?;
        s.max_segments = parse("Connections per download", &o.max_segments, MIN_SEGMENTS, MAX_SEGMENTS)?;
        s.max_retries = parse("Retries", &o.max_retries, MIN_RETRIES, MAX_RETRIES_LIMIT)?;
        s.speed_limit_k_bps = speed;
        Ok(s)
    }

    /// Click selection: plain click selects one, Ctrl toggles, Shift selects a range.
    fn select_row(&mut self, id: DownloadId) {
        if self.modifiers.shift() {
            if let Some(anchor) = self.anchor {
                let order: Vec<DownloadId> = self.visible().iter().map(|d| d.id).collect();
                if let (Some(a), Some(b)) = (order.iter().position(|x| *x == anchor), order.iter().position(|x| *x == id)) {
                    let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
                    self.selected = order[lo..=hi].to_vec();
                    return;
                }
            }
        }
        if self.modifiers.control() {
            if let Some(pos) = self.selected.iter().position(|x| *x == id) {
                self.selected.remove(pos);
            } else {
                self.selected.push(id);
            }
        } else {
            self.selected = vec![id];
        }
        self.anchor = Some(id);
    }

    fn on_key(&mut self, key: Key, mods: Modifiers) -> Task<Message> {
        if let Key::Named(Named::Escape) = key {
            if self.menu.is_some() || self.context_menu.is_some() {
                self.close_overlays();
            } else {
                self.dialog = None;
            }
            return Task::none();
        }
        if let Key::Named(Named::Enter) = key {
            return match self.dialog {
                Some(Dialog::ConfirmDelete(_)) => self.update(Message::ConfirmDelete),
                None if !self.selected.is_empty() => self.update(Message::ShowProperties),
                _ => Task::none(),
            };
        }
        if self.dialog.is_some() {
            return Task::none();
        }
        match key.as_ref() {
            Key::Named(Named::Delete) => self.update(Message::RequestDeleteSelected),
            Key::Character(c) if mods.command() => match c.to_ascii_lowercase().as_str() {
                "n" => self.update(Message::OpenAddDialog),
                "b" => self.update(Message::OpenBatchDialog),
                "f" => self.update(Message::FocusSearch),
                "o" => self.update(Message::OpenOptions),
                "p" => self.update(Message::PauseSelected),
                "r" => self.update(Message::ResumeSelected),
                "a" => self.update(Message::SelectAll),
                _ => Task::none(),
            },
            _ => Task::none(),
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let sub = self.queue_manager.subscribe();
        Subscription::batch([
            iced::time::every(Duration::from_millis(crate::config::system::UI_TICK_INTERVAL_MS)).map(|_| Message::Tick),
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
            event::listen_with(on_event),
            window::close_requests().map(Message::WindowCloseRequested),
        ])
    }
}

fn on_event(event: Event, status: event::Status, _window: window::Id) -> Option<Message> {
    match event {
        Event::Mouse(mouse::Event::CursorMoved { position }) => Some(Message::CursorMoved(position)),
        Event::Keyboard(keyboard::Event::ModifiersChanged(m)) => Some(Message::ModifiersChanged(m)),
        Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => {
            let is_escape = matches!(key, Key::Named(Named::Escape));
            // Keys a text field consumed (typing, Delete inside an input) aren't shortcuts.
            if status == event::Status::Captured && !is_escape {
                None
            } else {
                Some(Message::KeyPressed(key, modifiers))
            }
        }
        _ => None,
    }
}

/// Hides the window (it stays in the tray) or brings it back to the front.
fn set_window_hidden(hidden: bool) -> Task<Message> {
    iced::window::get_latest().and_then(move |id| {
        if hidden {
            window::change_mode(id, window::Mode::Hidden)
        } else {
            Task::batch([window::change_mode(id, window::Mode::Windowed), window::gain_focus(id)])
        }
    })
}

/// Makes the native title bar dark or light to match the app (Windows 10 1809+ / 11).
fn sync_title_bar(dark: bool) -> Task<Message> {
    iced::window::get_latest().and_then(move |id| {
        iced::window::run_with_handle(id, move |handle| set_title_bar_dark(handle, dark)).discard()
    })
}

#[cfg(windows)]
fn set_title_bar_dark(handle: iced::window::raw_window_handle::WindowHandle<'_>, dark: bool) {
    use iced::window::raw_window_handle::RawWindowHandle;
    #[link(name = "dwmapi")]
    extern "system" {
        fn DwmSetWindowAttribute(hwnd: isize, attribute: u32, value: *const core::ffi::c_void, size: u32) -> i32;
    }
    const DWMWA_USE_IMMERSIVE_DARK_MODE: u32 = 20;
    if let RawWindowHandle::Win32(w) = handle.as_raw() {
        let value: i32 = dark as i32;
        // SAFETY: hwnd comes from the live window and `value` outlives the call.
        unsafe {
            DwmSetWindowAttribute(
                w.hwnd.get(),
                DWMWA_USE_IMMERSIVE_DARK_MODE,
                &value as *const i32 as *const core::ffi::c_void,
                std::mem::size_of::<i32>() as u32,
            );
        }
    }
}

#[cfg(not(windows))]
fn set_title_bar_dark(_handle: iced::window::raw_window_handle::WindowHandle<'_>, _dark: bool) {}

/// "Start with Windows": a value under HKCU\...\Run pointing at this exe.
mod startup {
    const RUN_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
    const VALUE: &str = "1nternet Downloader";

    #[cfg(windows)]
    fn reg(args: &[&str]) -> std::io::Result<std::process::Output> {
        use std::os::windows::process::CommandExt;
        std::process::Command::new("reg").args(args).creation_flags(0x0800_0000).output()
    }

    #[cfg(windows)]
    pub fn is_enabled() -> bool {
        reg(&["query", RUN_KEY, "/v", VALUE]).map(|o| o.status.success()).unwrap_or(false)
    }

    #[cfg(windows)]
    pub fn set_enabled(on: bool) -> Result<(), String> {
        let out = if on {
            let exe = std::env::current_exe().map_err(|e| e.to_string())?;
            let data = format!("\"{}\"", exe.display());
            reg(&["add", RUN_KEY, "/v", VALUE, "/t", "REG_SZ", "/d", &data, "/f"])
        } else {
            reg(&["delete", RUN_KEY, "/v", VALUE, "/f"])
        }
        .map_err(|e| e.to_string())?;
        if out.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
        }
    }

    #[cfg(not(windows))]
    pub fn is_enabled() -> bool {
        false
    }

    #[cfg(not(windows))]
    pub fn set_enabled(_on: bool) -> Result<(), String> {
        Err("only supported on Windows".into())
    }
}

/// Opens a file with its default app, or shows it selected in Explorer.
fn open_path(path: &str, reveal: bool) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let mut cmd = std::process::Command::new("explorer.exe");
        if reveal {
            cmd.raw_arg(format!("/select,\"{}\"", path));
        } else {
            cmd.raw_arg(format!("\"{}\"", path));
        }
        if let Err(e) = cmd.spawn() {
            tracing::warn!("Couldn't open {}: {}", path, e);
        }
    }
    #[cfg(not(windows))]
    {
        let _ = (path, reveal);
    }
}
