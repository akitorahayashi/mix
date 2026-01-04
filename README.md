# mx

`mx` is a Rust CLI that unifies two daily workflows:

1. **Snippet command** – type `mx command <snippet>` (alias `mx c`) to stream any markdown snippet under
   `~/.config/mx/commands/` into your clipboard.
2. **Context Orchestration** – type `mx touch <key>` (alias `mx t`) to create context files in your project with clipboard content.

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
mx touch tk   # Creates mix/tasks.md with clipboard content
mx t rq       # Creates mix/requirements.md with clipboard content
mx t pdt      # Creates mix/pending/tasks.md with clipboard content
mx t tk1      # Creates mix/tasks/tasks1.md (dynamic numbered alias)

# Force overwrite existing files
mx t tk --force      # Overwrites mix/tasks.md with clipboard content
mx t tk -f           # Short form

# Clean context files
mx cl             # Deletes the entire mix/ directory (alias for clean)
mx cl tk          # Deletes only mix/tasks.md
mx cl tk1         # Deletes only mix/tasks/tasks1.md

# Dynamic path support
mx t myfile          # Creates mix/myfile.md (auto-appends .md)
mx t docs/spec       # Creates mix/docs/spec.md (auto-creates directories)
mx t config.yaml     # Creates mix/config.yaml (preserves extension)
```

### Context Management Keys (Aliases)

| Key  | Path                        |
|------|-----------------------------|
| df   | `mix/diff.md`              |
| wn   | `mix/warnings.md`          |
| is   | `mix/issue.md`             |
| er   | `mix/error.md`             |
| rp   | `mix/report.md`            |
| if   | `mix/info.md`              |
| aif  | `mix/additional_info.md`   |
| rq   | `mix/requirements.md`      |
| pdr  | `mix/pending/requirements.md`|
| tko  | `mix/tasks_outline.md`     |
| tk   | `mix/tasks.md`             |
| pdt  | `mix/pending/tasks.md`     |
| rv   | `mix/review.md`            |

### Dynamic Path Resolution

- **Pending Prefix**: `pd-` prefix places the file under `pending/`.
    - `mx t pd-tk` -> `mix/pending/tasks.md`
    - `mx t pd-feature/spec` -> `mix/pending/feature/spec.md`
- **Numbered Aliases**: `tk` followed by a number (e.g., `tk1`, `tk2`) maps to `tasks/tasks{N}.md`.

When no alias matches, the input is treated as a relative path:

- **Extension completion**: If no extension is specified, `.md` is automatically appended
- **Directory creation**: Parent directories are created automatically (e.g., `sdd/rq` creates `mix/sdd/rq.md`)
- **Security**: Path traversal attempts (using `..`) are rejected to prevent creating files outside `mix/`

### Default Clipboard Paste Behavior

`mx touch` automatically pastes clipboard contents into newly created context files. This is the default behavior.

**Important**: `mx touch` will **not** overwrite existing files by default. It will display a warning `⚠️ Context file already exists`.

To overwrite an existing file, use the `--force` (or `-f`) flag. This will overwrite the file with the current clipboard content.

**Common workflow**:
1. Copy error message or specification from browser
2. Run `mx t er` to save it as `mix/error.md`
3. Reference it in your work or use template placeholders like `{{mix/error.md}}`

### Template placeholders (dynamic context)

- Write `{{relative/path.md}}` inside any snippet to inline the referenced file when the snippet is copied.
- Paths are always resolved relative to the current project root (the directory you run `mx` from) and are validated with the same traversal checks as `mx t`.
- Missing files (or invalid paths) are replaced with a readable marker such as `[mx missing: mix/tasks.md (NotFound)]` so you can tell what went wrong.
- When `mx` runs outside of a project (project root cannot be detected), placeholders stay untouched and copy as literal text.

Example:

```
Current status: {{mix/tasks.md}}
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
- **Integration crates** under `tests/` exercise both the CLI (`cli_commands.rs`, `cli_flow.rs`, `cli_touch.rs`) and the
  public library API (`commands_api.rs`). Shared helpers in `tests/common/` seed snippet catalogs inside a
  temporary HOME and expose utilities for overriding clipboard/destination paths.
