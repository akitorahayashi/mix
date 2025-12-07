# mix

`mix` is a Rust CLI that unifies two daily workflows:

1. **Snippet copying** – type `mix <snippet>` (for example `mix wc`) to stream any markdown snippet under
   `~/.config/mix/commands/` into your clipboard.
2. **Context Orchestration** – type `mix touch <key>` (alias `mix t`) to create or manage context files in your project.

The project started from `rs-cli-tmpl`, keeping the layered architecture (CLI → command handlers → core logic
→ storage) while replacing the sample CRUD commands with real snippet-aware behaviors.

## Storage layout

```text
~/.config/mix/
  commands/
    w/wc.md
    sdd/sdd-0-rq.md
    ... (any nested directory structure)
```

- **Snippet lookup** scans `commands/` recursively for `.md` files. Both `mix wc` and `mix w/wc` resolve to
  `commands/w/wc.md`.

## CLI usage

```bash
# List available snippets
mix list

# Show version
mix --version

# Copy a snippet into the clipboard (uses pbcopy/wl-copy/xclip/clip automatically)
mix wc

# Create context files (alias: mix t)
mix touch tk   # Creates .mix/tasks.md
mix t rq       # Creates .mix/requirements.md
mix t pdt      # Creates .mix/pending/tasks.md
mix t tk1      # Creates .mix/tasks/tasks1.md (dynamic numbered alias)

# Clean context files
mix clean      # Deletes the entire .mix/ directory
mix clean tk   # Deletes only .mix/tasks.md
mix clean tk1  # Deletes only .mix/tasks/tasks1.md

# Dynamic path support (new!)
mix t myfile          # Creates .mix/myfile.md (auto-appends .md)
mix t docs/spec       # Creates .mix/docs/spec.md (auto-creates directories)
mix t config.yaml     # Creates .mix/config.yaml (preserves extension)
```

### Context Management Keys (Aliases)

| Key  | Path                        |
|------|-----------------------------|
| tk   | `.mix/tasks.md`             |
| rq   | `.mix/requirements.md`      |
| rv   | `.mix/review.md`            |
| df   | `.mix/diff.md`              |
| pdt  | `.mix/pending/tasks.md`     |
| pdr  | `.mix/pending/requirements.md`|
| wn   | `.mix/warnings.md`          |
| er   | `.mix/error.md`             |

### Dynamic Path Resolution

- **Numbered Aliases**: `tk` followed by a number (e.g., `tk1`, `tk2`) maps to `tasks/tasks{N}.md`.

When no alias matches, the input is treated as a relative path:

- **Extension completion**: If no extension is specified, `.md` is automatically appended
- **Directory creation**: Parent directories are created automatically (e.g., `sdd/rq` creates `.mix/sdd/rq.md`)
- **Security**: Path traversal attempts (using `..`) are rejected to prevent creating files outside `.mix/`

### Environment overrides

| Variable            | Purpose                                                                                 |
|---------------------|-----------------------------------------------------------------------------------------|
| `MIX_COMMANDS_ROOT` | Override the default `~/.config/mix` root (useful for testing or custom installations). |
| `MIX_CLIPBOARD_FILE`| Write clipboard contents to a file instead of invoking pbcopy/wl-copy/xclip/clip.       |
| `MIX_CLIPBOARD_CMD` | Provide a custom clipboard command if the auto-detected one is unavailable.             |

## Development guide

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
RUST_TEST_THREADS=1 cargo test --all-targets --all-features
```

The workspace follows the original template’s testing culture:

- **Unit tests** live next to their modules (clipboard abstraction, snippet storage, touch).
- **Core support** (`src/core/test_support.rs`) provides in-memory clipboard stubs and scratch snippet roots.
- **Integration crates** under `tests/` exercise both the CLI (`cli_commands.rs`, `cli_flow.rs`, `cli_touch.rs`) and the
  public library API (`commands_api.rs`). Shared helpers in `tests/common/` seed snippet catalogs inside a
  temporary HOME and expose utilities for overriding clipboard/destination paths.

## Migrating from `reference/menv`

1. Copy the existing markdown prompts into `~/.config/mix/commands/` while preserving subdirectories.
2. Run `mix list` to verify the catalog.
3. Use `mix touch` (or `mix t`) to manage your context files instead of legacy aliases.

That’s it—you now have a single Rust binary covering both snippet copy flows and context orchestration.
