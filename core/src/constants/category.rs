use crate::define_code_value_enum;

define_code_value_enum! {
    /// Sidebar UI Category Navigation code and value pairs
    pub enum CategoryCode(i32, &'static str) {
        All(0, "All"),
        Downloading(1, "Downloading"),
        Queued(2, "Queued"),
        Completed(3, "Completed"),
        Paused(4, "Paused"),
        Failed(5, "Failed"),
        Torrents(6, "Torrents"),
        Settings(7, "Settings"),
    }
}
