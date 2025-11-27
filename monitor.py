#!/usr/bin/env python3
"""
Serial monitor for RP2040-Zero
Usage: ./monitor.py
"""

import serial
import time
import signal
import sys

def signal_handler(sig, frame):
    print('\nExiting monitor...')
    sys.exit(0)

signal.signal(signal.SIGINT, signal_handler)

def monitor():
    try:
        ser = serial.Serial('/dev/ttyACM0', 115200, timeout=0.1)
        print("🔌 Connected to RP2040-Zero on /dev/ttyACM0")
        print("📡 Monitoring serial output (Ctrl+C to exit)")
        print("-" * 50)

        buffer = ""
        while True:
            if ser.in_waiting > 0:
                data = ser.read(ser.in_waiting).decode('utf-8', errors='ignore')
                buffer += data

                # Print complete lines
                while '\n' in buffer:
                    line, buffer = buffer.split('\n', 1)
                    if line.strip():
                        print(line.strip())

            time.sleep(0.01)

    except serial.SerialException as e:
        print(f"❌ Serial error: {e}")
        print("💡 Make sure RP2040-Zero is connected and not in bootloader mode")
    except Exception as e:
        print(f"❌ Error: {e}")
    finally:
        if 'ser' in locals():
            ser.close()

if __name__ == "__main__":
    monitor()