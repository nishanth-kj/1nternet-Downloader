use crate::define_code_value_enum;

define_code_value_enum! {
    /// Generic status code and string value pairs
    pub enum Status(i32, &'static str) {
        Idle(0, "Idle"),
        Pending(1, "Pending"),
        Running(2, "Running"),
        Paused(3, "Paused"),
        Completed(4, "Completed"),
        Failed(5, "Failed"),
        Cancelled(6, "Cancelled"),
    }
}

pub type StatusCode = Status;
