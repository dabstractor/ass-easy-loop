#!/usr/bin/env python3
"""
Test JSON output functionality for hidlog.py
"""

import json
import sys
import os
# Add utilities directory to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'utilities'))
from hidlog import LogMessage, LogLevel


def test_json_output():
    """Test JSON output format"""
    print("Testing JSON output format...")
    
    # Create test message
    msg = LogMessage(12345, LogLevel.ERROR, "PEMF", "Pulse timing deviation detected")
    
    # Get JSON representation
    json_data = msg.to_json()
    
    # Verify JSON structure
    expected_keys = ['timestamp', 'timestamp_formatted', 'level', 'module', 'message', 'received_at']
    for key in expected_keys:
        assert key in json_data, f"Missing key: {key}"
    
    # Verify values
    assert json_data['timestamp'] == 12345
    assert json_data['level'] == 'ERROR'
    assert json_data['module'] == 'PEMF'
    assert json_data['message'] == 'Pulse timing deviation detected'
    assert json_data['timestamp_formatted'] == '012.345s'
    
    # Verify JSON serialization works
    json_str = json.dumps(json_data)
    parsed_back = json.loads(json_str)
    assert parsed_back == json_data, "JSON serialization/deserialization failed"
    
    print(f"✓ JSON output: {json_str}")
    print("✓ JSON output test passed!")
    return True


if __name__ == '__main__':
    success = test_json_output()
    if success:
        print("\n✓ All JSON tests passed!")
        exit(0)
    else:
        print("\n✗ JSON tests failed!")
        exit(1)