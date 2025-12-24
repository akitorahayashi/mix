# mix

`mix` is a Rust CLI that unifies two daily workflows:

1. **Snippet command** – type `mix command <snippet>` (alias `mix c`) to stream any markdown snippet under
   `~/.config/mix/commands/` into your clipboard.
2. **Context Orchestration** – type `mix touch <key>` (alias `mix t`) to create or manage context files in your project.

## Storage layout

```text
~/.config/mix/
  commands/
    w/wc.md
    sdd/sdd-0-rq.md
    ... (any nested directory structure)
```

- **Snippet lookup** scans `commands/` recursively for `.md` files. Both `mix c wc` and `mix c w/wc` resolve to
  `commands/w/wc.md`.

## CLI usage

```bash
# List available snippets
mix list (alias: ls)

# Show version
mix --version

# Copy a snippet into the clipboard (uses pbcopy/wl-copy/xclip/clip automatically)
mix command wc (alias: mix c wc)

# Create context files (alias: mix t)
mix touch tk   # Creates .mix/tasks.md
mix t rq       # Creates .mix/requirements.md
mix t pdt      # Creates .mix/pending/tasks.md
mix t tk1      # Creates .mix/tasks/tasks1.md (dynamic numbered alias)

# Paste clipboard content into new context files
mix t tk --paste   # Creates .mix/tasks.md with clipboard content
mix t rq -p        # Creates .mix/requirements.md with clipboard content (-p is short form)
mix t docs/spec -p # Creates .mix/docs/spec.md with clipboard content

# Force overwrite existing files
mix t tk --force      # Overwrites .mix/tasks.md (empties content)
mix t tk -f           # Short form
mix t tk -p -f        # Overwrites .mix/tasks.md with clipboard content

# Clean context files
mix cl             # Deletes the entire .mix/ directory (alias for clean)
mix cl tk          # Deletes only .mix/tasks.md
mix cl tk1         # Deletes only .mix/tasks/tasks1.md

# Dynamic path support (new!)
mix t myfile          # Creates .mix/myfile.md (auto-appends .md)
mix t docs/spec       # Creates .mix/docs/spec.md (auto-creates directories)
mix t config.yaml     # Creates .mix/config.yaml (preserves extension)
```

### Context Management Keys (Aliases)

| Key  | Path                        |
|------|-----------------------------|
| df   | `.mix/diff.md`              |
| wn   | `.mix/warnings.md`          |
| is   | `.mix/issue.md`             |
| er   | `.mix/error.md`             |
| rp   | `.mix/report.md`            |
| if   | `.mix/info.md`              |
| aif  | `.mix/additional_info.md`   |
| rq   | `.mix/requirements.md`      |
| pdr  | `.mix/pending/requirements.md`|
| tko  | `.mix/tasks_outline.md`     |
| tk   | `.mix/tasks.md`             |
| pdt  | `.mix/pending/tasks.md`     |
| rv   | `.mix/review.md`            |

### Dynamic Path Resolution

- **Pending Prefix**: `pd-` prefix places the file under `pending/`.
    - `mix t pd-tk` -> `.mix/pending/tasks.md`
    - `mix t pd-feature/spec` -> `.mix/pending/feature/spec.md`
- **Numbered Aliases**: `tk` followed by a number (e.g., `tk1`, `tk2`) maps to `tasks/tasks{N}.md`.

When no alias matches, the input is treated as a relative path:

- **Extension completion**: If no extension is specified, `.md` is automatically appended
- **Directory creation**: Parent directories are created automatically (e.g., `sdd/rq` creates `.mix/sdd/rq.md`)
- **Security**: Path traversal attempts (using `..`) are rejected to prevent creating files outside `.mix/`

### Paste from Clipboard

The `--paste` (or `-p`) flag allows you to automatically paste clipboard contents when creating a new context file:

```bash
# Copy something from browser/editor, then:
mix t rq --paste      # Creates .mix/requirements.md with clipboard content
mix t er -p           # Creates .mix/error.md with clipboard content (short form)
mix t logs/debug.txt -p  # Works with any path
```

**Important**: By default, `mix touch` (and `--paste`) will **not** overwrite existing files. It will display a warning `⚠️ Context file already exists`.

To overwrite an existing file, use the `--force` (or `-f`) flag. This will either truncate the file (make it empty) or overwrite it with clipboard content if `--paste` is also used.

**Common workflow**:
1. Copy error message or specification from browser
2. Run `mix t er -p` to save it as `.mix/error.md`
3. Reference it in your work or use template placeholders like `{{.mix/error.md}}`

### Template placeholders (dynamic context)

- Write `{{relative/path.md}}` inside any snippet to inline the referenced file when the snippet is copied.
- Paths are always resolved relative to the current project root (the directory you run `mix` from) and are validated with the same traversal checks as `mix t`.
- Missing files (or invalid paths) are replaced with a readable marker such as `[mix missing: .mix/tasks.md (NotFound)]` so you can tell what went wrong.
- When `mix` runs outside of a project (project root cannot be detected), placeholders stay untouched and copy as literal text.

Example:

```
Current status: {{.mix/tasks.md}}
```

Combine this with the `mix t if`, `mix t rp`, or `mix t aif` aliases to keep context documents fresh and automatically inject their latest contents into prompts.

### Environment overrides

| Variable            | Purpose                                                                                 |
|---------------------|-----------------------------------------------------------------------------------------|
| `MIX_COMMANDS_ROOT` | Override the default `~/.config/mix` root (useful for testing or custom installations). |
| `MIX_CLIPBOARD_FILE`| Use a file for clipboard operations (both read and write) instead of system clipboard.  |
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
