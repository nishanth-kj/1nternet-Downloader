//! Modal dialogs shown over the main window.

use crate::config::system::{
    APP_VERSION, MAX_CONCURRENT_DOWNLOADS, MAX_RETRIES_LIMIT, MAX_SEGMENTS, MIN_CONCURRENT_DOWNLOADS, MIN_RETRIES,
    MIN_SEGMENTS,
};
use crate::model::DownloadStatus;
use crate::ui::app::{add_url_input_id, Dialog, IcedApp, Message, OptionsTab};
use crate::ui::format::{self, FileCategory};
use crate::ui::theme::{Accent, ThemeMode};
use crate::ui::view::{flat_button, icon, input_style, primary_button, secondary_button, status_label};
use iced::widget::text::Wrapping;
use iced::widget::{
    button, checkbox, column, container, horizontal_rule, horizontal_space, pick_list, row, scrollable, text,
    text_editor, text_input,
};
use iced::{Alignment, Background, Border, Color, Element, Length};
#[allow(unused_imports)]
use iced::widget::Space;

impl IcedApp {
    pub(super) fn view_dialog<'a>(&'a self, dialog: &'a Dialog) -> Element<'a, Message> {
        match dialog {
            Dialog::AddUrl => self.dialog_add(),
            Dialog::Batch => self.dialog_batch(),
            Dialog::Options => self.dialog_options(),
            Dialog::ConfirmDelete(ids) => self.dialog_delete(ids),
            Dialog::Properties(id) => self.dialog_properties(*id),
            Dialog::Complete(id) => self.dialog_complete(*id),
            Dialog::TorrentDetails(id) => self.dialog_torrent_details(*id),
            Dialog::CreateTorrent => self.dialog_create_torrent(),
            Dialog::About => self.dialog_about(),
        }
    }

    pub(super) fn dialog_frame<'a>(
        &self,
        title: String,
        width: f32,
        body: Element<'a, Message>,
        footer: Element<'a, Message>,
    ) -> Element<'a, Message> {
        let p = self.palette;
        let title_bar = container(
            row![
                text(title).size(13),
                horizontal_space(),
                button(icon("\u{E8BB}", 11.0, p.text)).padding([2, 8]).on_press(Message::CloseDialog).style(flat_button(p, false)),
            ]
            .align_y(Alignment::Center),
        )
        .padding([6, 12])
        .style(move |_t| container::Style {
            background: Some(Background::Color(p.toolbar)),
            text_color: Some(p.text),
            ..Default::default()
        });

        container(
            column![
                title_bar,
                horizontal_rule(1),
                container(body).padding([14, 16]),
                horizontal_rule(1),
                container(footer).padding([10, 16]),
            ]
            .width(Length::Fixed(width)),
        )
        .style(move |_t| container::Style {
            background: Some(Background::Color(p.background)),
            text_color: Some(p.text),
            border: Border { color: p.border, width: 1.0, radius: 6.0.into() },
            shadow: iced::Shadow {
                color: Color { a: 0.35, ..Color::BLACK },
                offset: iced::Vector::new(0.0, 6.0),
                blur_radius: 24.0,
            },
        })
        .into()
    }

    fn label<'a>(&self, s: &'a str) -> Element<'a, Message> {
        text(s).size(11.5).color(self.palette.muted).width(Length::Fixed(120.0)).into()
    }

    fn footer<'a>(&self, primary: &'a str, msg: Option<Message>, danger: bool) -> Element<'a, Message> {
        let p = self.palette;
        let mut primary_btn = button(text(primary).size(12)).padding([6, 18]).on_press_maybe(msg);
        primary_btn = if danger {
            primary_btn.style(move |_t, status| button::Style {
                background: Some(Background::Color(if status == button::Status::Hovered {
                    Color { a: 0.88, ..p.danger }
                } else {
                    p.danger
                })),
                text_color: Color::WHITE,
                border: iced::border::rounded(3),
                ..Default::default()
            })
        } else {
            primary_btn.style(primary_button(p))
        };
        row![
            horizontal_space(),
            button(text("Cancel").size(12)).padding([6, 16]).on_press(Message::CloseDialog).style(secondary_button(p)),
            primary_btn,
        ]
        .spacing(8)
        .into()
    }

    fn dialog_add(&self) -> Element<'_, Message> {
        let p = self.palette;
        let a = &self.add;
        let url_ok = format::is_supported_link(&a.url);
        let category = FileCategory::from_filename(&a.filename);
        let dupe = url_ok && self.downloads.iter().any(|d| d.url == a.url.trim());

        let hint: Element<'_, Message> = if a.url.trim().is_empty() {
            text("Paste an http(s) address or a magnet link.").size(10.5).color(p.muted).into()
        } else if !url_ok {
            text("That doesn't look like an http(s) or magnet link.").size(10.5).color(p.danger).into()
        } else if dupe {
            text("This address is already in your list. Adding it again creates a second copy.")
                .size(10.5)
                .color(p.warning)
                .into()
        } else {
            row![icon(category.glyph(), 11.0, p.muted), text(format!("Category: {}", category.label())).size(10.5).color(p.muted)]
                .spacing(6)
                .into()
        };

        let mut body = column![
            row![
                self.label("Address"),
                text_input("https://example.com/file.zip", &a.url)
                    .id(add_url_input_id())
                    .on_input(Message::AddUrlChanged)
                    .on_submit(Message::SubmitAdd)
                    .padding([6, 8])
                    .size(12)
                    .style(input_style(p)),
                button(text("Paste").size(11.5)).padding([6, 10]).on_press(Message::PasteIntoAdd).style(secondary_button(p)),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            row![self.label(""), hint],
            row![
                self.label("Save as"),
                text_input("Name from the address or server", &a.filename)
                    .on_input(Message::AddFilenameChanged)
                    .on_submit(Message::SubmitAdd)
                    .padding([6, 8])
                    .size(12)
                    .style(input_style(p)),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            row![
                self.label("Category"),
                pick_list(&FileCategory::ALL[..], Some(a.category.unwrap_or(category)), Message::AddCategoryPicked)
                    .text_size(11.5)
                    .padding([4, 8]),
                text(if a.category.is_some() { "chosen" } else { "detected from the name" }).size(10.5).color(p.muted),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            row![
                self.label("Save to folder"),
                text_input("Folder", &a.folder)
                    .on_input(Message::AddFolderChanged)
                    .on_submit(Message::SubmitAdd)
                    .padding([6, 8])
                    .size(12)
                    .style(input_style(p)),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            row![
                self.label(""),
                column![
                    checkbox("Put it in a category subfolder (e.g. …\\Video)", a.category_subfolder)
                        .on_toggle(Message::AddCategorySubfolder)
                        .size(13)
                        .text_size(11.5),
                ]
                .spacing(6),
            ],
        ]
        .spacing(8);
        if let Some(err) = &a.error {
            body = body.push(row![self.label(""), text(err).size(11.5).color(p.danger)]);
        }

        let footer = row![
            horizontal_space(),
            button(text("Cancel").size(12)).padding([6, 16]).on_press(Message::CloseDialog).style(secondary_button(p)),
            button(text("Download Later").size(12))
                .padding([6, 16])
                .on_press_maybe(url_ok.then_some(Message::AddLater))
                .style(secondary_button(p)),
            button(text("Start Download").size(12))
                .padding([6, 18])
                .on_press_maybe(url_ok.then_some(Message::SubmitAdd))
                .style(primary_button(p)),
        ]
        .spacing(8);
        self.dialog_frame("Add New Download".into(), 600.0, body.into(), footer.into())
    }

    fn dialog_batch(&self) -> Element<'_, Message> {
        let p = self.palette;
        let links = format::extract_links(&self.batch.text());
        let body = column![
            text("Paste links, one per line. Anything that isn't an http(s) or magnet link is skipped, and duplicates are merged.")
                .size(11.5)
                .color(p.muted),
            text_editor(&self.batch)
                .placeholder("https://example.com/part1.zip\nhttps://example.com/part2.zip")
                .on_action(Message::BatchAction)
                .height(Length::Fixed(220.0))
                .size(11.5),
            row![
                text(format!("{} link(s) found", links.len())).size(11.5).color(if links.is_empty() { p.muted } else { p.text }),
                horizontal_space(),
                checkbox("Start downloading now", self.batch_start_now)
                    .on_toggle(Message::BatchStartNow)
                    .size(13)
                    .text_size(11.5),
            ]
            .align_y(Alignment::Center),
            text(format!("Files go to {}", self.settings.download_dir)).size(10.5).color(p.muted),
        ]
        .spacing(8);
        let label: &'static str = if links.is_empty() { "Add Links" } else { "Add All" };
        self.dialog_frame(
            "Add Batch Download".into(),
            620.0,
            body.into(),
            self.footer(label, (!links.is_empty()).then_some(Message::SubmitBatch), false),
        )
    }

    fn dialog_options(&self) -> Element<'_, Message> {
        let p = self.palette;
        let o = &self.options;
        let tab = |label: &'static str, t: OptionsTab| {
            button(text(label).size(11.5))
                .padding([4, 14])
                .on_press(Message::SetOptionsTab(t))
                .style(flat_button(p, self.options_tab == t))
        };
        let field = |label: &'static str, value: &str, on: fn(String) -> Message, hint: String| {
            column![
                row![
                    text(label).size(11.5).color(p.muted).width(Length::Fixed(190.0)),
                    text_input("", value)
                        .on_input(on)
                        .on_submit(Message::SaveOptions)
                        .padding([5, 8])
                        .size(12)
                        .width(Length::Fixed(120.0))
                        .style(input_style(p)),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                row![text("").width(Length::Fixed(198.0)), text(hint).size(10.5).color(p.muted)],
            ]
            .spacing(2)
        };

        let page: Element<'_, Message> = match self.options_tab {
            OptionsTab::General => column![
                text("Default download folder").size(11.5).color(p.muted),
                text_input("C:\\Users\\you\\Downloads", &o.download_dir)
                    .on_input(Message::OptDownloadDir)
                    .on_submit(Message::SaveOptions)
                    .padding([6, 8])
                    .size(12)
                    .style(input_style(p)),
                checkbox("Sort new downloads into category subfolders by default", o.category_subfolders)
                    .on_toggle(Message::OptCategorySubfolders)
                    .size(13)
                    .text_size(11.5),
                text("Browser links (idr://) and batch downloads always use the default folder.").size(10.5).color(p.muted),
                horizontal_rule(1),
                checkbox("Open the download window when a download starts", o.show_progress_on_start)
                    .on_toggle(Message::OptShowProgress)
                    .size(13)
                    .text_size(11.5),
                checkbox("Show the file in Explorer when a download finishes", o.open_folder_when_done)
                    .on_toggle(Message::OptOpenFolderDone)
                    .size(13)
                    .text_size(11.5),
                checkbox("Show the \"Download complete\" dialog", o.show_complete_dialog)
                    .on_toggle(Message::OptShowComplete)
                    .size(13)
                    .text_size(11.5),
                checkbox("Watch the clipboard and offer to download copied links", o.monitor_clipboard)
                    .on_toggle(Message::OptMonitorClipboard)
                    .size(13)
                    .text_size(11.5),
                checkbox("Ask before removing downloads", o.confirm_remove)
                    .on_toggle(Message::OptConfirmRemove)
                    .size(13)
                    .text_size(11.5),
                checkbox("Start 1nternet Downloader when I sign in to Windows", o.start_with_windows)
                    .on_toggle(Message::OptStartWithWindows)
                    .size(13)
                    .text_size(11.5),
            ]
            .spacing(8)
            .into(),
            OptionsTab::Connection => column![
                field(
                    "Simultaneous downloads",
                    &o.max_concurrent,
                    Message::OptConcurrent,
                    format!("{}–{}", MIN_CONCURRENT_DOWNLOADS, MAX_CONCURRENT_DOWNLOADS)
                ),
                field(
                    "Connections per download",
                    &o.max_segments,
                    Message::OptSegments,
                    format!("{}–{}. Used only when the server accepts byte ranges.", MIN_SEGMENTS, MAX_SEGMENTS)
                ),
                field(
                    "Retries after an error",
                    &o.max_retries,
                    Message::OptRetries,
                    format!("{}–{}", MIN_RETRIES, MAX_RETRIES_LIMIT)
                ),
                field(
                    "Speed limit (KB/s)",
                    &o.speed_limit,
                    Message::OptSpeedLimit,
                    "0 = unlimited. Saved, but the engine doesn't enforce it yet.".into()
                ),
                text("Changes apply to downloads you add or restart from now on.").size(10.5).color(p.muted),
            ]
            .spacing(8)
            .into(),
            OptionsTab::Torrent => self.options_torrent_page(),
            OptionsTab::Appearance => column![
                row![
                    text("Theme").size(11.5).color(p.muted).width(Length::Fixed(190.0)),
                    pick_list(&ThemeMode::ALL[..], Some(o.theme), Message::OptTheme).text_size(12),
                ]
                .align_y(Alignment::Center),
                row![
                    text("Download progress colour").size(11.5).color(p.muted).width(Length::Fixed(190.0)),
                    pick_list(&Accent::ALL[..], Some(o.accent), Message::OptAccent).text_size(12),
                ]
                .align_y(Alignment::Center),
                self.accent_swatches(o.accent, o.theme),
                text("Column choice, sort order and pane visibility are under the View menu and are remembered automatically.")
                    .size(10.5)
                    .color(p.muted),
            ]
            .spacing(10)
            .into(),
        };

        let mut body = column![
            row![
                tab("General", OptionsTab::General),
                tab("Connection", OptionsTab::Connection),
                tab("Torrent", OptionsTab::Torrent),
                tab("Appearance", OptionsTab::Appearance)
            ]
            .spacing(2),
            horizontal_rule(1),
            container(scrollable(page)).height(Length::Fixed(340.0)).padding([6, 0]),
        ]
        .spacing(6);
        if let Some(err) = &o.error {
            body = body.push(text(err).size(11.5).color(p.danger));
        }
        self.dialog_frame("Options".into(), 600.0, body.into(), self.footer("Save", Some(Message::SaveOptions), false))
    }

    /// Clickable colour chips; the chosen one is outlined.
    fn accent_swatches(&self, chosen: Accent, mode: ThemeMode) -> Element<'_, Message> {
        let p = self.palette;
        let dark = mode.resolve() == ThemeMode::Dark;
        let mut r = row![text("").width(Length::Fixed(190.0))].spacing(6);
        for c in Accent::ALL {
            let color = c.color(dark);
            let on = c == chosen;
            r = r.push(
                button(Space::new(Length::Fixed(22.0), Length::Fixed(22.0)))
                    .padding(0)
                    .on_press(Message::OptAccent(c))
                    .style(move |_t, _s| button::Style {
                        background: Some(Background::Color(color)),
                        border: Border { color: if on { p.text } else { p.border }, width: if on { 2.5 } else { 1.0 }, radius: 11.0.into() },
                        ..Default::default()
                    }),
            );
        }
        r.into()
    }

    fn dialog_delete<'a>(&'a self, ids: &'a [i64]) -> Element<'a, Message> {
        let p = self.palette;
        let names: Vec<&str> = ids.iter().filter_map(|id| self.find(*id)).map(|d| d.filename.as_str()).collect();
        let running = ids
            .iter()
            .filter_map(|id| self.find(*id))
            .filter(|d| d.status == DownloadStatus::Downloading)
            .count();
        let mut list = column![].spacing(2);
        for n in names.iter().take(6) {
            list = list.push(text(format!("•  {}", n)).size(11.5).wrapping(Wrapping::None));
        }
        if names.len() > 6 {
            list = list.push(text(format!("…and {} more", names.len() - 6)).size(11.5).color(p.muted));
        }
        let mut body = column![
            text(format!("Remove {} download(s) from the list?", ids.len())).size(13),
            list,
            checkbox("Also delete the downloaded files from disk", self.delete_with_file)
                .on_toggle(Message::DeleteWithFile)
                .size(13)
                .text_size(11.5),
            checkbox("Don't ask me again (change it in Options)", self.delete_dont_ask)
                .on_toggle(Message::DeleteDontAsk)
                .size(13)
                .text_size(11.5),
        ]
        .spacing(10);
        if running > 0 {
            body = body.push(text(format!("{} of them are still downloading and will be stopped.", running)).size(11.5).color(p.warning));
        }
        self.dialog_frame("Remove Downloads".into(), 480.0, body.into(), self.footer("Remove", Some(Message::ConfirmDelete), true))
    }

    fn dialog_properties(&self, id: i64) -> Element<'_, Message> {
        let p = self.palette;
        let Some(d) = self.find(id) else {
            return self.dialog_frame(
                "Properties".into(),
                420.0,
                text("This download is no longer in the list.").size(12).into(),
                row![horizontal_space(), button(text("Close").size(12)).padding([6, 16]).on_press(Message::CloseDialog).style(secondary_button(p))].into(),
            );
        };
        let meta = self.meta.get(&d.id);
        let kv = |k: &'static str, v: String| {
            row![
                text(k).size(11.5).color(p.muted).width(Length::Fixed(130.0)),
                text(v).size(11.5).wrapping(Wrapping::WordOrGlyph),
            ]
            .spacing(8)
        };
        let category = FileCategory::from_filename(&d.filename);
        let mut info = column![
            kv("File name", d.filename.clone()),
            kv("Type", category.label().to_string()),
            kv("Status", status_label(d)),
            kv("Address", d.url.clone()),
            kv("Saved to", d.destination.clone()),
            kv("Size", format::size(d.total_bytes)),
            kv("Downloaded", format!("{} ({:.1}%)", format::bytes(d.downloaded_bytes), d.progress_percent)),
            kv("Transfer rate", format::rate(d.speed_bytes_per_sec)),
            kv("Time left", format::eta(d.eta_seconds)),
            kv("Added", format::datetime(meta.map(|m| m.added_at))),
            kv("Last try", format::datetime(meta.and_then(|m| m.last_try_at))),
        ]
        .spacing(4);
        if d.is_torrent {
            info = info.push(kv("Peers / seeds", format!("{} / {}", d.peers, d.seeds)));
        }
        if let Some(err) = &d.error_message {
            info = info.push(kv("Last error", err.clone()));
        }

        let headline = match d.status {
            DownloadStatus::Downloading if d.total_bytes > 0 => format!(
                "{} of {}  ·  {}  ·  {} left  ·  {} connection(s)",
                format::bytes(d.downloaded_bytes),
                format::bytes(d.total_bytes),
                format::rate(d.speed_bytes_per_sec),
                format::eta(d.eta_seconds),
                d.segments.iter().filter(|s| s.is_active).count().max(1)
            ),
            DownloadStatus::Downloading => "Connecting to the server…".to_string(),
            DownloadStatus::Completed => format!("Finished · {}", format::size(d.total_bytes)),
            DownloadStatus::Paused => format!("Paused at {} ({:.1}%)", format::bytes(d.downloaded_bytes), d.progress_percent),
            DownloadStatus::Queued => "Waiting in the queue".to_string(),
            DownloadStatus::Error => d.error_message.clone().unwrap_or_else(|| "Failed".into()),
        };
        let resume = if d.segments.len() > 1 {
            "Yes"
        } else if d.total_bytes == 0 {
            "(unknown)"
        } else {
            "Single connection"
        };
        let status_grid = column![
            text(d.url.clone()).size(11.5).color(p.muted).wrapping(Wrapping::Glyph),
            kv("Status", status_label(d)),
            kv("File size", format::size(d.total_bytes)),
            kv("Downloaded", format!("{} ({:.2}%)", format::bytes(d.downloaded_bytes), d.progress_percent)),
            kv("Transfer rate", format::rate(d.speed_bytes_per_sec)),
            kv("Time left", format::eta(d.eta_seconds)),
            kv("Resume capability", resume.to_string()),
        ]
        .spacing(3);
        let details = self.prefs.progress_details;
        let mut body = column![
            text(headline).size(11.5),
            status_grid,
            crate::ui::view::big_progress(p, d),
            self.segment_map(d),
            button(text(if details { "Hide details" } else { "Show details" }).size(11.5))
                .padding([3, 10])
                .on_press(Message::ToggleProgressDetails)
                .style(secondary_button(p)),
        ]
        .spacing(6);
        if details {
            body = body
                .push(text("Start positions and download progress by connections").size(10.5).color(p.muted))
                .push(scrollable(self.view_connections(d)).height(Length::Fixed(130.0)))
                .push(scrollable(info).height(Length::Fixed(150.0)));
        }

        let done = d.status == DownloadStatus::Completed;
        let toggle: Element<'_, Message> = match d.status {
            DownloadStatus::Downloading => button(text("Pause").size(12))
                .padding([6, 16])
                .on_press(Message::PauseOne(d.id))
                .style(secondary_button(p))
                .into(),
            DownloadStatus::Completed => button(text("Resume").size(12)).padding([6, 16]).style(secondary_button(p)).into(),
            _ => button(text("Resume").size(12))
                .padding([6, 16])
                .on_press(Message::ResumeOne(d.id))
                .style(primary_button(p))
                .into(),
        };
        let footer = row![
            toggle,
            button(text(if done { "Remove" } else { "Cancel" }).size(12))
                .padding([6, 14])
                .on_press(Message::RemoveOne(d.id))
                .style(secondary_button(p)),
            button(text("Open").size(12)).padding([6, 14]).on_press_maybe(done.then_some(Message::OpenFile)).style(secondary_button(p)),
            button(text("Open folder").size(12)).padding([6, 14]).on_press(Message::OpenFolder).style(secondary_button(p)),
            button(text("Copy address").size(12)).padding([6, 14]).on_press(Message::CopyUrls).style(secondary_button(p)),
            horizontal_space(),
            button(text("Close").size(12)).padding([6, 18]).on_press(Message::CloseDialog).style(primary_button(p)),
        ]
        .spacing(8);
        self.dialog_frame(format!("{:.0}%  {}", d.progress_percent, d.filename), 640.0, body.into(), footer.into())
    }

    fn dialog_complete(&self, id: i64) -> Element<'_, Message> {
        let p = self.palette;
        let Some(d) = self.find(id) else {
            return self.dialog_about();
        };
        let field = |label: &'static str, value: String| {
            column![
                text(label).size(10.5).color(p.muted),
                container(text(value).size(11.5).wrapping(Wrapping::Glyph))
                    .padding([4, 8])
                    .width(Length::Fill)
                    .style(move |_t| container::Style {
                        background: Some(Background::Color(p.panel)),
                        text_color: Some(p.text),
                        border: Border { color: p.border, width: 1.0, radius: 3.0.into() },
                        ..Default::default()
                    }),
            ]
            .spacing(3)
        };
        let body = column![
            row![
                icon(FileCategory::from_filename(&d.filename).glyph(), 28.0, p.text),
                column![
                    text(d.filename.clone()).size(13),
                    text(format!("Downloaded {} ({} bytes).", format::size(d.total_bytes), d.total_bytes)).size(11.5).color(p.muted),
                ]
                .spacing(2),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            field("Address", d.url.clone()),
            field("The file was saved as", d.destination.clone()),
            checkbox("Don't show this dialog again", self.complete_dont_show)
                .on_toggle(Message::CompleteDontShow)
                .size(13)
                .text_size(11.5),
        ]
        .spacing(10);
        let footer = row![
            button(text("Open").size(12)).padding([6, 18]).on_press(Message::OpenFile).style(primary_button(p)),
            button(text("Open folder").size(12)).padding([6, 14]).on_press(Message::OpenFolder).style(secondary_button(p)),
            horizontal_space(),
            button(text("Close").size(12)).padding([6, 18]).on_press(Message::CloseDialog).style(secondary_button(p)),
        ]
        .spacing(8);
        self.dialog_frame("Download complete".into(), 520.0, body.into(), footer.into())
    }

    fn dialog_about(&self) -> Element<'_, Message> {
        let p = self.palette;
        let keys = [
            ("Ctrl+N", "Add a download"),
            ("Ctrl+B", "Add a batch of links"),
            ("Ctrl+F", "Search"),
            ("Ctrl+A", "Select every visible row"),
            ("Ctrl / Shift + click", "Select several rows"),
            ("Double-click, Enter", "Properties"),
            ("Ctrl+R / Ctrl+P", "Resume / pause selected"),
            ("Del", "Remove selected"),
            ("Ctrl+O", "Options"),
            ("Esc", "Close a menu or dialog"),
        ];
        let mut table = column![].spacing(3);
        for (k, v) in keys {
            table = table.push(row![text(k).size(11.5).width(Length::Fixed(170.0)), text(v).size(11.5).color(p.muted)]);
        }
        let body = column![
            text("1nternet Downloader").size(15),
            text(format!("Version {}", APP_VERSION)).size(11.5).color(p.muted),
            text("Segmented HTTP(S) and BitTorrent download manager.").size(11.5),
            horizontal_rule(1),
            text("Keyboard shortcuts").size(11.5).color(p.muted),
            table,
        ]
        .spacing(8);
        self.dialog_frame(
            "About".into(),
            460.0,
            body.into(),
            row![horizontal_space(), button(text("Close").size(12)).padding([6, 18]).on_press(Message::CloseDialog).style(primary_button(p))].into(),
        )
    }
}
