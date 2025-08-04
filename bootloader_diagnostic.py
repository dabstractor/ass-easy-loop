#!/usr/bin/env python3
"""
Bootloader entry diagnostic tool

This script sends a bootloader command and monitors the detailed log output
to identify exactly where the bootloader entry process is failing.
"""

import hid
import time
import struct
import threading
from datetime import datetime

class BootloaderDiagnostic:
    def __init__(self):
        self.device = None
        self.logging = False
        self.log_messages = []
        self.log_thread = None
        
    def start_logging(self):
        """Start logging HID messages"""
        try:
            self.device = hid.Device(0x1234, 0x5678)
            print("✓ Connected to device for logging")
            
            self.logging = True
            self.log_messages = []
            self.log_thread = threading.Thread(target=self._log_loop, daemon=True)
            self.log_thread.start()
            
            print("✓ HID logging started")
            return True
            
        except Exception as e:
            print(f"✗ Failed to start logging: {e}")
            return False
    
    def _log_loop(self):
        """HID logging loop"""
        message_count = 0
        
        while self.logging:
            try:
                data = self.device.read(64, timeout=100)
                
                if data:
                    message_count += 1
                    timestamp = datetime.now().strftime("%H:%M:%S.%f")[:-3]
                    
                    try:
                        text = bytes(data).decode('utf-8').rstrip('\x00')
                        if text:
                            log_entry = f"[{timestamp}] #{message_count:04d}: {text}"
                            print(log_entry)
                            self.log_messages.append(log_entry)
                    except UnicodeDecodeError:
                        hex_data = ' '.join(f'{b:02x}' for b in data[:16])
                        log_entry = f"[{timestamp}] #{message_count:04d}: RAW: {hex_data}..."
                        print(log_entry)
                        self.log_messages.append(log_entry)
                        
            except Exception as e:
                if self.logging and "Success" not in str(e):
                    # Only log non-timeout errors
                    if message_count % 50 == 0:  # Reduce spam
                        print(f"[DEBUG] HID read: {e}")
    
    def stop_logging(self):
        """Stop HID logging"""
        if self.logging:
            self.logging = False
            if self.log_thread:
                self.log_thread.join(timeout=2)
            
            if self.device:
                try:
                    self.device.close()
                except:
                    pass
                self.device = None
                
            print("✓ HID logging stopped")
    
    def send_bootloader_command(self):
        """Send bootloader command and return success"""
        try:
            # Use a separate device connection for command sending
            cmd_device = hid.Device(0x1234, 0x5678)
            print("✓ Connected to device for command")
            
            # Create bootloader command
            command = bytearray(64)
            command[0] = 0x80  # ENTER_BOOTLOADER command type
            command[1] = 0x01  # Command ID
            
            # Payload: timeout as 4-byte little-endian integer
            timeout_ms = 5000
            payload = struct.pack('<I', timeout_ms)
            
            command[2] = len(payload)  # Payload length
            
            # Calculate XOR checksum
            checksum = command[0] ^ command[1] ^ command[2]
            for byte in payload:
                checksum ^= byte
            command[3] = checksum
            
            # Add payload
            command[4:4+len(payload)] = payload
            
            print(f"Sending bootloader command (timeout={timeout_ms}ms)...")
            
            # Send command
            bytes_sent = cmd_device.write(bytes(command))
            print(f"✓ Sent {bytes_sent} bytes")
            
            cmd_device.close()
            return True
            
        except Exception as e:
            print(f"✗ Failed to send command: {e}")
            return False
    
    def analyze_bootloader_logs(self):
        """Analyze the collected logs to identify failure points"""
        print("\n=== BOOTLOADER ENTRY ANALYSIS ===")
        
        if not self.log_messages:
            print("✗ No log messages collected")
            return
        
        print(f"Collected {len(self.log_messages)} log messages")
        
        # Look for specific bootloader entry stages
        stages = {
            'command_received': False,
            'task_started': False,
            'entry_requested': False,
            'hardware_validation': False,
            'task_shutdown': False,
            'ready_for_bootloader': False,
            'entering_bootloader': False,
            'entry_failed': False,
            'entry_timeout': False
        }
        
        failure_reasons = []
        
        for message in self.log_messages:
            msg_lower = message.lower()
            
            if 'bootloader entry command received' in msg_lower:
                stages['command_received'] = True
            
            if 'bootloader entry task started' in msg_lower:
                stages['task_started'] = True
            
            if 'bootloader entry request accepted' in msg_lower:
                stages['entry_requested'] = True
            
            if 'hardware state validation' in msg_lower:
                stages['hardware_validation'] = True
            
            if 'task shutdown' in msg_lower:
                stages['task_shutdown'] = True
            
            if 'ready for bootloader' in msg_lower:
                stages['ready_for_bootloader'] = True
            
            if 'entering bootloader mode' in msg_lower:
                stages['entering_bootloader'] = True
            
            if 'bootloader entry failed' in msg_lower or 'entry failed' in msg_lower:
                stages['entry_failed'] = True
                failure_reasons.append(message)
            
            if 'bootloader entry timed out' in msg_lower or 'timed out' in msg_lower:
                stages['entry_timeout'] = True
                failure_reasons.append(message)
            
            if 'resetting bootloader entry state' in msg_lower:
                failure_reasons.append(message)
        
        # Report stage completion
        print("\nBootloader Entry Stages:")
        for stage, completed in stages.items():
            status = "✓" if completed else "✗"
            print(f"  {status} {stage.replace('_', ' ').title()}: {completed}")
        
        # Identify failure point
        if stages['entering_bootloader']:
            print("\n🎉 SUCCESS: Bootloader entry completed successfully!")
        elif stages['entry_failed'] or stages['entry_timeout']:
            print("\n❌ FAILURE: Bootloader entry failed")
            if failure_reasons:
                print("Failure reasons:")
                for reason in failure_reasons:
                    print(f"  - {reason}")
        elif not stages['command_received']:
            print("\n❌ FAILURE: Command not received by firmware")
        elif not stages['task_started']:
            print("\n❌ FAILURE: Bootloader entry task not started")
        elif not stages['entry_requested']:
            print("\n❌ FAILURE: Bootloader entry request failed")
        elif not stages['hardware_validation']:
            print("\n❌ FAILURE: Hardware validation not started")
        elif not stages['task_shutdown']:
            print("\n❌ FAILURE: Task shutdown not initiated")
        elif not stages['ready_for_bootloader']:
            print("\n❌ FAILURE: Never reached ready state")
        else:
            print("\n⚠ UNCLEAR: Process started but didn't complete")
        
        # Show relevant log messages
        print("\nRelevant log messages:")
        bootloader_messages = [msg for msg in self.log_messages 
                             if any(word in msg.lower() for word in 
                                   ['bootloader', 'entry', 'shutdown', 'hardware', 'task', 'ready'])]
        
        for msg in bootloader_messages[-10:]:  # Show last 10 relevant messages
            print(f"  {msg}")
    
    def run_diagnostic(self):
        """Run complete bootloader diagnostic"""
        print("=== BOOTLOADER ENTRY DIAGNOSTIC ===")
        print("This will monitor the bootloader entry process in detail")
        print()
        
        try:
            # Step 1: Start logging
            if not self.start_logging():
                return False
            
            # Step 2: Wait a moment for initial logs
            print("Collecting initial system logs...")
            time.sleep(2)
            
            # Step 3: Send bootloader command
            if not self.send_bootloader_command():
                return False
            
            # Step 4: Monitor for bootloader entry process
            print("Monitoring bootloader entry process for 15 seconds...")
            time.sleep(15)
            
            # Step 5: Analyze results
            self.analyze_bootloader_logs()
            
            return True
            
        finally:
            self.stop_logging()

def main():
    diagnostic = BootloaderDiagnostic()
    
    try:
        success = diagnostic.run_diagnostic()
        
        if success:
            print("\n✓ DIAGNOSTIC COMPLETE")
            print("Check the analysis above to identify the failure point")
            return 0
        else:
            print("\n❌ DIAGNOSTIC FAILED")
            return 1
            
    except KeyboardInterrupt:
        print("\n\nDiagnostic interrupted by user")
        return 1
    except Exception as e:
        print(f"\n❌ DIAGNOSTIC ERROR: {e}")
        import traceback
        traceback.print_exc()
        return 1

if __name__ == "__main__":
    exit(main())