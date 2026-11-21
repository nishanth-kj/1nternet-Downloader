// Fast C low-level primitives bridge
// Main logic stays purely in Rust; performance-critical block operations
// (CRC32, block zero scanning) leverage C routines.

extern "C" {
    fn idr_fast_crc32(crc: u32, buffer: *const u8, length: usize) -> u32;
    fn idr_fast_is_all_zero(buffer: *const u8, length: usize) -> i32;
}

/// Compute fast CRC32 checksum over memory slice using C lookup
pub fn fast_crc32(buffer: &[u8]) -> u32 {
    unsafe { idr_fast_crc32(0, buffer.as_ptr(), buffer.len()) }
}

/// Check if a memory buffer is entirely zero (for sparse file validation)
pub fn fast_is_all_zero(buffer: &[u8]) -> bool {
    unsafe { idr_fast_is_all_zero(buffer.as_ptr(), buffer.len()) != 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fast_crc32() {
        let data = b"123456789";
        let crc = fast_crc32(data);
        assert_eq!(crc, 0xCBF43926);
    }

    #[test]
    fn test_fast_is_all_zero() {
        let zeroes = vec![0u8; 1024];
        assert!(fast_is_all_zero(&zeroes));

        let mut non_zero = vec![0u8; 1024];
        non_zero[500] = 1;
        assert!(!fast_is_all_zero(&non_zero));
    }
}
