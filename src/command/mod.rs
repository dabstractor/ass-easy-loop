//! Command processing infrastructure for automated testing
//! Implements standardized 64-byte HID report format for command handling

// Re-export command parsing and handler modules
pub mod parsing;
pub mod handler;

// Public re-exports for external use
pub use parsing::{
    CommandReport, ParseResult, CommandQueue, CommandParser,
    ResponseQueue, AuthenticationValidator
};