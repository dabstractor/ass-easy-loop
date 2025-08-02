#!/usr/bin/env python3
"""
Basic CI Integration Test

Simple test to verify CI integration functionality without complex dependencies.
"""

import os
import sys
import tempfile
import json
from pathlib import Path

# Add the parent directory to the path so we can import test_framework
sys.path.insert(0, str(Path(__file__).parent.parent))

def test_ci_environment_detection():
    """Test CI environment detection"""
    print("Testing CI environment detection...")
    
    from test_framework.ci_integration import CIIntegration
    
    ci = CIIntegration(output_dir=tempfile.mkdtemp(), verbose=False)
    
    # Test with GitHub Actions environment
    original_env = dict(os.environ)
    try:
        os.environ.update({
            'GITHUB_ACTIONS': 'true',
            'GITHUB_RUN_NUMBER': '123',
            'GITHUB_REF_NAME': 'main',
            'GITHUB_SHA': 'abc123'
        })
        
        env_info = ci.detect_ci_environment()
        assert env_info.ci_system == 'github_actions'
        assert env_info.build_number == '123'
        assert env_info.branch_name == 'main'
        assert env_info.commit_hash == 'abc123'
        print("✓ GitHub Actions detection works")
        
    finally:
        os.environ.clear()
        os.environ.update(original_env)

def test_default_configuration():
    """Test default CI configuration"""
    print("Testing default configuration...")
    
    from test_framework.ci_integration import CIIntegration
    
    ci = CIIntegration(output_dir=tempfile.mkdtemp(), verbose=False)
    config_data = ci.get_default_ci_configuration()
    
    assert 'test_config' in config_data
    assert 'required_devices' in config_data
    assert 'max_parallel_devices' in config_data
    
    test_config = config_data['test_config']
    assert 'name' in test_config
    assert 'steps' in test_config
    assert len(test_config['steps']) > 0
    
    print("✓ Default configuration is valid")

def test_configuration_loading():
    """Test configuration loading from file"""
    print("Testing configuration loading...")
    
    from test_framework.ci_integration import CIIntegration
    
    # Create test configuration
    test_config = {
        "test_config": {
            "name": "Test CI Config",
            "description": "Test configuration",
            "steps": [
                {
                    "name": "basic_test",
                    "test_type": "USB_COMMUNICATION_TEST",
                    "parameters": {"message_count": 1},
                    "timeout": 5.0,
                    "required": True
                }
            ],
            "parallel_execution": False,
            "global_timeout": 60.0
        },
        "required_devices": 1,
        "max_parallel_devices": 1,
        "timeout_seconds": 120.0
    }
    
    # Write to temporary file
    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        json.dump(test_config, f)
        config_file = f.name
    
    try:
        ci = CIIntegration(output_dir=tempfile.mkdtemp(), verbose=False)
        loaded_config = ci.load_ci_configuration(config_file)
        
        assert loaded_config.required_devices == 1
        assert loaded_config.max_parallel_devices == 1
        assert loaded_config.timeout_seconds == 120.0
        assert loaded_config.test_config.name == "Test CI Config"
        
        print("✓ Configuration loading works")
        
    finally:
        os.unlink(config_file)

def test_exit_code_mapping():
    """Test exit code mapping"""
    print("Testing exit code mapping...")
    
    from test_framework.ci_integration import CITestResult, CIEnvironmentInfo
    
    env_info = CIEnvironmentInfo(
        ci_system="test", build_number=None, branch_name=None,
        commit_hash=None, pull_request=None, workspace_path="/test",
        environment_variables={}
    )
    
    # Test success
    result = CITestResult(
        success=True, exit_code=0, total_tests=5, passed_tests=5,
        failed_tests=0, skipped_tests=0, duration_seconds=30.0,
        devices_tested=["TEST_DEVICE"], environment_info=env_info,
        artifacts_generated=[], error_summary=None
    )
    assert result.exit_code == 0
    
    # Test failure
    result = CITestResult(
        success=False, exit_code=1, total_tests=5, passed_tests=3,
        failed_tests=2, skipped_tests=0, duration_seconds=30.0,
        devices_tested=["TEST_DEVICE"], environment_info=env_info,
        artifacts_generated=[], error_summary="Test failures"
    )
    assert result.exit_code == 1
    
    print("✓ Exit code mapping works")

def test_factory_function():
    """Test factory function"""
    print("Testing factory function...")
    
    from test_framework.ci_integration import create_ci_integration, CIIntegration
    
    temp_dir = tempfile.mkdtemp()
    ci = create_ci_integration(output_dir=temp_dir, verbose=True)
    
    assert isinstance(ci, CIIntegration)
    assert str(ci.output_dir) == temp_dir
    assert ci.verbose == True
    
    print("✓ Factory function works")

def main():
    """Run basic CI integration tests"""
    print("Running basic CI integration tests...")
    print("=" * 50)
    
    try:
        test_ci_environment_detection()
        test_default_configuration()
        test_configuration_loading()
        test_exit_code_mapping()
        test_factory_function()
        
        print("=" * 50)
        print("✅ All basic tests passed!")
        return 0
        
    except Exception as e:
        print(f"❌ Test failed: {e}")
        import traceback
        traceback.print_exc()
        return 1

if __name__ == '__main__':
    sys.exit(main())