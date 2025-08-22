# 🚨 USB ENUMERATION CHEAT SHEET

## 🔥 NEVER TOUCH THESE LINES

```rust
#[rtic::app(device = rp2040_hal::pac, peripherals = true, dispatchers = [TIMER_IRQ_1, TIMER_IRQ_2, TIMER_IRQ_3])]

type MyMono = Systick<1000>;

static mut USB_BUS: Option<UsbBusAllocator<UsbBus>> = None;

usb_poll_task::spawn_after(Duration::<u64, 1, 1000>::millis(10)).unwrap();

usb_dev.poll(&mut [hid_class])  // NEVER REMOVE THIS LINE
```

## ✅ IMMEDIATE VERIFICATION SEQUENCE

```bash
# AFTER EVERY SINGLE CHANGE:
cargo build && cargo run && lsusb | grep fade
```

**Expected output:** `Bus XXX Device XXX: ID fade:1212 dabstractor Ass-Easy Loop`

## ⚡ EMERGENCY RECOVERY

```bash
# IF USB BREAKS:
git stash          # STOP ALL WORK
cargo run         # Verify baseline restored
# THEN investigate what broke it
```

## 🚫 ABSOLUTE FORBIDDEN CHANGES

- ❌ Changing RTIC dispatchers from `[TIMER_IRQ_1, TIMER_IRQ_2, TIMER_IRQ_3]`
- ❌ Modifying USB polling timing from 10ms
- ❌ Adding unnecessary imports to USB initialization
- ❌ Changing monotonic timer from `Systick<1000>`
- ❌ Removing or modifying `usb_dev.poll(&mut [hid_class])`
- ❌ Adding battery monitoring code to USB tasks

## 🛡️ SAFE CHANGE PROTOCOL

1. **One change only** - Test immediately
2. **Verify USB** - Run full verification sequence
3. **Commit working** - Only commit if USB works
4. **Document changes** - Comment unusual modifications

## 🆘 USB ENUMERATION BROKEN?

**IMMEDIATELY STOP AND:**
1. `git stash` or `git reset --hard`
2. `cargo run` to restore working state
3. Identify exact change that broke it
4. Understand why before attempting fix

---
*USB enumeration is mission-critical. When in doubt, don't touch it.*