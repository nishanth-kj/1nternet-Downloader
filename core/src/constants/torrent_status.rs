use crate::define_code_value_enum;

define_code_value_enum! {
    /// BitTorrent Task Status code and value pairs
    pub enum TorrentStatus(i32, &'static str) {
        Queued(0, "Queued"),
        Downloading(1, "Downloading"),
        Paused(2, "Paused"),
        Completed(3, "Completed"),
        Seeding(4, "Seeding"),
        Error(5, "Error"),
    }
}

pub type TorrentStatusCode = TorrentStatus;
