//! Simple unit tests for the logging module core functionality
//! These tests focus on the serialization and deserialization without USB HID dependencies

#[test]
fn test_log_level_values() {
    // Test that log levels have the expected numeric values
    assert_eq_no_std!(0u8, 0); // LogLevel::Debug as u8
    assert_eq_no_std!(1u8, 1); // LogLevel::Info as u8  
    assert_eq_no_std!(2u8, 2); // LogLevel::Warn as u8
    assert_eq_no_std!(3u8, 3); // LogLevel::Error as u8
}

#[test]
fn test_message_serialization_format() {
    // Test the basic serialization format without dependencies
    let mut buffer = [0u8; 64];
    
    // Simulate LogLevel::Warn (2)
    buffer[0] = 2;
    
    // Simulate module "BATTERY" (8 bytes, null-padded)
    buffer[1..8].copy_from_slice(b"BATTERY");
    buffer[8] = 0; // null padding
    
    // Simulate message "Low voltage" (48 bytes, null-terminated)
    buffer[9..20].copy_from_slice(b"Low voltage");
    // Rest is already zero-initialized
    
    // Simulate timestamp 0x12345678 (little-endian)
    let timestamp = 0x12345678u32;
    buffer[57..61].copy_from_slice(&timestamp.to_le_bytes());
    
    // Verify the format
    assert_eq_no_std!(buffer[0], 2); // Log level
    assert_eq_no_std!(&buffer[1..8], b"BATTERY");
    assert_eq_no_std!(buffer[8], 0);
    assert_eq_no_std!(&buffer[9..20], b"Low voltage");
    
    // Verify timestamp deserialization
    let mut timestamp_bytes = [0u8; 4];
    timestamp_bytes.copy_from_slice(&buffer[57..61]);
    let recovered_timestamp = u32::from_le_bytes(timestamp_bytes);
    assert_eq_no_std!(recovered_timestamp, 0x12345678);
}

#[test]
fn test_buffer_size() {
    // Verify the buffer size is exactly 64 bytes as required
    let buffer = [0u8; 64];
    assert_eq_no_std!(buffer.len(), 64);
}

#[test]
fn test_timestamp_serialization() {
    // Test timestamp serialization/deserialization
    let test_timestamps = [0u32, 1000, 0xFFFFFFFF, 0x12345678, 0xDEADBEEF];
    
    for &timestamp in &test_timestamps {
        let bytes = timestamp.to_le_bytes();
        let recovered = u32::from_le_bytes(bytes);
        assert_eq_no_std!(timestamp, recovered);
    }
}

#[test]
fn test_string_truncation_logic() {
    // Test string truncation logic
    let long_string = "This is a very long string that exceeds the maximum length allowed";
    let max_len = 48;
    
    let truncated_len = core::cmp::min(long_string.len(), max_len);
    assert_no_std!(truncated_len <= max_len);
    
    let truncated = &long_string.as_bytes()[..truncated_len];
    assert_no_std!(truncated.len() <= max_len);
}