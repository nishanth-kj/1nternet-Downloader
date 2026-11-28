//! Main window layout: menu bar, toolbar, category tree, download list, details and status bar.

use crate::model::{DownloadSnapshot, DownloadStatus};
use crate::ui::app::{search_input_id, DetailsTab, Dialog, Filter, IcedApp, MenuKind, Message};
use crate::ui::format::{self, FileCategory};
use crate::ui::prefs::Column;
use crate::ui::theme::Palette;
use iced::widget::{
    button, column, container, horizontal_rule, horizontal_space, mouse_area, opaque, progress_bar, row, scrollable,
    stack, text, text_input, tooltip, vertical_rule, Space,
};
use iced::widget::text::Wrapping;
use iced::{border, Alignment, Background, Border, Color, Element, Font, Length, Padding, Theme};

/// Windows' built-in icon font (Windows 10 and later).
pub(super) const ICON_FONT: Font = Font::with_name("Segoe MDL2 Assets");

pub(super) fn icon<'a>(glyph: &'a str, size: f32, color: Color) -> iced::widget::Text<'a> {
    text(glyph).font(ICON_FONT).size(size).color(color)
}

/// Width of a menu-bar title: its text plus even padding.
pub(super) fn menu_title_width(label: &str) -> f32 {
    label.chars().count() as f32 * 7.0 + 22.0
}
const MENU_BAR_HEIGHT: f32 = 22.0;
const ROW_HEIGHT: f32 = 22.0;

// ---- shared style helpers ---------------------------------------------------------------

pub(super) fn flat_button(p: Palette, active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_t, status| {
        let bg = match (active, status) {
            (true, _) => Some(Background::Color(p.selection)),
            (false, button::Status::Hovered) => Some(Background::Color(p.hover)),
            (false, button::Status::Pressed) => Some(Background::Color(p.selection)),
            _ => None,
        };
        button::Style {
            background: bg,
            text_color: if status == button::Status::Disabled { p.muted } else { p.text },
            border: Border {
                color: if active { p.selection_border } else { Color::TRANSPARENT },
                width: if active { 1.0 } else { 0.0 },
                radius: 3.0.into(),
            },
            ..Default::default()
        }
    }
}

pub(super) fn primary_button(p: Palette) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_t, status| {
        let mut bg = p.strong;
        if status == button::Status::Hovered {
            bg = Color { a: 0.88, ..bg };
        }
        button::Style {
            background: Some(Background::Color(if status == button::Status::Disabled { p.track } else { bg })),
            text_color: if status == button::Status::Disabled { p.muted } else { p.background },
            border: border::rounded(3),
            ..Default::default()
        }
    }
}

pub(super) fn secondary_button(p: Palette) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_t, status| button::Style {
        background: Some(Background::Color(if status == button::Status::Hovered { p.hover } else { p.panel })),
        text_color: if status == button::Status::Disabled { p.muted } else { p.text },
        border: Border { color: p.border, width: 1.0, radius: 3.0.into() },
        ..Default::default()
    }
}

pub(super) fn panel(p: Palette, bg: Color) -> impl Fn(&Theme) -> container::Style {
    move |_t| container::Style {
        background: Some(Background::Color(bg)),
        text_color: Some(p.text),
        ..Default::default()
    }
}

pub(super) fn input_style(p: Palette) -> impl Fn(&Theme, text_input::Status) -> text_input::Style {
    move |_t, status| text_input::Style {
        background: Background::Color(p.background),
        border: Border {
            color: if matches!(status, text_input::Status::Focused) { p.selection_border } else { p.border },
            width: 1.0,
            radius: 3.0.into(),
        },
        icon: p.muted,
        placeholder: p.muted,
        value: p.text,
        selection: p.selection,
    }
}

fn status_color(p: Palette, s: DownloadStatus) -> Color {
    match s {
        DownloadStatus::Downloading => p.accent,
        DownloadStatus::Completed => p.muted,
        DownloadStatus::Paused => p.warning,
        DownloadStatus::Error => p.danger,
        DownloadStatus::Queued => p.muted,
    }
}

pub(super) fn status_label(d: &DownloadSnapshot) -> String {
    match d.status {
        DownloadStatus::Downloading if d.total_bytes == 0 && d.is_torrent => "Fetching metadata…".into(),
        DownloadStatus::Downloading if d.total_bytes == 0 => "Connecting…".into(),
        DownloadStatus::Completed if d.is_torrent => "Complete (seeding)".into(),
        DownloadStatus::Downloading => "Downloading".into(),
        DownloadStatus::Completed => "Complete".into(),
        DownloadStatus::Paused => "Paused".into(),
        DownloadStatus::Error => "Error".into(),
        DownloadStatus::Queued => "Queued".into(),
    }
}

fn cell<'a>(s: impl text::IntoFragment<'a>, width: Length, color: Color) -> Element<'a, Message> {
    container(text(s).size(11.5).color(color).wrapping(Wrapping::Glyph))
        .width(width)
        .height(Length::Fixed(ROW_HEIGHT))
        .clip(true)
        .padding(Padding { top: 4.0, bottom: 0.0, left: 6.0, right: 6.0 })
        .into()
}

fn percent_bar<'a>(p: Palette, pct: f32, color: Color, height: f32) -> Element<'a, Message> {
    stack![
        progress_bar(0.0..=100.0, pct.clamp(0.0, 100.0))
            .height(Length::Fixed(height))
            .style(move |_t| progress_bar::Style {
                background: Background::Color(p.track),
                bar: Background::Color(color),
                border: Border { color: p.border, width: 1.0, radius: 2.0.into() },
            }),
        container(text(format!("{:.1}%", pct)).size(10).color(p.text))
            .center_x(Length::Fill)
            .center_y(Length::Fixed(height)),
    ]
    .into()
}

/// Large progress bar for the download window: percent plus the per-connection map.
pub(super) fn big_progress<'a>(p: Palette, d: &'a DownloadSnapshot) -> Element<'a, Message> {
    column![percent_bar(p, d.progress_percent, status_color(p, d.status), 22.0)].into()
}

// ---- main view --------------------------------------------------------------------------

impl IcedApp {
    pub fn view(&self) -> Element<'_, Message> {
        let p = self.palette;
        let mut body = row![];
        if self.prefs.show_categories {
            body = body.push(self.view_sidebar()).push(vertical_rule(1));
        }
        let mut main_col = column![self.view_list()].height(Length::Fill);
        if self.prefs.show_details {
            main_col = main_col.push(horizontal_rule(1)).push(self.view_details());
        }
        body = body.push(main_col);

        let base: Element<'_, Message> = container(
            column![
                self.view_menu_bar(),
                self.view_toolbar(),
                horizontal_rule(1),
                container(body).height(Length::Fill),
                horizontal_rule(1),
                self.view_status_bar(),
            ]
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .style(panel(p, p.background))
        .into();

        let mut layers = stack![base].width(Length::Fill).height(Length::Fill);

        if let Some(kind) = self.menu {
            let left: f32 = 6.0
                + MenuKind::ALL
                    .iter()
                    .take_while(|k| **k != kind)
                    .map(|k| menu_title_width(k.label()))
                    .sum::<f32>();
            layers = layers.push(self.click_away());
            layers = layers.push(
                container(opaque(self.view_menu_popup(self.menu_entries(kind))))
                    .padding(Padding { top: MENU_BAR_HEIGHT, left, ..Padding::ZERO }),
            );
        } else if let Some(pos) = self.context_menu {
            layers = layers.push(self.click_away());
            layers = layers.push(
                container(opaque(self.view_menu_popup(self.menu_entries(MenuKind::File))))
                    .padding(Padding { top: pos.y, left: pos.x, ..Padding::ZERO }),
            );
        }

        if let Some(dialog) = &self.dialog {
            layers = layers.push(self.view_dialog_layer(dialog));
        }

        mouse_area(layers).on_move(Message::CursorMoved).into()
    }

    fn click_away(&self) -> Element<'_, Message> {
        mouse_area(Space::new(Length::Fill, Length::Fill))
            .on_press(Message::CloseMenus)
            .on_right_press(Message::CloseMenus)
            .into()
    }

    fn view_menu_bar(&self) -> Element<'_, Message> {
        let p = self.palette;
        let mut bar = row![].spacing(0).padding([0, 6]).align_y(Alignment::Center);
        for kind in MenuKind::ALL {
            bar = bar.push(
                button(container(text(kind.label()).size(12)).center_x(Length::Fill))
                    .width(Length::Fixed(menu_title_width(kind.label())))
                    .padding([2, 0])
                    .on_press(Message::OpenMenu(kind))
                    .style(flat_button(p, self.menu == Some(kind))),
            );
        }
        container(bar)
            .height(Length::Fixed(MENU_BAR_HEIGHT))
            .width(Length::Fill)
            .center_y(Length::Fixed(MENU_BAR_HEIGHT))
            .style(panel(p, p.panel))
            .into()
    }

    fn view_toolbar(&self) -> Element<'_, Message> {
        let p = self.palette;
        let sel = self.selected_snaps();
        let can_resume = sel
            .iter()
            .any(|d| matches!(d.status, DownloadStatus::Paused | DownloadStatus::Error | DownloadStatus::Queued));
        let can_pause = sel.iter().any(|d| d.status == DownloadStatus::Downloading);
        let any_sel = !sel.is_empty();
        let any_done = self.downloads.iter().any(|d| d.status == DownloadStatus::Completed);

        let tool = |glyph: &'static str, label: &'static str, tip: &'static str, msg: Option<Message>, _color: Color| {
            let b = button(
                column![
                    icon(glyph, 18.0, if msg.is_some() { p.text } else { p.muted }),
                    text(label).size(10.5),
                ]
                .align_x(Alignment::Center)
                .spacing(1),
            )
            .width(Length::Fixed(68.0))
            .padding([4, 2])
            .on_press_maybe(msg)
            .style(flat_button(p, false));
            tooltip(b, container(text(tip).size(10.5)).padding(6).style(move |_t| container::Style {
                background: Some(Background::Color(p.panel)),
                text_color: Some(p.text),
                border: Border { color: p.border, width: 1.0, radius: 3.0.into() },
                ..Default::default()
            }), tooltip::Position::Bottom)
        };
        let sep = || container(vertical_rule(1)).height(Length::Fixed(30.0)).padding([0, 6]);

        let bar = row![
            tool("\u{E710}", "Add URL", "Add a new download (Ctrl+N)", Some(Message::OpenAddDialog), p.accent),
            tool("\u{E8FD}", "Batch", "Add many links at once (Ctrl+B)", Some(Message::OpenBatchDialog), p.accent),
            sep(),
            tool("\u{E768}", "Resume", "Start or resume the selected downloads (Ctrl+R)", can_resume.then_some(Message::ResumeSelected), p.success),
            tool("\u{E769}", "Pause", "Pause the selected downloads (Ctrl+P)", can_pause.then_some(Message::PauseSelected), p.warning),
            tool("\u{E896}", "Resume All", "Resume every unfinished download", Some(Message::ResumeAll), p.success),
            tool("\u{E71A}", "Pause All", "Pause every running download", Some(Message::PauseAll), p.warning),
            sep(),
            tool("\u{E74D}", "Delete", "Remove the selected downloads (Del)", any_sel.then_some(Message::RequestDeleteSelected), p.danger),
            tool("\u{E73E}", "Clear Done", "Remove all completed downloads from the list", any_done.then_some(Message::RequestDeleteCompleted), p.success),
            sep(),
            tool("\u{E713}", "Options", "Folders, connections and appearance (Ctrl+O)", Some(Message::OpenOptions), p.text),
            horizontal_space(),
            text_input("Search name or URL  (Ctrl+F)", &self.search)
                .id(search_input_id())
                .on_input(Message::SearchChanged)
                .width(Length::Fixed(220.0))
                .padding([4, 8])
                .size(12)
                .style(input_style(p)),
        ]
        .spacing(2)
        .padding([4, 8])
        .align_y(Alignment::Center);

        container(bar).width(Length::Fill).style(panel(p, p.toolbar)).into()
    }

    fn view_sidebar(&self) -> Element<'_, Message> {
        let p = self.palette;
        let item = |indent: u16, glyph: &'static str, label: String, f: Filter| {
            let n = self.count(f);
            let active = self.filter == f;
            button(
                row![
                    Space::with_width(Length::Fixed(indent as f32)),
                    icon(glyph, 12.0, if active { p.text } else { p.muted }).width(Length::Fixed(20.0)),
                    text(label).size(12).wrapping(Wrapping::None),
                    horizontal_space(),
                    text(if n > 0 { n.to_string() } else { String::new() }).size(10.5).color(p.muted),
                ]
                .align_y(Alignment::Center),
            )
            .width(Length::Fill)
            .padding([2, 8])
            .on_press(Message::SelectFilter(f))
            .style(flat_button(p, active))
        };
        let heading = |s: &'static str| container(text(s).size(10.5).color(p.muted)).padding(Padding { top: 10.0, bottom: 2.0, left: 10.0, right: 0.0 });

        let mut tree = column![
            row![
                button(icon(if self.all_expanded { "\u{E70D}" } else { "\u{E76C}" }, 10.0, p.muted))
                    .padding([3, 4])
                    .on_press(Message::ToggleAllExpanded)
                    .style(flat_button(p, false)),
                item(0, "\u{E896}", "All Downloads".into(), Filter::All),
            ]
            .align_y(Alignment::Center),
        ]
        .spacing(1);
        if self.all_expanded {
            for c in FileCategory::ALL {
                tree = tree.push(item(22, c.glyph(), c.label().into(), Filter::Category(c)));
            }
        }
        tree = tree
            .push(item(0, "\u{E823}", "Unfinished".into(), Filter::Unfinished))
            .push(item(0, "\u{E930}", "Finished".into(), Filter::Finished))
            .push(heading("STATUS"))
            .push(item(0, "\u{E74B}", "Downloading".into(), Filter::Downloading))
            .push(item(0, "\u{E916}", "Queued".into(), Filter::Queued))
            .push(item(0, "\u{E769}", "Paused".into(), Filter::Paused))
            .push(item(0, "\u{E783}", "Failed".into(), Filter::Failed))
            .push(heading("PROTOCOL"))
            .push(item(0, "\u{E8AB}", "Torrents".into(), Filter::Torrents));

        container(scrollable(tree.padding([6, 4])).height(Length::Fill))
            .width(Length::Fixed(180.0))
            .height(Length::Fill)
            .style(panel(p, p.sidebar))
            .into()
    }

    fn view_list(&self) -> Element<'_, Message> {
        let p = self.palette;
        let cols = &self.prefs.columns;

        let mut header = row![].align_y(Alignment::Center);
        for &c in cols {
            let arrow = if self.prefs.sort == c {
                if self.prefs.sort_descending { "\u{E70D}" } else { "\u{E70E}" }
            } else {
                ""
            };
            header = header.push(
                button(
                    row![text(c.label()).size(11.5).wrapping(Wrapping::None), icon(arrow, 9.0, p.muted)]
                        .spacing(6)
                        .align_y(Alignment::Center),
                )
                .clip(true)
                    .width(c.width())
                    .padding([3, 6])
                    .on_press(Message::SortBy(c))
                    .style(move |_t, status| button::Style {
                        background: Some(Background::Color(if status == button::Status::Hovered { p.hover } else { p.header })),
                        text_color: p.text,
                        border: Border { color: p.border, width: 0.5, radius: 0.0.into() },
                        ..Default::default()
                    }),
            );
        }

        let rows = self.visible();
        let body: Element<'_, Message> = if rows.is_empty() {
            let msg = if self.downloads.is_empty() {
                "No downloads yet. Click “Add URL” (Ctrl+N), paste a link, or send one from your browser."
            } else {
                "Nothing matches this category or search."
            };
            container(text(msg).size(12).color(p.muted)).padding(24).center_x(Length::Fill).into()
        } else {
            let mut list = column![];
            for (i, d) in rows.iter().enumerate() {
                list = list.push(self.view_row(d, i));
            }
            scrollable(list).height(Length::Fill).into()
        };

        column![header, body].height(Length::FillPortion(3)).into()
    }

    fn view_row<'a>(&'a self, d: &'a DownloadSnapshot, index: usize) -> Element<'a, Message> {
        let p = self.palette;
        let selected = self.selected.contains(&d.id);
        let meta = self.meta.get(&d.id);
        let sc = status_color(p, d.status);

        let mut r = row![].align_y(Alignment::Center).height(Length::Fixed(ROW_HEIGHT));
        for &c in &self.prefs.columns {
            let w = c.width();
            let el: Element<'a, Message> = match c {
                Column::FileName => {
                    let cat = FileCategory::from_filename(&d.filename);
                    container(row![
                        container(icon(cat.glyph(), 12.0, p.muted))
                            .width(Length::Fixed(20.0))
                            .padding(Padding { top: 2.0, ..Padding::ZERO }),
                        text(&d.filename).size(11.5).wrapping(Wrapping::Glyph),
                    ])
                    .width(w)
                    .height(Length::Fixed(ROW_HEIGHT))
                    .clip(true)
                    .padding(Padding { top: 4.0, bottom: 0.0, left: 6.0, right: 6.0 })
                    .into()
                }
                Column::Type => cell(FileCategory::from_filename(&d.filename).label(), w, p.muted),
                Column::Size => cell(format::size(d.total_bytes), w, p.text),
                Column::Downloaded => cell(format::bytes(d.downloaded_bytes), w, p.text),
                Column::Progress => container(percent_bar(p, d.progress_percent, sc, 16.0)).width(w).padding([0, 6]).into(),
                Column::Status => cell(status_label(d), w, sc),
                Column::TimeLeft => cell(
                    if d.status == DownloadStatus::Downloading { format::eta(d.eta_seconds) } else { String::new() },
                    w,
                    p.text,
                ),
                Column::TransferRate => cell(format::rate(d.speed_bytes_per_sec), w, p.text),
                Column::Connections => cell(
                    if d.is_torrent && d.status == DownloadStatus::Downloading {
                        // seeds / peers
                        format!("{}/{}", d.seeds, d.peers)
                    } else if d.status == DownloadStatus::Downloading {
                        d.segments.iter().filter(|s| s.is_active).count().max(1).to_string()
                    } else {
                        String::new()
                    },
                    w,
                    p.text,
                ),
                Column::LastTry => cell(format::datetime(meta.and_then(|m| m.last_try_at)), w, p.muted),
                Column::Added => cell(format::datetime(meta.map(|m| m.added_at)), w, p.muted),
                Column::SavePath => cell(format::short_folder(&d.destination), w, p.muted),
                Column::Url => cell(&d.url, w, p.muted),
            };
            r = r.push(el);
        }

        let bg = if selected {
            p.selection
        } else if index % 2 == 1 {
            p.row_alt
        } else {
            p.row
        };
        let styled = container(r).width(Length::Fill).style(move |_t| container::Style {
            background: Some(Background::Color(bg)),
            text_color: Some(p.text),
            border: Border {
                color: if selected { p.selection_border } else { Color::TRANSPARENT },
                width: if selected { 1.0 } else { 0.0 },
                radius: 0.0.into(),
            },
            ..Default::default()
        });

        mouse_area(styled)
            .on_press(Message::RowPressed(d.id))
            .on_right_press(Message::RowRightPressed(d.id))
            .into()
    }

    fn view_details(&self) -> Element<'_, Message> {
        let p = self.palette;
        let tab = |label: &'static str, t: DetailsTab| {
            button(text(label).size(11.5))
                .padding([3, 12])
                .on_press(Message::SetDetailsTab(t))
                .style(flat_button(p, self.details_tab == t))
        };
        let tabs = row![tab("General", DetailsTab::General), tab("Connections", DetailsTab::Connections)].spacing(2);

        let content: Element<'_, Message> = match self.selected_snaps().as_slice() {
            [] => text("Select a download to see its details. Double-click a row for all properties.")
                .size(11.5)
                .color(p.muted)
                .into(),
            [d] => match self.details_tab {
                DetailsTab::General => self.view_general(d),
                DetailsTab::Connections => self.view_connections(d),
            },
            many => {
                let total: u64 = many.iter().map(|d| d.total_bytes).sum();
                let done: u64 = many.iter().map(|d| d.downloaded_bytes).sum();
                let speed: f64 = many.iter().map(|d| d.speed_bytes_per_sec).sum();
                column![
                    text(format!("{} downloads selected", many.len())).size(12),
                    text(format!(
                        "{} of {} downloaded · {}",
                        format::bytes(done),
                        format::bytes(total),
                        if speed > 0.0 { format::rate(speed) } else { "idle".into() }
                    ))
                    .size(11.5)
                    .color(p.muted),
                ]
                .spacing(4)
                .into()
            }
        };

        container(column![tabs, horizontal_rule(1), scrollable(container(content).padding([6, 2])).height(Length::Fill)].spacing(4))
            .padding([6, 10])
            .width(Length::Fill)
            .height(Length::Fixed(180.0))
            .style(panel(p, p.panel))
            .into()
    }

    /// The segment map: one bar per connection, sized by its share of the file.
    pub(super) fn segment_map<'a>(&self, d: &'a DownloadSnapshot) -> Element<'a, Message> {
        let p = self.palette;
        let sc = status_color(p, d.status);
        if d.segments.is_empty() || d.total_bytes == 0 {
            return percent_bar(p, d.progress_percent, sc, 18.0);
        }
        let mut segs: Vec<_> = d.segments.iter().collect();
        segs.sort_by_key(|s| s.start_offset);
        let mut map = row![].spacing(1);
        for s in segs {
            let share = ((s.total_bytes() as f64 / d.total_bytes as f64) * 1000.0).round().max(1.0) as u16;
            // Finished pieces take the row colour: green while downloading, grey once complete.
            let color = if s.is_completed || s.is_active { sc } else { p.muted };
            map = map.push(
                progress_bar(0.0..=1.0, s.progress_ratio())
                    .width(Length::FillPortion(share))
                    .height(Length::Fixed(18.0))
                    .style(move |_t| progress_bar::Style {
                        background: Background::Color(p.track),
                        bar: Background::Color(color),
                        border: Border { color: p.border, width: 0.5, radius: 0.0.into() },
                    }),
            );
        }
        map.into()
    }

    fn view_general<'a>(&'a self, d: &'a DownloadSnapshot) -> Element<'a, Message> {
        let p = self.palette;
        let kv = |k: &'static str, v: String| {
            row![
                text(k).size(11.5).color(p.muted).width(Length::Fixed(110.0)),
                text(v).size(11.5).wrapping(Wrapping::WordOrGlyph),
            ]
            .spacing(6)
        };
        let resume = if d.segments.len() > 1 {
            "Yes (server accepts byte ranges)"
        } else if d.total_bytes == 0 {
            "Unknown until the server responds"
        } else {
            "Single connection"
        };
        let mut col = column![
            self.segment_map(d),
            row![
                column![
                    kv("Status", status_label(d)),
                    kv("File size", format::size(d.total_bytes)),
                    kv("Downloaded", format!("{} ({:.1}%)", format::bytes(d.downloaded_bytes), d.progress_percent)),
                    kv("Transfer rate", format::rate(d.speed_bytes_per_sec)),
                    kv("Time left", format::eta(d.eta_seconds)),
                ]
                .spacing(3)
                .width(Length::FillPortion(2)),
                column![
                    kv("Address", d.url.clone()),
                    kv("Save to", d.destination.clone()),
                    kv("Connections", format!("{} of {}", d.segments.iter().filter(|s| s.is_active).count(), d.segments.len().max(1))),
                    kv("Resume", resume.to_string()),
                ]
                .spacing(3)
                .width(Length::FillPortion(3)),
            ]
            .spacing(16),
        ]
        .spacing(8);
        if let Some(err) = &d.error_message {
            col = col.push(text(format!("Last error: {}", err)).size(11.5).color(p.danger));
        }
        col.into()
    }

    pub(super) fn view_connections<'a>(&'a self, d: &'a DownloadSnapshot) -> Element<'a, Message> {
        let p = self.palette;
        if d.segments.is_empty() {
            return text("This download uses a single connection (no segment data yet, or the server doesn't support ranges).")
                .size(11.5)
                .color(p.muted)
                .into();
        }
        let h = |s: &'static str, w: f32| text(s).size(10.5).color(p.muted).width(Length::Fixed(w));
        let mut col = column![row![
            h("#", 34.0),
            h("Start", 96.0),
            h("End", 96.0),
            h("Downloaded", 150.0),
            h("Progress", 160.0),
            h("State", 80.0)
        ]]
        .spacing(2);
        let mut segs: Vec<_> = d.segments.iter().collect();
        segs.sort_by_key(|s| s.start_offset);
        for s in segs {
            let (state, color) = if s.is_completed {
                ("Done", p.success)
            } else if s.is_active {
                ("Receiving", p.accent)
            } else {
                ("Waiting", p.muted)
            };
            col = col.push(
                row![
                    text(s.segment_id.to_string()).size(11.5).width(Length::Fixed(34.0)),
                    text(format::bytes(s.start_offset)).size(11.5).width(Length::Fixed(96.0)),
                    text(format::bytes(s.end_offset)).size(11.5).width(Length::Fixed(96.0)),
                    text(format!("{} / {}", format::bytes(s.downloaded_bytes), format::bytes(s.total_bytes())))
                        .size(11.5)
                        .width(Length::Fixed(150.0)),
                    container(percent_bar(p, s.progress_ratio() * 100.0, color, 14.0))
                        .width(Length::Fixed(150.0))
                        .padding(Padding { right: 10.0, ..Padding::ZERO }),
                    text(state).size(11.5).color(color).width(Length::Fixed(80.0)),
                ]
                .align_y(Alignment::Center),
            );
        }
        col.into()
    }

    fn view_status_bar(&self) -> Element<'_, Message> {
        let p = self.palette;
        let active = self.count(Filter::Downloading);
        let speed: f64 = self.downloads.iter().map(|d| d.speed_bytes_per_sec).sum();
        let note = self.status_note.clone().unwrap_or_else(|| "Ready".into());
        let sep = || container(vertical_rule(1)).height(Length::Fixed(14.0));
        container(
            row![
                text(note).size(11.5),
                horizontal_space(),
                text(format!("{} items", self.downloads.len())).size(11.5).color(p.muted),
                sep(),
                text(format!("{} active", active)).size(11.5).color(p.muted),
                sep(),
                row![
                    icon("\u{E896}", 11.0, p.accent),
                    text(if speed > 0.0 { format::rate(speed) } else { "0 B/s".into() }).size(11.5).color(p.accent)
                ]
                .spacing(4)
                .align_y(Alignment::Center),
                sep(),
                text(format!("Folder: {}", self.settings.download_dir)).size(11.5).color(p.muted).wrapping(Wrapping::None),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        )
        .padding([3, 10])
        .width(Length::Fill)
        .style(panel(p, p.panel))
        .into()
    }

    // ---- menus ------------------------------------------------------------------------

    fn menu_entries(&self, kind: MenuKind) -> Vec<MenuEntry> {
        use MenuEntry::*;
        let sel = self.selected_snaps();
        let one = sel.len() == 1;
        let first_done = sel.first().is_some_and(|d| d.status == DownloadStatus::Completed);
        let can_resume = sel
            .iter()
            .any(|d| matches!(d.status, DownloadStatus::Paused | DownloadStatus::Error | DownloadStatus::Queued));
        let can_pause = sel.iter().any(|d| d.status == DownloadStatus::Downloading);
        let any = !sel.is_empty();
        let item = |label: &str, key: &str, msg: Option<Message>| Item { label: label.into(), shortcut: key.into(), msg, checked: None };
        let check = |label: &str, on: bool, msg: Message| Item { label: label.into(), shortcut: String::new(), msg: Some(msg), checked: Some(on) };

        match kind {
            MenuKind::Tasks => vec![
                item("Add new download…", "Ctrl+N", Some(Message::OpenAddDialog)),
                item("Add batch download…", "Ctrl+B", Some(Message::OpenBatchDialog)),
                item("Add links from clipboard", "", Some(Message::BatchFromClipboard)),
                Separator,
                item("Exit", "", Some(Message::Exit)),
            ],
            MenuKind::File => vec![
                item("Open", "", (one && first_done).then_some(Message::OpenFile)),
                item("Open folder", "", one.then_some(Message::OpenFolder)),
                item("Copy address", "", any.then_some(Message::CopyUrls)),
                Separator,
                item("Resume", "Ctrl+R", can_resume.then_some(Message::ResumeSelected)),
                item("Pause", "Ctrl+P", can_pause.then_some(Message::PauseSelected)),
                Separator,
                item("Remove…", "Del", any.then_some(Message::RequestDeleteSelected)),
                item("Download window / properties", "Enter", any.then_some(Message::ShowProperties)),
            ],
            MenuKind::Downloads => vec![
                item("Resume all", "", Some(Message::ResumeAll)),
                item("Pause all", "", Some(Message::PauseAll)),
                Separator,
                item("Remove all completed…", "", Some(Message::RequestDeleteCompleted)),
                Separator,
                item("Find", "Ctrl+F", Some(Message::FocusSearch)),
                item("Select all", "Ctrl+A", Some(Message::SelectAll)),
                Separator,
                item("Options…", "Ctrl+O", Some(Message::OpenOptions)),
            ],
            MenuKind::Torrent => {
                let torrent = sel.first().filter(|d| d.is_torrent && sel.len() == 1);
                let tid = torrent.map(|d| d.id);
                let seq = tid
                    .and_then(|id| self.torrent_view.as_ref().filter(|(v, _)| *v == id).map(|(_, d)| d.sequential))
                    .unwrap_or(false);
                vec![
                    item("Add magnet or .torrent link…", "Ctrl+N", Some(Message::OpenAddDialog)),
                    item("Create torrent…", "", Some(Message::OpenCreateTorrent)),
                    Separator,
                    Heading("Selected torrent".into()),
                    item("Torrent details…", "Enter", tid.map(|id| Message::OpenTorrentDetails(id, crate::ui::torrent_ui::TorrentTab::General))),
                    item("Files and priorities…", "", tid.map(|id| Message::OpenTorrentDetails(id, crate::ui::torrent_ui::TorrentTab::Files))),
                    item("Trackers… (add / reannounce)", "", tid.map(|id| Message::OpenTorrentDetails(id, crate::ui::torrent_ui::TorrentTab::Trackers))),
                    item("Peers…", "", tid.map(|id| Message::OpenTorrentDetails(id, crate::ui::torrent_ui::TorrentTab::Peers))),
                    Item {
                        label: "Download in order (sequential)".into(),
                        shortcut: String::new(),
                        msg: tid.map(|id| Message::TorrentSequential(id, !seq)),
                        checked: Some(seq),
                    },
                    item("Force recheck", "", tid.map(Message::TorrentRecheck)),
                    item("Force reannounce", "", tid.map(Message::TorrentReannounce)),
                    Separator,
                    item("Show torrents only", "", Some(Message::SelectFilter(Filter::Torrents))),
                    item("Torrent settings…", "", Some(Message::OpenTorrentSettings)),
                ]
            }
            MenuKind::View => {
                let mut v = vec![
                    check("Categories pane", self.prefs.show_categories, Message::ToggleCategories),
                    check("Details pane", self.prefs.show_details, Message::ToggleDetails),
                    Separator,
                    // Theme and colours live in Options › Appearance.
                    item("Theme and colours…", "", Some(Message::OpenAppearance)),
                    Separator,
                    Heading("Columns".into()),
                ];
                for c in Column::ALL.into_iter().filter(|c| c.hideable()) {
                    v.push(check(c.label(), self.prefs.columns.contains(&c), Message::ToggleColumn(c)));
                }
                v
            }
            MenuKind::Help => vec![
                item("Keyboard shortcuts & about…", "", Some(Message::OpenAbout)),
            ],
        }
    }

    fn view_menu_popup(&self, entries: Vec<MenuEntry>) -> Element<'_, Message> {
        let p = self.palette;
        let mut col = column![].spacing(0).width(Length::Fixed(250.0));
        for e in entries {
            col = col.push(match e {
                MenuEntry::Separator => Element::from(container(horizontal_rule(1)).padding([3, 6])),
                MenuEntry::Heading(h) => container(text(h).size(10.5).color(p.muted)).padding([4, 12]).into(),
                MenuEntry::Item { label, shortcut, msg, checked } => {
                    let enabled = msg.is_some();
                    let mark = match checked {
                        Some(true) => "\u{E73E}",
                        _ => "",
                    };
                    button(
                        row![
                            icon(mark, 11.0, p.text).width(Length::Fixed(20.0)),
                            text(label).size(12).color(if enabled { p.text } else { p.muted }),
                            horizontal_space(),
                            text(shortcut).size(10.5).color(p.muted),
                        ]
                        .align_y(Alignment::Center),
                    )
                    .width(Length::Fill)
                    .padding([4, 8])
                    .on_press_maybe(msg)
                    .style(flat_button(p, false))
                    .into()
                }
            });
        }
        container(col)
            .padding(4)
            .style(move |_t| container::Style {
                background: Some(Background::Color(p.background)),
                text_color: Some(p.text),
                border: Border { color: p.border, width: 1.0, radius: 4.0.into() },
                shadow: iced::Shadow {
                    color: Color { a: 0.25, ..Color::BLACK },
                    offset: iced::Vector::new(0.0, 3.0),
                    blur_radius: 10.0,
                },
            })
            .into()
    }

    fn view_dialog_layer<'a>(&'a self, dialog: &'a Dialog) -> Element<'a, Message> {
        let p = self.palette;
        let inner = self.view_dialog(dialog);
        opaque(
            mouse_area(
                container(opaque(inner))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .style(move |_t| container::Style {
                        background: Some(Background::Color(p.overlay)),
                        ..Default::default()
                    }),
            ),
        )
    }
}

pub(super) enum MenuEntry {
    Item { label: String, shortcut: String, msg: Option<Message>, checked: Option<bool> },
    Heading(String),
    Separator,
}

