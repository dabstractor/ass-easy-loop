#!/usr/bin/env python3
"""
Test script for hidlog.py functionality
"""

import struct
import sys
import os
# Add utilities directory to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'utilities'))
from hidlog import HidLogReceiver, LogMessage, LogLevel


def create_test_hid_report(level: LogLevel, module: str, message: str, timestamp: int) -> bytes:
    """Create a test HID report with the expected format"""
    data = bytearray(64)
    
    # Byte 0: Log level
    data[0] = level.value
    
    # Bytes 1-8: Module name (null-padded)
    module_bytes = module.encode('utf-8')[:8]
    data[1:1+len(module_bytes)] = module_bytes
    
    # Bytes 9-56: Message content (null-terminated)
    message_bytes = message.encode('utf-8')[:47]  # Leave room for null terminator
    data[9:9+len(message_bytes)] = message_bytes
    data[9+len(message_bytes)] = 0  # Null terminator
    
    # Bytes 57-60: Timestamp (little-endian u32)
    data[57:61] = struct.pack('<I', timestamp)
    
    return bytes(data)


def test_log_message_parsing():
    """Test log message parsing functionality"""
    print("Testing log message parsing...")
    
    receiver = HidLogReceiver()
    
    # Test cases
    test_cases = [
        (LogLevel.INFO, "BATTERY", "State changed: Normal -> Charging (ADC: 1680)", 12345),
        (LogLevel.ERROR, "PEMF", "Pulse timing deviation detected: +2.1ms", 67890),
        (LogLevel.DEBUG, "SYSTEM", "Task execution time: 0.5ms", 11111),
        (LogLevel.WARN, "USB", "Queue 75% full, consider reducing log verbosity", 22222),
    ]
    
    for level, module, message, timestamp in test_cases:
        # Create test HID report
        hid_data = create_test_hid_report(level, module, message, timestamp)
        
        # Parse the message
        parsed = receiver.parse_log_message(hid_data)
        
        if parsed:
            print(f"✓ Parsed: {parsed}")
            
            # Verify parsed data matches input
            assert parsed.level == level, f"Level mismatch: {parsed.level} != {level}"
            assert parsed.module == module, f"Module mismatch: '{parsed.module}' != '{module}'"
            assert parsed.message == message, f"Message mismatch: '{parsed.message}' != '{message}'"
            assert parsed.timestamp == timestamp, f"Timestamp mismatch: {parsed.timestamp} != {timestamp}"
        else:
            print(f"✗ Failed to parse message for {level} {module}")
            return False
    
    print("✓ All parsing tests passed!")
    return True


def test_message_formatting():
    """Test log message formatting"""
    print("\nTesting message formatting...")
    
    # Create test message
    msg = LogMessage(12345, LogLevel.INFO, "BATTERY", "Test message")
    
    # Test string representation
    formatted = str(msg)
    print(f"Formatted message: {formatted}")
    
    # Verify format contains expected components
    assert "[012.345s]" in formatted, "Timestamp not formatted correctly"
    assert "[INFO ]" in formatted, "Log level not formatted correctly"
    assert "[BATTERY ]" in formatted, "Module not formatted correctly"
    assert "Test message" in formatted, "Message content missing"
    
    # Test JSON output
    json_data = msg.to_json()
    assert json_data['timestamp'] == 12345, "JSON timestamp incorrect"
    assert json_data['level'] == 'INFO', "JSON level incorrect"
    assert json_data['module'] == 'BATTERY', "JSON module incorrect"
    assert json_data['message'] == 'Test message', "JSON message incorrect"
    assert 'timestamp_formatted' in json_data, "JSON missing formatted timestamp"
    assert 'received_at' in json_data, "JSON missing received_at timestamp"
    
    print("✓ Message formatting test passed!")
    return True


def test_edge_cases():
    """Test edge cases and error handling"""
    print("\nTesting edge cases...")
    
    receiver = HidLogReceiver()
    
    # Test with invalid data
    invalid_data = b'\x00' * 32  # Too short
    parsed = receiver.parse_log_message(invalid_data)
    assert parsed is None, "Should return None for invalid data"
    
    # Test with invalid log level
    invalid_level_data = create_test_hid_report(LogLevel.INFO, "TEST", "Invalid level", 1000)
    invalid_level_data = bytearray(invalid_level_data)
    invalid_level_data[0] = 99  # Invalid level
    parsed = receiver.parse_log_message(bytes(invalid_level_data))
    assert parsed is None, "Should return None for invalid log level"
    
    # Test with very long module name (should be truncated)
    long_module_data = create_test_hid_report(LogLevel.INFO, "VERYLONGMODULENAME", "Test", 1000)
    parsed = receiver.parse_log_message(long_module_data)
    assert parsed is not None, "Should handle long module names"
    assert len(parsed.module) <= 8, "Module name should be truncated"
    
    # Test with very long message (should be truncated)
    long_message = "A" * 100  # Much longer than 47 character limit
    long_msg_data = create_test_hid_report(LogLevel.INFO, "TEST", long_message, 1000)
    parsed = receiver.parse_log_message(long_msg_data)
    assert parsed is not None, "Should handle long messages"
    assert len(parsed.message) <= 47, "Message should be truncated"
    
    print("✓ Edge case tests passed!")
    return True


def main():
    """Run all tests"""
    print("Running hidlog.py tests...\n")
    
    success = True
    success &= test_log_message_parsing()
    success &= test_message_formatting()
    success &= test_edge_cases()
    
    if success:
        print("\n✓ All tests passed!")
        return 0
    else:
        print("\n✗ Some tests failed!")
        return 1


if __name__ == '__main__':
    exit(main())