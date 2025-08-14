//! Command processing infrastructure for automated testing
//! Implements standardized 64-byte HID report format for command handling

// Re-export command parsing and handler modules
pub mod handler;
pub mod parsing;

// Public re-exports for external use
pub use parsing::{
    AuthenticationValidator, CommandParser, CommandQueue, CommandReport, ParseResult, ResponseQueue,
};
