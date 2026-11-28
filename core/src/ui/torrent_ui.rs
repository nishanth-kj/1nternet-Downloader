//! Torrent screens: the details window, "Create torrent" and the Options › Torrent page.

use crate::config::{TorrentEncryption, TorrentSettings};
use crate::service::r#impl::torrent::{TorrentDetails, TrackerState};
use crate::ui::app::{IcedApp, Message};
use crate::ui::format;
use crate::ui::view::{flat_button, input_style, primary_button, secondary_button};
use iced::widget::text::Wrapping;
use iced::widget::{
    button, checkbox, column, container, horizontal_rule, horizontal_space, pick_list, progress_bar, row, scrollable,
    text, text_editor, text_input,
};
use iced::{Alignment, Background, Border, Color, Element, Length};

/// Tabs of the torrent details window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TorrentTab {
    General,
    Files,
    Trackers,
    Peers,
    Pieces,
}

/// File download priority as shown to the user (libtorrent values 0..7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilePriority {
    Skip,
    Low,
    Normal,
    High,
    Maximum,
}

impl FilePriority {
    pub const ALL: [FilePriority; 5] =
        [FilePriority::Skip, FilePriority::Low, FilePriority::Normal, FilePriority::High, FilePriority::Maximum];

    pub fn value(self) -> i32 {
        match self {
            FilePriority::Skip => 0,
            FilePriority::Low => 1,
            FilePriority::Normal => 4,
            FilePriority::High => 6,
            FilePriority::Maximum => 7,
        }
    }

    pub fn from_value(v: i32) -> Self {
        match v {
            0 => FilePriority::Skip,
            1..=3 => FilePriority::Low,
            4 => FilePriority::Normal,
            5 | 6 => FilePriority::High,
            _ => FilePriority::Maximum,
        }
    }
}

impl std::fmt::Display for FilePriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            FilePriority::Skip => "Don't download",
            FilePriority::Low => "Low",
            FilePriority::Normal => "Normal",
            FilePriority::High => "High",
            FilePriority::Maximum => "Maximum",
        })
    }
}

/// Piece size choices for "Create torrent".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PieceSize {
    Auto,
    K256,
    K512,
    M1,
    M2,
    M4,
    M8,
    M16,
}

impl PieceSize {
    pub const ALL: [PieceSize; 8] = [
        PieceSize::Auto,
        PieceSize::K256,
        PieceSize::K512,
        PieceSize::M1,
        PieceSize::M2,
        PieceSize::M4,
        PieceSize::M8,
        PieceSize::M16,
    ];

    pub fn kib(self) -> i32 {
        match self {
            PieceSize::Auto => 0,
            PieceSize::K256 => 256,
            PieceSize::K512 => 512,
            PieceSize::M1 => 1024,
            PieceSize::M2 => 2048,
            PieceSize::M4 => 4096,
            PieceSize::M8 => 8192,
            PieceSize::M16 => 16384,
        }
    }
}

impl std::fmt::Display for PieceSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PieceSize::Auto => f.write_str("Automatic"),
            p if p.kib() < 1024 => write!(f, "{} KiB", p.kib()),
            p => write!(f, "{} MiB", p.kib() / 1024),
        }
    }
}

/// Form state for "Create torrent".
pub struct CreateTorrentForm {
    pub source: String,
    pub output: String,
    pub output_edited: bool,
    pub trackers: text_editor::Content,
    pub web_seeds: String,
    pub comment: String,
    pub piece: PieceSize,
    pub private: bool,
    pub seed: bool,
    pub busy: bool,
    pub error: Option<String>,
    pub done: Option<String>,
}

impl Default for CreateTorrentForm {
    fn default() -> Self {
        Self {
            source: String::new(),
            output: String::new(),
            output_edited: false,
            trackers: text_editor::Content::new(),
            web_seeds: String::new(),
            comment: String::new(),
            piece: PieceSize::Auto,
            private: false,
            seed: true,
            busy: false,
            error: None,
            done: None,
        }
    }
}

/// Editable copy of `TorrentSettings` for the Options page (numbers as text).
pub struct TorrentDraft {
    pub s: TorrentSettings,
    pub port: String,
    pub max_conn: String,
    pub max_peers: String,
    pub dl: String,
    pub ul: String,
    pub ratio: String,
    pub seed_minutes: String,
    pub trackers: text_editor::Content,
}

impl TorrentDraft {
    pub fn from_settings(s: TorrentSettings) -> Self {
        Self {
            port: s.listen_port.to_string(),
            max_conn: s.max_connections.to_string(),
            max_peers: s.max_peers_per_torrent.to_string(),
            dl: s.download_limit_kbps.to_string(),
            ul: s.upload_limit_kbps.to_string(),
            ratio: if s.seed_ratio_limit > 0.0 { format!("{}", s.seed_ratio_limit) } else { "0".into() },
            seed_minutes: s.seed_time_limit_minutes.to_string(),
            trackers: text_editor::Content::with_text(&s.extra_trackers),
            s,
        }
    }

    /// Validates the text fields and returns finished settings.
    pub fn to_settings(&self) -> Result<TorrentSettings, String> {
        fn num(label: &str, v: &str, min: u32, max: u32) -> Result<u32, String> {
            match v.trim().parse::<u32>() {
                Ok(n) if (min..=max).contains(&n) => Ok(n),
                _ => Err(format!("{} must be a whole number from {} to {}.", label, min, max)),
            }
        }
        let mut s = self.s.clone();
        s.listen_port = num("Port", &self.port, 1, 65535)? as u16;
        s.max_connections = num("Maximum connections", &self.max_conn, 10, 5000)?;
        s.max_peers_per_torrent = num("Peers per torrent", &self.max_peers, 2, 1000)?;
        s.download_limit_kbps = num("Download limit", &self.dl, 0, 10_000_000)?;
        s.upload_limit_kbps = num("Upload limit", &self.ul, 0, 10_000_000)?;
        s.seed_time_limit_minutes = num("Seeding time", &self.seed_minutes, 0, 1_000_000)?;
        s.seed_ratio_limit = match self.ratio.trim().parse::<f64>() {
            Ok(r) if (0.0..=1000.0).contains(&r) => r,
            _ => return Err("Share ratio must be 0 (no limit) or a number such as 2 or 1.5.".into()),
        };
        s.extra_trackers = self.trackers.text().trim().to_string();
        Ok(s)
    }
}

/// Changes on the Options › Torrent page.
#[derive(Debug, Clone)]
pub enum TorrentOpt {
    Port(String),
    RandomPort(bool),
    MaxConn(String),
    MaxPeers(String),
    Encryption(TorrentEncryption),
    Dht(bool),
    Lsd(bool),
    Pex(bool),
    Utp(bool),
    Upnp(bool),
    NatPmp(bool),
    AnnounceAll(bool),
    ValidateHttps(bool),
    Trackers(text_editor::Action),
    Dl(String),
    Ul(String),
    Sequential(bool),
    SeedFinished(bool),
    Ratio(String),
    SeedMinutes(String),
    UserAgent(String),
}

impl TorrentDraft {
    pub fn apply(&mut self, m: TorrentOpt) {
        match m {
            TorrentOpt::Port(v) => self.port = v,
            TorrentOpt::RandomPort(b) => self.s.random_port = b,
            TorrentOpt::MaxConn(v) => self.max_conn = v,
            TorrentOpt::MaxPeers(v) => self.max_peers = v,
            TorrentOpt::Encryption(e) => self.s.encryption = e,
            TorrentOpt::Dht(b) => self.s.dht = b,
            TorrentOpt::Lsd(b) => self.s.lsd = b,
            TorrentOpt::Pex(b) => self.s.pex = b,
            TorrentOpt::Utp(b) => self.s.utp = b,
            TorrentOpt::Upnp(b) => self.s.upnp = b,
            TorrentOpt::NatPmp(b) => self.s.natpmp = b,
            TorrentOpt::AnnounceAll(b) => self.s.announce_to_all_trackers = b,
            TorrentOpt::ValidateHttps(b) => self.s.validate_https_trackers = b,
            TorrentOpt::Trackers(a) => self.trackers.perform(a),
            TorrentOpt::Dl(v) => self.dl = v,
            TorrentOpt::Ul(v) => self.ul = v,
            TorrentOpt::Sequential(b) => self.s.sequential_by_default = b,
            TorrentOpt::SeedFinished(b) => self.s.seed_finished = b,
            TorrentOpt::Ratio(v) => self.ratio = v,
            TorrentOpt::SeedMinutes(v) => self.seed_minutes = v,
            TorrentOpt::UserAgent(v) => self.s.user_agent = v,
        }
    }
}

fn duration(secs: i64) -> String {
    if secs <= 0 {
        return "0s".into();
    }
    format::eta(Some(secs as u64))
}

impl IcedApp {
    fn small_input<'a>(&self, value: &'a str, on: impl Fn(String) -> Message + 'a, width: f32) -> Element<'a, Message> {
        text_input("", value)
            .on_input(on)
            .padding([4, 8])
            .size(12)
            .width(Length::Fixed(width))
            .style(input_style(self.palette))
            .into()
    }

    /// Options › Torrent.
    pub(super) fn options_torrent_page(&self) -> Element<'_, Message> {
        let p = self.palette;
        let t = &self.torrent_draft;
        let check = |label: &'static str, on: bool, f: fn(bool) -> TorrentOpt| {
            checkbox(label, on).on_toggle(move |b| Message::OptTorrent(f(b))).size(14).text_size(11.5)
        };
        let field = |label: &'static str, input: Element<'static, Message>, hint: &'static str| {
            row![
                text(label).size(11.5).color(p.muted).width(Length::Fixed(170.0)),
                input,
                text(hint).size(10.5).color(p.muted),
            ]
            .spacing(8)
            .align_y(Alignment::Center)
        };
        let heading = |s: &'static str| text(s).size(12).color(p.text);
        let inp = |v: &str, f: fn(String) -> TorrentOpt, w: f32| -> Element<'static, Message> {
            text_input("", v)
                .on_input(move |s| Message::OptTorrent(f(s)))
                .padding([4, 8])
                .size(12)
                .width(Length::Fixed(w))
                .style(input_style(p))
                .into()
        };

        column![
            heading("Connection"),
            field("Listening port", inp(&t.port, TorrentOpt::Port, 90.0), ""),
            check("Use a random port each start", t.s.random_port, TorrentOpt::RandomPort),
            field("Maximum connections", inp(&t.max_conn, TorrentOpt::MaxConn, 90.0), "all torrents"),
            field("Peers per torrent", inp(&t.max_peers, TorrentOpt::MaxPeers, 90.0), "new torrents"),
            row![
                text("Encryption").size(11.5).color(p.muted).width(Length::Fixed(170.0)),
                pick_list(&TorrentEncryption::ALL[..], Some(t.s.encryption), |e| Message::OptTorrent(TorrentOpt::Encryption(e)))
                    .text_size(12),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            horizontal_rule(1),
            heading("Finding peers"),
            row![
                column![
                    check("DHT (distributed tracker)", t.s.dht, TorrentOpt::Dht),
                    check("Local peer discovery (LSD)", t.s.lsd, TorrentOpt::Lsd),
                    check("Peer exchange (PeX)", t.s.pex, TorrentOpt::Pex),
                ]
                .spacing(5)
                .width(Length::FillPortion(1)),
                column![
                    check("μTP", t.s.utp, TorrentOpt::Utp),
                    check("UPnP port mapping", t.s.upnp, TorrentOpt::Upnp),
                    check("NAT-PMP port mapping", t.s.natpmp, TorrentOpt::NatPmp),
                ]
                .spacing(5)
                .width(Length::FillPortion(1)),
            ],
            horizontal_rule(1),
            heading("Trackers"),
            check("Announce to all trackers and tiers", t.s.announce_to_all_trackers, TorrentOpt::AnnounceAll),
            check("Check HTTPS tracker certificates", t.s.validate_https_trackers, TorrentOpt::ValidateHttps),
            text("Add these trackers to every new torrent (one per line)").size(11).color(p.muted),
            text_editor(&t.trackers)
                .on_action(|a| Message::OptTorrent(TorrentOpt::Trackers(a)))
                .height(Length::Fixed(70.0))
                .size(11.5),
            horizontal_rule(1),
            heading("Speed"),
            field("Download limit", inp(&t.dl, TorrentOpt::Dl, 90.0), "KB/s, 0 = unlimited"),
            field("Upload limit", inp(&t.ul, TorrentOpt::Ul, 90.0), "KB/s, 0 = unlimited"),
            horizontal_rule(1),
            heading("Downloading and seeding"),
            check("Download pieces in order (sequential) for new torrents", t.s.sequential_by_default, TorrentOpt::Sequential),
            check("Keep seeding finished torrents", t.s.seed_finished, TorrentOpt::SeedFinished),
            field("Stop at share ratio", inp(&t.ratio, TorrentOpt::Ratio, 90.0), "0 = no limit"),
            field("Stop after seeding", inp(&t.seed_minutes, TorrentOpt::SeedMinutes, 90.0), "minutes, 0 = no limit"),
            field("Client name (user agent)", inp(&t.s.user_agent, TorrentOpt::UserAgent, 220.0), ""),
        ]
        .spacing(7)
        .into()
    }

    /// The torrent details window.
    pub(super) fn dialog_torrent_details(&self, id: i64) -> Element<'_, Message> {
        let p = self.palette;
        let snap = self.find(id);
        let name = snap.map(|d| d.filename.clone()).unwrap_or_else(|| "Torrent".into());
        let tab = |label: &'static str, t: TorrentTab| {
            button(text(label).size(12))
                .padding([3, 12])
                .on_press(Message::TorrentTab(t))
                .style(flat_button(p, self.torrent_tab == t))
        };
        let tabs = row![
            tab("General", TorrentTab::General),
            tab("Files", TorrentTab::Files),
            tab("Trackers", TorrentTab::Trackers),
            tab("Peers", TorrentTab::Peers),
            tab("Pieces", TorrentTab::Pieces),
        ]
        .spacing(2);

        let details = self.torrent_view.as_ref().filter(|(vid, _)| *vid == id).map(|(_, d)| d);
        let page: Element<'_, Message> = match (details, self.torrent_tab) {
            (None, _) => text("Waiting for libtorrent… (a magnet link shows details once its metadata arrives)")
                .size(12)
                .color(p.muted)
                .into(),
            (Some(d), TorrentTab::General) => self.torrent_general(id, d),
            (Some(d), TorrentTab::Files) => self.torrent_files(d),
            (Some(d), TorrentTab::Trackers) => self.torrent_trackers(d),
            (Some(d), TorrentTab::Peers) => self.torrent_peers(d),
            (Some(d), TorrentTab::Pieces) => self.torrent_pieces(d),
        };

        let body = column![
            tabs,
            horizontal_rule(1),
            container(scrollable(container(page).padding([4, 2]))).height(Length::Fixed(360.0)),
        ]
        .spacing(6);

        let footer = row![
            button(text("Force recheck").size(12)).padding([5, 12]).on_press(Message::TorrentRecheck(id)).style(secondary_button(p)),
            button(text("Reannounce").size(12)).padding([5, 12]).on_press(Message::TorrentReannounce(id)).style(secondary_button(p)),
            button(text("Open folder").size(12)).padding([5, 12]).on_press(Message::OpenFolder).style(secondary_button(p)),
            horizontal_space(),
            button(text("Close").size(12)).padding([5, 18]).on_press(Message::CloseDialog).style(primary_button(p)),
        ]
        .spacing(8);
        self.dialog_frame(format!("Torrent — {}", name), 780.0, body.into(), footer.into())
    }

    fn kv<'a>(&self, k: &'static str, v: String) -> Element<'a, Message> {
        let p = self.palette;
        row![
            text(k).size(11.5).color(p.muted).width(Length::Fixed(150.0)),
            text(v).size(11.5).wrapping(Wrapping::WordOrGlyph),
        ]
        .spacing(8)
        .into()
    }

    fn torrent_general<'a>(&'a self, id: i64, d: &'a TorrentDetails) -> Element<'a, Message> {
        let snap = self.find(id);
        let total = snap.map(|s| s.total_bytes).unwrap_or(0);
        let done = snap.map(|s| s.downloaded_bytes).unwrap_or(0);
        let down = snap.map(|s| s.speed_bytes_per_sec).unwrap_or(0.0);
        let (seeds, peers) = snap.map(|s| (s.seeds, s.peers)).unwrap_or((0, 0));
        column![
            self.piece_bar(d, 16.0),
            self.kv("Size", format::size(total)),
            self.kv("Downloaded", format!("{} ({})", format::bytes(done), format::bytes(d.extra.all_time_download))),
            self.kv("Uploaded", format::bytes(d.extra.all_time_upload)),
            self.kv("Share ratio", format!("{:.2}", d.ratio())),
            self.kv("Download speed", format::rate(down)),
            self.kv("Upload speed", format::rate(d.upload_speed)),
            self.kv("Seeds / peers", format!("{} / {}", seeds, peers)),
            self.kv(
                "Pieces",
                format!("{} of {} × {}", d.extra.pieces_have, d.extra.num_pieces, format::bytes(d.extra.piece_length.max(0) as u64))
            ),
            self.kv("Active for", duration(d.extra.active_seconds)),
            self.kv("Seeding for", duration(d.extra.seeding_seconds)),
            self.kv("Info hash", d.extra.info_hash.clone()),
            self.kv("Save path", d.extra.save_path.clone()),
            checkbox("Download pieces in order (sequential)", d.sequential)
                .on_toggle(move |b| Message::TorrentSequential(id, b))
                .size(14)
                .text_size(11.5),
        ]
        .spacing(4)
        .into()
    }

    fn torrent_files<'a>(&'a self, d: &'a TorrentDetails) -> Element<'a, Message> {
        let p = self.palette;
        if d.files.is_empty() {
            return text("No file list yet.").size(12).color(p.muted).into();
        }
        let mut col = column![row![
            button(text("Download all").size(11)).padding([3, 10]).on_press(Message::TorrentAllFiles(true)).style(secondary_button(p)),
            button(text("Skip all").size(11)).padding([3, 10]).on_press(Message::TorrentAllFiles(false)).style(secondary_button(p)),
            horizontal_space(),
            text(format!("{} files", d.files.len())).size(11).color(p.muted),
        ]
        .spacing(6)
        .align_y(Alignment::Center)]
        .spacing(3);
        for f in &d.files {
            let pct = if f.size == 0 { 100.0 } else { (f.done as f64 / f.size as f64 * 100.0) as f32 };
            let idx = f.index;
            col = col.push(
                row![
                    text(f.path.clone()).size(11.5).wrapping(Wrapping::Glyph).width(Length::Fill),
                    text(format::bytes(f.size)).size(11.5).width(Length::Fixed(80.0)),
                    container(
                        progress_bar(0.0..=100.0, pct).height(Length::Fixed(12.0)).style(move |_t| progress_bar::Style {
                            background: Background::Color(p.track),
                            bar: Background::Color(if f.priority == 0 { p.muted } else { p.accent }),
                            border: Border { color: p.border, width: 0.5, radius: 2.0.into() },
                        })
                    )
                    .width(Length::Fixed(90.0)),
                    text(format!("{:.0}%", pct)).size(11).width(Length::Fixed(38.0)),
                    pick_list(&FilePriority::ALL[..], Some(FilePriority::from_value(f.priority)), move |fp| {
                        Message::TorrentFilePriority(idx, fp)
                    })
                    .text_size(11)
                    .padding([2, 6])
                    .width(Length::Fixed(130.0)),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
            );
        }
        col.into()
    }

    fn torrent_trackers<'a>(&'a self, d: &'a TorrentDetails) -> Element<'a, Message> {
        let p = self.palette;
        let mut col = column![row![
            text_input("Add a tracker URL (udp:// or http(s)://)", &self.tracker_input)
                .on_input(Message::TrackerInput)
                .on_submit(Message::TrackerAdd)
                .padding([4, 8])
                .size(12)
                .style(input_style(p)),
            button(text("Add").size(12)).padding([4, 12]).on_press(Message::TrackerAdd).style(secondary_button(p)),
        ]
        .spacing(6)]
        .spacing(4);
        let h = |s: &'static str, w: Length| text(s).size(11).color(p.muted).width(w);
        col = col.push(
            row![
                h("URL", Length::Fill),
                h("Tier", Length::Fixed(36.0)),
                h("Status", Length::Fixed(90.0)),
                h("Seeds", Length::Fixed(50.0)),
                h("Peers", Length::Fixed(50.0)),
                h("Message", Length::Fixed(170.0)),
            ]
            .spacing(6),
        );
        if d.trackers.is_empty() {
            col = col.push(text("No trackers. DHT and PeX can still find peers.").size(11.5).color(p.muted));
        }
        for t in &d.trackers {
            let (status, color) = match t.state {
                TrackerState::Working => ("Working", p.accent),
                TrackerState::Updating => ("Updating…", p.text),
                TrackerState::Error => ("Not working", p.danger),
                TrackerState::NotContacted => ("Not contacted", p.muted),
            };
            let n = |v: i32| if v < 0 { "–".to_string() } else { v.to_string() };
            col = col.push(
                row![
                    text(t.url.clone()).size(11.5).wrapping(Wrapping::Glyph).width(Length::Fill),
                    text(t.tier.to_string()).size(11.5).width(Length::Fixed(36.0)),
                    text(status).size(11.5).color(color).width(Length::Fixed(90.0)),
                    text(n(t.seeds)).size(11.5).width(Length::Fixed(50.0)),
                    text(n(t.peers)).size(11.5).width(Length::Fixed(50.0)),
                    text(t.message.clone()).size(11).color(p.muted).wrapping(Wrapping::Glyph).width(Length::Fixed(170.0)),
                ]
                .spacing(6),
            );
        }
        col.into()
    }

    fn torrent_peers<'a>(&'a self, d: &'a TorrentDetails) -> Element<'a, Message> {
        let p = self.palette;
        if d.peers.is_empty() {
            return text("Not connected to any peers yet.").size(12).color(p.muted).into();
        }
        let h = |s: &'static str, w: Length| text(s).size(11).color(p.muted).width(w);
        let mut col = column![row![
            h("Address", Length::Fixed(170.0)),
            h("Client", Length::Fill),
            h("Progress", Length::Fixed(70.0)),
            h("Down", Length::Fixed(80.0)),
            h("Up", Length::Fixed(80.0)),
            h("Downloaded", Length::Fixed(80.0)),
        ]
        .spacing(6)]
        .spacing(3);
        let mut peers: Vec<_> = d.peers.iter().collect();
        peers.sort_by(|a, b| b.down_speed.partial_cmp(&a.down_speed).unwrap_or(std::cmp::Ordering::Equal));
        for pe in peers {
            col = col.push(
                row![
                    text(pe.address.clone()).size(11.5).width(Length::Fixed(170.0)),
                    text(if pe.is_seed { format!("{} (seed)", pe.client) } else { pe.client.clone() })
                        .size(11.5)
                        .wrapping(Wrapping::Glyph)
                        .width(Length::Fill),
                    text(format!("{:.0}%", pe.progress * 100.0)).size(11.5).width(Length::Fixed(70.0)),
                    text(format::rate(pe.down_speed)).size(11.5).width(Length::Fixed(80.0)),
                    text(format::rate(pe.up_speed)).size(11.5).width(Length::Fixed(80.0)),
                    text(format::bytes(pe.downloaded)).size(11.5).width(Length::Fixed(80.0)),
                ]
                .spacing(6),
            );
        }
        col.into()
    }

    fn torrent_pieces<'a>(&'a self, d: &'a TorrentDetails) -> Element<'a, Message> {
        let p = self.palette;
        column![
            text(format!(
                "{} of {} pieces ({} each)",
                d.extra.pieces_have,
                d.extra.num_pieces,
                format::bytes(d.extra.piece_length.max(0) as u64)
            ))
            .size(12),
            self.piece_bar(d, 28.0),
            text("Each block shows a group of pieces; darker means more of that group is downloaded.").size(11).color(p.muted),
        ]
        .spacing(8)
        .into()
    }

    /// Downloaded pieces drawn as up to 150 blocks.
    fn piece_bar<'a>(&self, d: &TorrentDetails, height: f32) -> Element<'a, Message> {
        let p = self.palette;
        let n = d.pieces.len();
        if n == 0 {
            return container(text("")).height(Length::Fixed(height)).into();
        }
        let bins = n.min(150);
        let mut bar = row![].spacing(0);
        for b in 0..bins {
            let start = b * n / bins;
            let end = ((b + 1) * n / bins).max(start + 1);
            let have = d.pieces[start..end].iter().filter(|x| **x != 0).count() as f32 / (end - start) as f32;
            let color = if have <= 0.0 {
                p.track
            } else {
                Color { a: 0.35 + 0.65 * have, ..p.accent }
            };
            bar = bar.push(
                container(text(""))
                    .width(Length::FillPortion(1))
                    .height(Length::Fixed(height))
                    .style(move |_t| container::Style { background: Some(Background::Color(color)), ..Default::default() }),
            );
        }
        container(bar)
            .style(move |_t| container::Style {
                border: Border { color: p.border, width: 1.0, radius: 2.0.into() },
                ..Default::default()
            })
            .into()
    }

    /// "Create torrent".
    pub(super) fn dialog_create_torrent(&self) -> Element<'_, Message> {
        let p = self.palette;
        let c = &self.create_form;
        let label = |s: &'static str| text(s).size(11.5).color(p.muted).width(Length::Fixed(130.0));
        let mut body = column![
            row![
                label("File or folder"),
                self.small_input(&c.source, Message::CtSource, 420.0),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            row![label("Save .torrent as"), self.small_input(&c.output, Message::CtOutput, 420.0)]
                .spacing(8)
                .align_y(Alignment::Center),
            row![
                label("Trackers"),
                text("One per line. Each line becomes its own tier.").size(11).color(p.muted),
            ]
            .spacing(8),
            text_editor(&c.trackers)
                .placeholder("udp://tracker.example.org:6969/announce")
                .on_action(Message::CtTrackers)
                .height(Length::Fixed(80.0))
                .size(11.5),
            row![label("Web seeds"), self.small_input(&c.web_seeds, Message::CtWebSeeds, 420.0)]
                .spacing(8)
                .align_y(Alignment::Center),
            row![label("Comment"), self.small_input(&c.comment, Message::CtComment, 420.0)]
                .spacing(8)
                .align_y(Alignment::Center),
            row![
                label("Piece size"),
                pick_list(&PieceSize::ALL[..], Some(c.piece), Message::CtPiece).text_size(12),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            checkbox("Private torrent (no DHT or peer exchange)", c.private)
                .on_toggle(Message::CtPrivate)
                .size(14)
                .text_size(11.5),
            checkbox("Start seeding it right away", c.seed).on_toggle(Message::CtSeed).size(14).text_size(11.5),
        ]
        .spacing(8);
        if c.busy {
            body = body.push(text("Hashing files… this can take a while for large folders.").size(12).color(p.text));
        }
        if let Some(e) = &c.error {
            body = body.push(text(e.clone()).size(12).color(p.danger));
        }
        if let Some(done) = &c.done {
            body = body.push(text(format!("Created {}", done)).size(12).color(p.text));
        }
        let ready = !c.busy && !c.source.trim().is_empty() && !c.output.trim().is_empty();
        let footer = row![
            horizontal_space(),
            button(text("Close").size(12)).padding([5, 16]).on_press(Message::CloseDialog).style(secondary_button(p)),
            button(text(if c.busy { "Creating…" } else { "Create" }).size(12))
                .padding([5, 18])
                .on_press_maybe(ready.then_some(Message::CtSubmit))
                .style(primary_button(p)),
        ]
        .spacing(8);
        self.dialog_frame("Create Torrent".into(), 640.0, body.into(), footer.into())
    }
}
