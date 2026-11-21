use crate::service::fast_c::FastOpsService;

extern "C" {
    fn idr_fast_crc32(crc: u32, data: *const u8, len: usize) -> u32;
    fn idr_fast_is_all_zero(data: *const u8, len: usize) -> i32;
}

/// Compute fast CRC-32 using low-level C routines
pub fn fast_crc32(data: &[u8]) -> u32 {
    unsafe { idr_fast_crc32(0, data.as_ptr(), data.len()) }
}

/// Fast branchless AVX2/SSE2/SWAR zero checking via C primitive
pub fn fast_is_all_zero(data: &[u8]) -> bool {
    unsafe { idr_fast_is_all_zero(data.as_ptr(), data.len()) == 1 }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct FastCOps;

impl FastOpsService for FastCOps {
    fn crc32(&self, data: &[u8]) -> u32 {
        fast_crc32(data)
    }

    fn is_all_zero(&self, data: &[u8]) -> bool {
        fast_is_all_zero(data)
    }
}
