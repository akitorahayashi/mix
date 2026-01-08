# mx

`mx` is a Rust CLI that unifies two daily workflows:

1. **Snippet command** – type `mx command <snippet>` (alias `mx c`) to stream any markdown snippet under
   `~/.config/mx/commands/` into your clipboard.
2. **Context Orchestration** – type `mx touch <key>` (alias `mx t`) to create context files in your project with clipboard content, and `mx cat <key>` (alias `mx ct`) to view their contents.

## Storage layout

```text
~/.config/mx/
  commands/
    w/wc.md
    sdd/sdd-0-rq.md
    ... (any nested directory structure)
```

- **Snippet lookup** scans `commands/` recursively for `.md` files. Both `mx c wc` and `mx c w/wc` resolve to
  `commands/w/wc.md`.

## CLI usage

```bash
# List available snippets
mx list (alias: ls)

# Show version
mx --version

# Copy a snippet into the clipboard (uses pbcopy/wl-copy/xclip/clip automatically)
mx command wc (alias: mx c wc)

# Create context files with clipboard content (alias: mx t)
mx touch tk   # Creates .mx/tasks.md with clipboard content
mx t rq       # Creates .mx/requirements.md with clipboard content
mx t pdt      # Creates .mx/pending/tasks.md with clipboard content
mx t tk1      # Creates .mx/tasks/tasks1.md (dynamic numbered alias)

# View context file contents (alias: mx ct)
mx cat tk     # Displays contents of .mx/tasks.md
mx ct rq      # Displays contents of .mx/requirements.md
mx ct pdt     # Displays contents of .mx/pending/tasks.md
mx ct docs/spec  # Displays contents of .mx/docs/spec.md

# Force overwrite existing files
mx t tk --force      # Overwrites .mx/tasks.md with clipboard content
mx t tk -f           # Short form

# Clean context files
mx cl             # Deletes the entire .mx/ directory (alias for clean)
mx cl tk          # Deletes only .mx/tasks.md
mx cl tk1         # Deletes only .mx/tasks/tasks1.md

# Dynamic path support
mx t myfile          # Creates .mx/myfile.md (auto-appends .md)
mx t docs/spec       # Creates .mx/docs/spec.md (auto-creates directories)
mx t config.yaml     # Creates .mx/config.yaml (preserves extension)
```

### Context Management Keys (Aliases)

| Key  | Path                        |
|------|-----------------------------|
| df   | `.mx/diff.md`              |
| wn   | `.mx/warnings.md`          |
| is   | `.mx/issue.md`             |
| er   | `.mx/error.md`             |
| rp   | `.mx/report.md`            |
| if   | `.mx/info.md`              |
| aif  | `.mx/additional_info.md`   |
| rq   | `.mx/requirements.md`      |
| pdr  | `.mx/pending/requirements.md`|
| tko  | `.mx/tasks_outline.md`     |
| tk   | `.mx/tasks.md`             |
| pdt  | `.mx/pending/tasks.md`     |
| rv   | `.mx/review.md`            |

### Dynamic Path Resolution

- **Pending Prefix**: `pd-` prefix places the file under `pending/`.
    - `mx t pd-tk` -> `.mx/pending/tasks.md`
    - `mx t pd-feature/spec` -> `.mx/pending/feature/spec.md`
- **Numbered Aliases**: `tk` followed by a number (e.g., `tk1`, `tk2`) maps to `tasks/tasks{N}.md`.

When no alias matches, the input is treated as a relative path:

- **Extension completion**: If no extension is specified, `.md` is automatically appended
- **Directory creation**: Parent directories are created automatically (e.g., `sdd/rq` creates `.mx/sdd/rq.md`)
- **Security**: Path traversal attempts (using `..`) are rejected to prevent creating files outside `.mx/`

### Default Clipboard Paste Behavior

`mx touch` automatically pastes clipboard contents into newly created context files. This is the default behavior.

**Important**: `mx touch` will **not** overwrite existing files by default. It will display a warning `⚠️ Context file already exists`.

To overwrite an existing file, use the `--force` (or `-f`) flag. This will overwrite the file with the current clipboard content.

**Common workflow**:
1. Copy error message or specification from browser
2. Run `mx t er` to save it as `.mx/error.md`
3. View it with `mx ct er` or reference it in your work using template placeholders like `{{.mx/error.md}}`

### Template placeholders (dynamic context)

- Write `{{relative/path.md}}` inside any snippet to inline the referenced file when the snippet is copied.
- Paths are always resolved relative to the current project root (the directory you run `mx` from) and are validated with the same traversal checks as `mx t`.
- Missing files (or invalid paths) are replaced with a readable marker such as `[mx missing: .mx/tasks.md (NotFound)]` so you can tell what went wrong.
- When `mx` runs outside of a project (project root cannot be detected), placeholders stay untouched and copy as literal text.

Example:

```
Current status: {{.mx/tasks.md}}
```

Combine this with the `mx t if`, `mx t rp`, or `mx t aif` aliases to keep context documents fresh and automatically inject their latest contents into prompts.

### Environment overrides

| Variable            | Purpose                                                                                 |
|---------------------|-----------------------------------------------------------------------------------------|
| `MX_COMMANDS_ROOT`  | Override the default `~/.config/mx` root (useful for testing or custom installations). |
| `MX_CLIPBOARD_FILE` | Use a file for clipboard operations (both read and write) instead of system clipboard.  |
| `MX_CLIPBOARD_CMD`  | Provide a custom clipboard command if the auto-detected one is unavailable.             |

## Development guide

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
RUST_TEST_THREADS=1 cargo test --all-targets --all-features
```

The workspace follows the original template's testing culture:

- **Unit tests** live next to their modules (clipboard abstraction, snippet storage, touch).
- **Test support** (`src/commands/test_support.rs`) provides helpers like in-memory clipboard stubs.
- **Integration tests** under `tests/commands/` exercise each CLI command in a separate file (e.g., `copy.rs`, `touch.rs`). Shared helpers in `tests/common/` seed snippet catalogs inside a
  temporary HOME and expose utilities for overriding clipboard/destination paths.
