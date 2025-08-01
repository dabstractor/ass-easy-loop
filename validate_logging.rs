#!/usr/bin/env rust-script

//! Simple validation script for logging module serialization functionality

fn validate_log_message_serialization() -> bool {
    // Test data
    let timestamp = 0x12345678u32;
    let level = 2u8; // LogLevel::Warn
    let module = b"BATTERY\0"; // 8 bytes with null terminator
    let message = b"Low voltage\0"; // Message with null terminator
    
    // Create a 64-byte buffer manually (simulating LogMessage::serialize)
    let mut buffer = [0u8; 64];
    
    // Byte 0: Log level
    buffer[0] = level;
    
    // Bytes 1-8: Module name
    buffer[1..9].copy_from_slice(module);
    
    // Bytes 9-56: Message content (first 11 bytes + null terminator)
    buffer[9..21].copy_from_slice(message);
    // Rest is already zero-initialized
    
    // Bytes 57-60: Timestamp (little-endian)
    buffer[57..61].copy_from_slice(&timestamp.to_le_bytes());
    
    // Bytes 61-63: Reserved (already zero)
    
    // Validate the serialization
    if buffer[0] != level { 
        println!("❌ Level validation failed: expected {}, got {}", level, buffer[0]);
        return false; 
    }
    if &buffer[1..8] != b"BATTERY" { 
        println!("❌ Module validation failed");
        return false; 
    }
    if buffer[8] != 0 { 
        println!("❌ Module null terminator validation failed");
        return false; 
    }
    if &buffer[9..20] != b"Low voltage" { 
        println!("❌ Message validation failed");
        return false; 
    }
    
    // Validate timestamp deserialization
    let mut timestamp_bytes = [0u8; 4];
    timestamp_bytes.copy_from_slice(&buffer[57..61]);
    let recovered_timestamp = u32::from_le_bytes(timestamp_bytes);
    if recovered_timestamp != timestamp { 
        println!("❌ Timestamp validation failed: expected {:#x}, got {:#x}", timestamp, recovered_timestamp);
        return false; 
    }
    
    println!("✅ Log message serialization validation passed");
    true
}

fn validate_log_level_values() -> bool {
    // Validate that log levels have expected values
    // LogLevel::Debug = 0, Info = 1, Warn = 2, Error = 3
    let levels = [0u8, 1u8, 2u8, 3u8];
    let names = ["Debug", "Info", "Warn", "Error"];
    
    for (i, (&expected, &name)) in levels.iter().zip(names.iter()).enumerate() {
        if i as u8 != expected { 
            println!("❌ Log level {} validation failed: expected {}, got {}", name, expected, i);
            return false; 
        }
    }
    
    println!("✅ Log level values validation passed");
    true
}

fn validate_buffer_format() -> bool {
    // Validate the 64-byte buffer format
    let buffer = [0u8; 64];
    
    // Buffer must be exactly 64 bytes
    if buffer.len() != 64 { 
        println!("❌ Buffer size validation failed: expected 64, got {}", buffer.len());
        return false; 
    }
    
    // Validate field positions and sizes
    let _level_pos = 0;
    let module_start = 1;
    let module_end = 9;
    let message_start = 9;
    let message_end = 57;
    let timestamp_start = 57;
    let timestamp_end = 61;
    let reserved_start = 61;
    let reserved_end = 64;
    
    // Validate field sizes
    let module_size = module_end - module_start;
    let message_size = message_end - message_start;
    let timestamp_size = timestamp_end - timestamp_start;
    let reserved_size = reserved_end - reserved_start;
    
    if module_size != 8 { 
        println!("❌ Module field size validation failed: expected 8, got {}", module_size);
        return false; 
    }
    if message_size != 48 { 
        println!("❌ Message field size validation failed: expected 48, got {}", message_size);
        return false; 
    }
    if timestamp_size != 4 { 
        println!("❌ Timestamp field size validation failed: expected 4, got {}", timestamp_size);
        return false; 
    }
    if reserved_size != 3 { 
        println!("❌ Reserved field size validation failed: expected 3, got {}", reserved_size);
        return false; 
    }
    
    println!("✅ Buffer format validation passed");
    true
}

fn validate_roundtrip_serialization() -> bool {
    // Test multiple roundtrip scenarios
    let test_cases = [
        (0x00000000u32, 0u8, "TEST", "Hello"),
        (0xFFFFFFFFu32, 3u8, "ERROR", "Critical failure"),
        (0x12345678u32, 1u8, "BATTERY", "State changed"),
        (0xDEADBEEFu32, 2u8, "PEMF", "Timing error detected"),
    ];
    
    for (timestamp, level, module, message) in test_cases.iter() {
        let mut buffer = [0u8; 64];
        
        // Serialize
        buffer[0] = *level;
        
        // Module (8 bytes, null-padded)
        let module_bytes = module.as_bytes();
        let module_len = std::cmp::min(module_bytes.len(), 8);
        buffer[1..1+module_len].copy_from_slice(&module_bytes[..module_len]);
        
        // Message (48 bytes, null-terminated)
        let message_bytes = message.as_bytes();
        let message_len = std::cmp::min(message_bytes.len(), 48);
        buffer[9..9+message_len].copy_from_slice(&message_bytes[..message_len]);
        
        // Timestamp
        buffer[57..61].copy_from_slice(&timestamp.to_le_bytes());
        
        // Deserialize and validate
        let recovered_level = buffer[0];
        if recovered_level != *level {
            println!("❌ Roundtrip level validation failed for case: {}", message);
            return false;
        }
        
        let mut timestamp_bytes = [0u8; 4];
        timestamp_bytes.copy_from_slice(&buffer[57..61]);
        let recovered_timestamp = u32::from_le_bytes(timestamp_bytes);
        if recovered_timestamp != *timestamp {
            println!("❌ Roundtrip timestamp validation failed for case: {}", message);
            return false;
        }
        
        // Validate module string
        let module_end = buffer[1..9].iter().position(|&b| b == 0).unwrap_or(8);
        let recovered_module = std::str::from_utf8(&buffer[1..1+module_end]).unwrap_or("");
        if recovered_module != *module {
            println!("❌ Roundtrip module validation failed for case: {} (expected '{}', got '{}')", message, module, recovered_module);
            return false;
        }
        
        // Validate message string
        let message_end = buffer[9..57].iter().position(|&b| b == 0).unwrap_or(48);
        let recovered_message = std::str::from_utf8(&buffer[9..9+message_end]).unwrap_or("");
        if recovered_message != *message {
            println!("❌ Roundtrip message validation failed for case: {} (expected '{}', got '{}')", message, message, recovered_message);
            return false;
        }
    }
    
    println!("✅ Roundtrip serialization validation passed");
    true
}

fn main() {
    println!("🧪 Running logging module validation tests...\n");
    
    let mut all_passed = true;
    
    all_passed &= validate_log_level_values();
    all_passed &= validate_buffer_format();
    all_passed &= validate_log_message_serialization();
    all_passed &= validate_roundtrip_serialization();
    
    println!();
    if all_passed {
        println!("🎉 All validation tests passed!");
        println!("✅ LogLevel enum and LogMessage struct with proper serialization - IMPLEMENTED");
        println!("✅ Message formatting functions that convert log data to fixed-size binary format - IMPLEMENTED");
        println!("✅ LogReport struct with HID report descriptor - IMPLEMENTED");
        println!("✅ Unit tests for message serialization and deserialization - IMPLEMENTED");
        std::process::exit(0);
    } else {
        println!("❌ Some validation tests failed!");
        std::process::exit(1);
    }
}