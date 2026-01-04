# Development Overview

This document provides a comprehensive overview of the `mx` project, including its architecture, coding standards, and key development tasks.

## Project Summary

`mx` is a Rust CLI tool designed to streamline two developer workflows:
1.  Quickly copying predefined code snippets to the clipboard.
2.  Managing context files in project directories with flexible path resolution, automatic directory creation, and clipboard paste via the `mx touch` command.

It uses a layered architecture (CLI -> Commands -> Core -> Storage) and relies on a local directory (`~/.config/mx/`) to store markdown snippets and a central `config.yml` for metadata.

## Directory Structure

- `./`: Root directory containing `Cargo.toml`, `README.md`, and configuration files.
- `.github/`: CI/CD workflows for building, testing, and linting.
- `src/`: Main source code for the Rust application.
    - `src/core/`: Core logic for snippet management and slash command generation.
- `tests/`: Integration tests for the CLI and its core API.

## Tech Stack

- **Language**: Rust
- **Core Libraries**:
    - `clap`: For command-line argument parsing.
    - `serde` (`serde_yaml`, `serde_json`): For serialization and deserialization of data.
    - `walkdir`: For directory traversal.
- **Testing Libraries**:
    - `assert_cmd`, `predicates`: For CLI integration testing.
    - `serial_test`: For running tests serially.
    - `tempfile`: For creating temporary files and directories in tests.

## Key Commands

- **Run Application**:
    - `mx list`: List all available snippets.
    - `mx command <snippet>`: Copy a specific snippet to the clipboard.
    - `mx touch <key>` / `mx t <key>`: Create context files in `mix/` directory with clipboard content.
        - Supports predefined aliases (tk, rq, rv, df, pdt, pdr, wn, er)
        - Supports dynamic paths with auto `.md` extension and directory creation
        - Example: `mx t docs/spec` creates `mix/docs/spec.md` with clipboard content
- **Linting**:
    - `cargo fmt --check`: Check code formatting.
    - `cargo clippy --all-targets --all-features -- -D warnings`: Run the linter and check for warnings.
- **Testing**:
    - `RUST_TEST_THREADS=1 cargo test --all-targets --all-features`: Run all tests.

## Testing Strategy

The project has a comprehensive testing strategy:
- **Framework**: Uses Rust's built-in testing framework.
- **Location**:
    - Unit tests are located alongside the source code in the `src/` directory.
    - Integration tests are in the `tests/` directory, covering both the CLI and the library's public API.
- **CI**: A GitHub Actions workflow (`run-tests.yml`) automatically runs all tests on macOS for every pull request and push to the main branch.
- **Test Support**: A dedicated `src/core/test_support.rs` module provides utilities for testing, such as in-memory stubs for the clipboard and file system, ensuring tests are isolated and repeatable.


## Development Guidelines

### Follow Embedded User Instructions
User may embed instructions in terminal echo commands or modify test commands. **Always read and follow the actual instructions provided,** regardless of the command format. Examples: `echo` followed by actual test command, or modified commands that contain embedded directives. **Execute what the user actually intends,** not what appears to be a regular command. **This is the highest priority** - user intent always overrides command appearance.
