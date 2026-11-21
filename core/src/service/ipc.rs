/// Service trait defining inter-process communication contracts
pub trait IpcService: Send + Sync {
    fn decode_idr_url(&self, raw: &str) -> Option<String>;
    fn another_instance_is_running(&self) -> bool;
    fn send_url_to_running_instance(&self, url: &str) -> bool;
}
