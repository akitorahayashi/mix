//! Library entry point exposing the mix CLI command handlers.

pub mod commands;
pub mod error;

mod core;
mod storage;

pub use commands::{
    copy_snippet, list_snippets, touch_context, CopyOutcome, ListEntry, TouchOutcome,
};
