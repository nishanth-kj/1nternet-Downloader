use idm_lib::service::{fast_crc32, fast_is_all_zero};

#[test]
fn test_fast_crc32() {
    let input = b"123456789";
    let crc = fast_crc32(input);
    assert_eq!(crc, 0xCBF43926);
}

#[test]
fn test_fast_is_all_zero() {
    let zeroes = vec![0u8; 1024];
    assert!(fast_is_all_zero(&zeroes));

    let mut non_zeroes = vec![0u8; 1024];
    non_zeroes[512] = 1;
    assert!(!fast_is_all_zero(&non_zeroes));
}
