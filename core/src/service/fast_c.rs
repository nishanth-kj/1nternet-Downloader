/// Service trait defining high-performance native block operations
pub trait FastOpsService: Send + Sync {
    fn crc32(&self, data: &[u8]) -> u32;
    fn is_all_zero(&self, data: &[u8]) -> bool;
}
