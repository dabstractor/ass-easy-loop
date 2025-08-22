# 📜 PROJECT DEVELOPMENT RULES & GUIDELINES

## ⚠️ ABSOLUTES - NEVER VIOLATE THESE RULES

### 🔴 USB ENUMERATION PROTECTION RULES
**NEVER MODIFY THESE COMPONENTS WITHOUT EXPLICIT PERMISSION:**
- RTIC dispatcher configuration: `#[rtic::app(..., dispatchers = [TIMER_IRQ_1, TIMER_IRQ_2, TIMER_IRQ_3])]`
- USB bus initialization sequence in `init()` function
- USB device polling timing and frequency
- USB HID class initialization parameters
- Static USB bus allocation: `static mut USB_BUS: Option<UsbBusAllocator<UsbBus>>`

**AFTER ANY CHANGE TO USB-RELATED CODE:**
1. IMMEDIATELY run `cargo build`
2. IMMEDIATELY run `cargo run` to flash
3. IMMEDIATELY verify `lsusb | grep -i fade` shows device
4. IF device disappears - STOP WORK and ROLLBACK IMMEDIATELY

### 🔴 RTIC FRAMEWORK RULES
**DO NOT MODIFY RTIC CONFIGURATIONS WITHOUT UNDERSTANDING:**
- Monotonic timer settings: `type MyMono = Systick<1000>`
- Task priority assignments
- Shared resource locking mechanisms
- Software task dispatcher counts

### 🔴 TIMING AND UNITS RULES
**CONSISTENCY IS CRITICAL:**
- Use `Duration::<u64, 1, 1000>::millis(X)` for human-readable timing
- Use `Duration::<u64, 1, 1000000>::micros(X)` only when microsecond precision required
- NEVER mix millisecond and microsecond units in same timing context
- USB polling must remain at 10ms intervals

## ✅ MANDATORY DEVELOPMENT PRACTICES

### 🛡️ CHANGE CONTROL PROCEDURES
1. **ONE CHANGE AT A TIME** - Never modify multiple systems simultaneously
2. **TEST AFTER EVERY LINE** - Verify USB enumeration after each modification
3. **GIT COMMIT BEFORE RISKY CHANGES** - Always have rollback point
4. **DOCUMENT SUSPECT CHANGES** - Comment why and what you're changing

### 🧪 VERIFICATION REQUIREMENTS
**AFTER EVERY CODE MODIFICATION:**
```bash
# Required verification sequence
cargo build                    # Must compile clean
cargo run                      # Must flash successfully  
lsusb | grep -i fade          # Must show device enumeration
# Only proceed if ALL steps pass
```

### 📁 FILE MODIFICATION GUIDELINES

#### ✅ SAFE TO MODIFY:
- Feature implementations in `src/drivers/` (when not affecting USB)
- Configuration values in `src/config/`
- Type definitions in `src/types/` (when not affecting USB structures)
- Test code and documentation
- Non-critical task implementations

#### 🔴 STRICTLY FORBIDDEN MODIFICATIONS:
- **src/main.rs USB sections** - Never touch unless specifically instructed
- **RTIC app configuration** - `[rtic::app(...)]` declarations
- **USB bus allocator** - `USB_BUS` static declaration
- **USB device initialization** - Clock setup, PLL configuration
- **USB polling tasks** - Timing, spawn intervals, poll() calls
- **Monotonic timer configuration** - Systick settings

## 🎯 DEVELOPMENT WORKFLOW

### 1. PRE-CHANGE SAFETY CHECK
```bash
# Document current working state
git status
git diff  # Should be clean for critical files
lsusb | grep fade  # Document working device
```

### 2. INCREMENTAL DEVELOPMENT
```bash
# Make ONE small change
# Test immediately
cargo build && cargo run && lsusb | grep fade
# Only continue if ALL PASS
```

### 3. POST-CHANGE VALIDATION
```bash
# Verify nothing broke
cargo build                    # Zero errors required
cargo run                      # Must complete successfully
lsusb | grep fade             # Device must enumerate
dmesg | tail -20              # Check for USB errors
```

## ⚡ EMERGENCY RECOVERY PROCEDURES

### IF USB ENUMERATION BREAKS:
1. **IMMEDIATELY STOP ALL WORK**
2. **DO NOT MAKE ANY MORE CHANGES**
3. **ROLLBACK TO LAST KNOWN WORKING STATE:**
   ```bash
   git stash  # or git reset --hard HEAD
   ```
4. **VERIFY RECOVERY:**
   ```bash
   cargo build && cargo run && lsusb | grep fade
   ```
5. **ONLY THEN - Identify and understand the breaking change**

### IF GIT HISTORY IS CORRUPTED:
1. **Use project backup or clean clone**
2. **NEVER attempt to "fix" by adding more code**
3. **Restore from known good commit**

## 📋 CODE REVIEW CHECKLIST

Before any commit affecting embedded functionality:

- [ ] USB enumeration verified after changes
- [ ] RTIC dispatcher configuration unchanged
- [ ] USB polling timing unchanged (10ms)
- [ ] Monotonic timer configuration unchanged
- [ ] USB bus allocator structure unchanged
- [ ] No syntax errors or extra braces
- [ ] All verification steps pass
- [ ] Git diff reviewed for unintended changes

## 🚨 WARNING SIGNS - STOP WORK IMMEDIATELY

- ❌ `lsusb` shows no device or "Unknown Device"
- ❌ "not enough interrupts to dispatch" errors
- ❌ USB-related compilation errors
- ❌ Flashing fails with USB errors
- ❌ Device requires manual BOOTSEL reset
- ❌ Multiple unrelated warnings appear

## 🛠️ DEBUGGING PROTOCOL

### When troubleshooting USB issues:
1. **CHECK HARDWARE FIRST** - Ensure device is physically connected
2. **VERIFY BASELINE** - Test with known working firmware
3. **ISOLATE CHANGES** - Identify exact commit that broke enumeration
4. **UNDERSTAND BEFORE FIXING** - Research why change caused breakage
5. **RESTORE THEN ENHANCE** - Get working state back before improvements

### Safe debugging approach:
```bash
# Start with clean slate
git stash
cargo run  # Verify working baseline

# Make minimal changes
# Test after each
# Document what works/breaks
```

## 📚 KNOWLEDGE REQUIREMENTS

Developers MUST understand before modifying critical systems:

1. **RTIC Framework** - How dispatchers, tasks, and shared resources work
2. **USB Device Enumeration** - What makes a device appear in lsusb
3. **RP2040 USB Peripheral** - How the hardware works
4. **Embedded Timing Systems** - Milliseconds vs microseconds implications
5. **Fugit Duration Types** - How timing units affect real-time systems

## 🏁 FINAL REMINDER

This project's primary function depends on USB enumeration. **The device is 100% useless if USB doesn't work.** 

**PRIORITIZE USB ENUMERATION ABOVE ALL OTHER FEATURES.**

---
*Violating these rules results in complete device failure and wasted development time. Follow them religiously.*
