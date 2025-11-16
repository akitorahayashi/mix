//! Library entry point exposing the mix CLI command handlers.

pub mod commands;
pub mod error;

mod core;
mod storage;

pub use commands::{
    copy_snippet, generate_slash_commands, list_snippets, CopyOutcome, ListEntry,
    SlashGenerationOutcome, SlashRequest, SlashTarget,
};
