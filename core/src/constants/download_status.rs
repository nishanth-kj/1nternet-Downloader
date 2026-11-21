use crate::define_code_value_enum;

define_code_value_enum! {
    /// Download Status code and value pairs
    pub enum DownloadStatus(i32, &'static str) {
        Queued(0, "Queued"),
        Downloading(1, "Downloading"),
        Paused(2, "Paused"),
        Completed(3, "Completed"),
        Error(4, "Error"),
    }
}

pub type DownloadStatusCode = DownloadStatus;
