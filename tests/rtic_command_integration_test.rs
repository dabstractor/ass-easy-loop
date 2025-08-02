//! Integration tests for RTIC task coordination with command processing
//! Tests the integration between USB polling task and command handler task
//! Requirements: 2.2, 8.1, 8.2, 8.3

#![no_std]
#![no_main]

use panic_halt as _;
use rp2040_hal as hal;
use hal::{
    clocks::init_clocks_and_plls,
    gpio::{
        bank0::{Gpio15, Gpio25, Gpio26},
        FunctionSio, Pin, PullDown, PullNone, SioInput, SioOutput,
    },
    sio::Sio,
    usb::UsbBus,
    watchdog::Watchdog,
    adc::{Adc, AdcPin},
};
use usb_device::{
    class_prelude::UsbBusAllocator,
    prelude::{UsbDevice, UsbDeviceBuilder, UsbVidPid},
};
use usbd_hid::{
    descriptor::{generator_prelude::*, SerializedDescriptor},
    hid_class::HIDClass,
};
use rtic_monotonics::rp2040::prelude::*;
use heapless::Vec;

// Import project modules
use ass_easy_loop::{
    command::{CommandReport, ParseResult, CommandQueue, CommandParser, ResponseQueue, AuthenticationValidator, TestCommand},
    logging::{LogLevel, Logger, QueueLogger, LogQueue, LogReport, LogMessage},
    battery::BatteryState,
    bootloader::BootloaderEntryManager,
    system_state::SystemStateHandler,
    test_processor::TestCommandProcessor,
    config::usb as usb_config,
};

// Create the monotonic timer type
rp2040_timer_monotonic!(Mono);

/// Test USB HID report descriptor
#[gen_hid_descriptor(
    (collection = APPLICATION, usage_page = VENDOR_DEFINED_START, usage = 0x01) = {
        (usage = 0x01, logical_min = 0x00) = {
            #[item_settings data,variable,absolute]
            input=input_buffer=64;
        };
        (usage = 0x02, logical_min = 0x00) = {
            #[item_settings data,variable,absolute]
            output=output_buffer=64;
        };
    }
)]
pub struct TestHidReport {
    pub input_buffer: [u8; 64],
    pub output_buffer: [u8; 64],
}

#[rtic::app(device = rp2040_pac, peripherals = true, dispatchers = [TIMER_IRQ_1, TIMER_IRQ_2, TIMER_IRQ_3])]
mod app {
    use super::*;

    #[shared]
    struct Shared {
        usb_dev: UsbDevice<'static, UsbBus>,
        hid_class: HIDClass<'static, UsbBus>,
        command_queue: CommandQueue<8>,
        response_queue: ResponseQueue<8>,
        command_parser: CommandParser,
        bootloader_manager: BootloaderEntryManager,
        system_state_handler: SystemStateHandler,
        test_processor: TestCommandProcessor,
        test_results: Vec<TestResult, 16>,
    }

    #[local]
    struct Local {
        test_counter: u32,
        commands_received: u32,
        commands_processed: u32,
    }

    /// Test result structure for tracking integration test outcomes
    #[derive(Clone, Debug, PartialEq)]
    pub struct TestResult {
        pub test_name: &'static str,
        pub passed: bool,
        pub details: &'static str,
        pub timestamp_ms: u32,
    }

    #[init]
    fn init(mut ctx: init::Context) -> (Shared, Local) {
        // Set up clocks and PLL
        let mut watchdog = Watchdog::new(ctx.device.WATCHDOG);
        let clocks = init_clocks_and_plls(
            12_000_000u32,
            ctx.device.XOSC,
            ctx.device.CLOCKS,
            ctx.device.PLL_SYS,
            ctx.device.PLL_USB,
            &mut ctx.device.RESETS,
            &mut watchdog,
        ).ok().unwrap();

        // Initialize the RP2040 Timer monotonic
        Mono::start(ctx.device.TIMER, &mut ctx.device.RESETS);

        // Set up USB bus allocator and HID class device
        static mut USB_BUS: Option<UsbBusAllocator<UsbBus>> = None;
        
        let usb_bus = UsbBus::new(
            ctx.device.USBCTRL_REGS,
            ctx.device.USBCTRL_DPRAM,
            clocks.usb_clock,
            true,
            &mut ctx.device.RESETS,
        );
        
        unsafe {
            USB_BUS = Some(UsbBusAllocator::new(usb_bus));
        }
        
        let usb_bus_ref = unsafe { USB_BUS.as_ref().unwrap() };

        // Create HID class device with test report descriptor
        let hid_class = HIDClass::new(usb_bus_ref, TestHidReport::descriptor(), 60);

        // Configure USB device
        let usb_dev = UsbDeviceBuilder::new(usb_bus_ref, UsbVidPid(usb_config::VENDOR_ID, usb_config::PRODUCT_ID))
            .device_release(usb_config::DEVICE_RELEASE)
            .device_class(0x00)
            .build();

        // Initialize command infrastructure
        let command_queue = CommandQueue::new();
        let response_queue = ResponseQueue::new();
        let command_parser = CommandParser::new();
        let bootloader_manager = BootloaderEntryManager::new();
        let system_state_handler = SystemStateHandler::new();
        let test_processor = TestCommandProcessor::new();

        // Start integration tests
        integration_test_coordinator::spawn().ok();
        usb_polling_test_task::spawn().ok();
        command_processing_test_task::spawn().ok();

        (
            Shared {
                usb_dev,
                hid_class,
                command_queue,
                response_queue,
                command_parser,
                bootloader_manager,
                system_state_handler,
                test_processor,
                test_results: Vec::new(),
            },
            Local {
                test_counter: 0,
                commands_received: 0,
                commands_processed: 0,
            },
        )
    }

    /// Integration test coordinator task
    /// Orchestrates the execution of various integration tests
    /// Requirements: 2.2, 8.1, 8.2, 8.3
    #[task(shared = [test_results], local = [test_counter], priority = 1)]
    async fn integration_test_coordinator(mut ctx: integration_test_coordinator::Context) {
        let test_counter = ctx.local.test_counter;
        
        // Test 1: Command queue integration
        *test_counter += 1;
        command_queue_integration_test::spawn(*test_counter).ok();
        Mono::delay(100.millis()).await;

        // Test 2: USB HID command parsing integration
        *test_counter += 1;
        usb_command_parsing_test::spawn(*test_counter).ok();
        Mono::delay(100.millis()).await;

        // Test 3: Task priority coordination test
        *test_counter += 1;
        task_priority_coordination_test::spawn(*test_counter).ok();
        Mono::delay(100.millis()).await;

        // Test 4: Command timeout handling test
        *test_counter += 1;
        command_timeout_test::spawn(*test_counter).ok();
        Mono::delay(100.millis()).await;

        // Test 5: Response queue integration test
        *test_counter += 1;
        response_queue_integration_test::spawn(*test_counter).ok();
        Mono::delay(100.millis()).await;

        // Wait for all tests to complete
        Mono::delay(1000.millis()).await;

        // Report test results
        test_results_reporter::spawn().ok();
    }

    /// Test command queue integration between USB polling and command handler tasks
    /// Requirements: 2.2 (command processing with medium priority)
    #[task(shared = [command_queue, test_results], priority = 2)]
    async fn command_queue_integration_test(mut ctx: command_queue_integration_test::Context, test_id: u32) {
        let timestamp = Mono::now().duration_since_epoch().to_millis() as u32;
        
        // Create a test command
        let test_payload = [0x01, 0x02, 0x03, 0x04]; // Test payload
        let test_command = CommandReport::new(
            TestCommand::SystemStateQuery as u8,
            test_id as u8,
            &test_payload
        ).unwrap();

        // Test command enqueueing
        let enqueue_success = ctx.shared.command_queue.lock(|queue| {
            queue.enqueue(test_command.clone(), timestamp, 5000)
        });

        // Verify command was enqueued successfully
        let queue_length = ctx.shared.command_queue.lock(|queue| queue.len());
        
        let test_result = TestResult {
            test_name: "Command Queue Integration",
            passed: enqueue_success && queue_length > 0,
            details: if enqueue_success && queue_length > 0 {
                "Command successfully enqueued"
            } else {
                "Failed to enqueue command"
            },
            timestamp_ms: timestamp,
        };

        // Store test result
        ctx.shared.test_results.lock(|results| {
            let _ = results.push(test_result);
        });

        // Test command dequeuing
        let dequeued_command = ctx.shared.command_queue.lock(|queue| {
            queue.dequeue()
        });

        let dequeue_test_result = TestResult {
            test_name: "Command Queue Dequeue",
            passed: dequeued_command.is_some(),
            details: if dequeued_command.is_some() {
                "Command successfully dequeued"
            } else {
                "Failed to dequeue command"
            },
            timestamp_ms: timestamp + 1,
        };

        ctx.shared.test_results.lock(|results| {
            let _ = results.push(dequeue_test_result);
        });
    }

    /// Test USB HID command parsing integration
    /// Requirements: 2.1 (USB HID output report handling)
    #[task(shared = [command_parser, test_results], priority = 2)]
    async fn usb_command_parsing_test(mut ctx: usb_command_parsing_test::Context, test_id: u32) {
        let timestamp = Mono::now().duration_since_epoch().to_millis() as u32;
        
        // Create a valid 64-byte USB HID report
        let mut report_buffer = [0u8; 64];
        report_buffer[0] = TestCommand::ExecuteTest as u8; // Command type
        report_buffer[1] = test_id as u8; // Command ID
        report_buffer[2] = 4; // Payload length
        report_buffer[4] = 0xAA; // Test payload
        report_buffer[5] = 0xBB;
        report_buffer[6] = 0xCC;
        report_buffer[7] = 0xDD;
        
        // Calculate and set authentication token (checksum)
        let mut checksum = report_buffer[0] ^ report_buffer[1] ^ report_buffer[2];
        for i in 4..8 {
            checksum ^= report_buffer[i];
        }
        report_buffer[3] = checksum;

        // Test command parsing
        let parse_result = ctx.shared.command_parser.lock(|parser| {
            parser.parse_command(&report_buffer)
        });

        let parsing_success = matches!(parse_result, ParseResult::Valid(_));
        
        let test_result = TestResult {
            test_name: "USB Command Parsing",
            passed: parsing_success,
            details: if parsing_success {
                "Command parsed successfully"
            } else {
                "Command parsing failed"
            },
            timestamp_ms: timestamp,
        };

        ctx.shared.test_results.lock(|results| {
            let _ = results.push(test_result);
        });

        // Test authentication validation
        if let ParseResult::Valid(command) = parse_result {
            let auth_valid = AuthenticationValidator::validate_command(&command);
            let format_valid = AuthenticationValidator::validate_format(&command).is_ok();
            
            let auth_test_result = TestResult {
                test_name: "Command Authentication",
                passed: auth_valid && format_valid,
                details: if auth_valid && format_valid {
                    "Authentication and format validation passed"
                } else {
                    "Authentication or format validation failed"
                },
                timestamp_ms: timestamp + 1,
            };

            ctx.shared.test_results.lock(|results| {
                let _ = results.push(auth_test_result);
            });
        }
    }

    /// Test task priority coordination to ensure command processing doesn't interfere with critical timing
    /// Requirements: 8.1, 8.2, 8.3 (performance impact minimization)
    #[task(shared = [command_queue, test_results], priority = 3)]
    async fn task_priority_coordination_test(mut ctx: task_priority_coordination_test::Context, test_id: u32) {
        let timestamp = Mono::now().duration_since_epoch().to_millis() as u32;
        let start_time = Mono::now();
        
        // Simulate high-priority task execution (like pEMF pulse generation)
        // This task should not be preempted by command processing
        
        // Create multiple commands to test priority handling
        for i in 0..5 {
            let test_command = CommandReport::new(
                TestCommand::PerformanceMetrics as u8,
                (test_id + i) as u8,
                &[i as u8, 0x00, 0x00, 0x00]
            ).unwrap();

            ctx.shared.command_queue.lock(|queue| {
                queue.enqueue(test_command, timestamp + i as u32, 5000)
            });
        }

        // Perform timing-critical work that should not be interrupted
        let mut computation_result = 0u32;
        for i in 0..1000 {
            computation_result = computation_result.wrapping_add(i);
            // Small delay to simulate real work
            for _ in 0..100 {
                cortex_m::asm::nop();
            }
        }

        let end_time = Mono::now();
        let execution_time_ms = (end_time - start_time).to_millis();
        
        // Verify that execution completed without significant delay
        // (indicating no preemption by lower priority tasks)
        let timing_acceptable = execution_time_ms < 100; // Should complete quickly
        
        let test_result = TestResult {
            test_name: "Task Priority Coordination",
            passed: timing_acceptable && computation_result > 0,
            details: if timing_acceptable {
                "High priority task completed without interference"
            } else {
                "High priority task execution was delayed"
            },
            timestamp_ms: timestamp,
        };

        ctx.shared.test_results.lock(|results| {
            let _ = results.push(test_result);
        });
    }

    /// Test command timeout handling
    /// Requirements: 6.4 (timeout handling)
    #[task(shared = [command_queue, test_results], priority = 2)]
    async fn command_timeout_test(mut ctx: command_timeout_test::Context, test_id: u32) {
        let timestamp = Mono::now().duration_since_epoch().to_millis() as u32;
        
        // Create a command with a very short timeout
        let test_command = CommandReport::new(
            TestCommand::ConfigurationQuery as u8,
            test_id as u8,
            &[0xFF, 0xFF, 0xFF, 0xFF]
        ).unwrap();

        // Enqueue with 1ms timeout (will expire quickly)
        ctx.shared.command_queue.lock(|queue| {
            queue.enqueue(test_command, timestamp, 1)
        });

        // Wait for timeout to occur
        Mono::delay(10.millis()).await;

        // Check for timed out commands
        let current_time = Mono::now().duration_since_epoch().to_millis() as u32;
        let timed_out_count = ctx.shared.command_queue.lock(|queue| {
            queue.remove_timed_out_commands(current_time)
        });

        let test_result = TestResult {
            test_name: "Command Timeout Handling",
            passed: timed_out_count > 0,
            details: if timed_out_count > 0 {
                "Timed out commands properly removed"
            } else {
                "Timeout handling failed"
            },
            timestamp_ms: current_time,
        };

        ctx.shared.test_results.lock(|results| {
            let _ = results.push(test_result);
        });
    }

    /// Test response queue integration
    /// Requirements: 2.5 (error responses with diagnostic information)
    #[task(shared = [response_queue, test_results], priority = 2)]
    async fn response_queue_integration_test(mut ctx: response_queue_integration_test::Context, test_id: u32) {
        let timestamp = Mono::now().duration_since_epoch().to_millis() as u32;
        
        // Create a test response
        let test_response = CommandReport::success_response(
            test_id as u8,
            &[0x01, 0x02, 0x03, 0x04]
        ).unwrap();

        // Test response enqueueing
        let enqueue_success = ctx.shared.response_queue.lock(|queue| {
            queue.enqueue(test_response, test_id, timestamp)
        });

        // Test response dequeuing
        let dequeued_response = ctx.shared.response_queue.lock(|queue| {
            queue.dequeue()
        });

        let test_result = TestResult {
            test_name: "Response Queue Integration",
            passed: enqueue_success && dequeued_response.is_some(),
            details: if enqueue_success && dequeued_response.is_some() {
                "Response queue operations successful"
            } else {
                "Response queue operations failed"
            },
            timestamp_ms: timestamp,
        };

        ctx.shared.test_results.lock(|results| {
            let _ = results.push(test_result);
        });
    }

    /// Mock USB polling task for testing
    /// Simulates the behavior of the real USB polling task
    #[task(shared = [usb_dev, hid_class, command_queue], local = [commands_received], priority = 1)]
    async fn usb_polling_test_task(mut ctx: usb_polling_test_task::Context) {
        let commands_received = ctx.local.commands_received;
        
        loop {
            // Simulate USB polling behavior
            ctx.shared.usb_dev.lock(|usb_dev| {
                ctx.shared.hid_class.lock(|hid_class| {
                    // Poll the USB device
                    usb_dev.poll(&mut [hid_class]);
                    
                    // Simulate command reception (in real implementation, this would come from USB)
                    // For testing, we'll periodically create test commands
                    if *commands_received < 3 {
                        let test_command = CommandReport::new(
                            TestCommand::SystemStateQuery as u8,
                            *commands_received as u8,
                            &[0x01, 0x02]
                        ).unwrap();

                        let timestamp = Mono::now().duration_since_epoch().to_millis() as u32;
                        ctx.shared.command_queue.lock(|queue| {
                            queue.enqueue(test_command, timestamp, 5000)
                        });

                        *commands_received += 1;
                    }
                })
            });

            Mono::delay(50.millis()).await;
        }
    }

    /// Mock command processing task for testing
    /// Simulates the behavior of the real command handler task
    #[task(shared = [command_queue, response_queue], local = [commands_processed], priority = 2)]
    async fn command_processing_test_task(mut ctx: command_processing_test_task::Context) {
        let commands_processed = ctx.local.commands_processed;
        
        loop {
            // Check for commands in the queue
            let queued_command = ctx.shared.command_queue.lock(|queue| {
                queue.dequeue()
            });

            if let Some(cmd) = queued_command {
                // Process the command (simplified for testing)
                let response = CommandReport::success_response(
                    cmd.command.command_id,
                    &[0xAA, 0xBB]
                ).unwrap();

                // Queue response
                let timestamp = Mono::now().duration_since_epoch().to_millis() as u32;
                ctx.shared.response_queue.lock(|queue| {
                    queue.enqueue(response, cmd.sequence_number, timestamp)
                });

                *commands_processed += 1;
            }

            Mono::delay(50.millis()).await;
        }
    }

    /// Test results reporter task
    /// Reports the results of all integration tests
    #[task(shared = [test_results], priority = 1)]
    async fn test_results_reporter(mut ctx: test_results_reporter::Context) {
        let results = ctx.shared.test_results.lock(|results| {
            results.clone()
        });

        let mut passed_tests = 0;
        let mut total_tests = results.len();

        // In a real test environment, these would be reported via test framework
        // For now, we'll use a simple pass/fail count
        for result in &results {
            if result.passed {
                passed_tests += 1;
            }
        }

        // Test completion indicator
        // In real implementation, this would trigger test framework completion
        let all_tests_passed = passed_tests == total_tests;
        
        // Simulate test completion
        if all_tests_passed {
            // All tests passed - success
            loop {
                Mono::delay(1000.millis()).await;
            }
        } else {
            // Some tests failed - indicate failure
            loop {
                Mono::delay(500.millis()).await;
            }
        }
    }
}