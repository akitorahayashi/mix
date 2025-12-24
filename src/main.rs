use clap::{CommandFactory, Parser, Subcommand};
use mix::commands::{self, CopyOutcome, ListEntry};
use mix::error::AppError;

#[derive(Parser)]
#[command(name = "mix")]
#[command(version)]
#[command(about = "Unified CLI for mix snippets and slash command generation")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Snippet name to copy when no subcommand is provided
    snippet: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// List all available snippets
    #[command(visible_alias = "ls")]
    List,
    /// Create context files
    #[command(visible_alias = "t")]
    Touch {
        key: String,
        /// Paste content from clipboard into the new file
        #[arg(short = 'p', long = "paste")]
        paste: bool,
        /// Force overwrite existing files
        #[arg(short = 'f', long = "force")]
        force: bool,
    },
    /// Clean context files or directory
    #[command(visible_alias = "cl")]
    Clean {
        /// Optional key to clean a specific file (e.g., 'tk', 'tk1')
        key: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match (cli.command, cli.snippet) {
        (Some(Commands::List), _) => handle_list(),
        (Some(Commands::Touch { key, paste, force }), _) => handle_touch(&key, paste, force),
        (Some(Commands::Clean { key }), _) => handle_clean(key),
        (None, Some(snippet)) => handle_copy(&snippet),
        (None, None) => {
            Cli::command().print_help().ok();
            println!();
            Ok(())
        }
    };

    if let Err(err) = result {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

fn handle_copy(name: &str) -> Result<(), AppError> {
    let CopyOutcome { key, relative_path, absolute_path } = commands::copy_snippet(name)?;
    println!("✅ Copied '{key}' from {relative_path} -> {}", absolute_path.display());
    Ok(())
}

fn handle_touch(key: &str, paste: bool, force: bool) -> Result<(), AppError> {
    let outcome = commands::touch_context(key, paste, force)?;

    if outcome.overwritten {
        println!("✅ Context file overwritten: {}", outcome.path.display());
    } else if outcome.existed {
        println!("⚠️ Context file already exists: {}", outcome.path.display());
    } else {
        println!("✅ Context file created: {}", outcome.path.display());
    }

    Ok(())
}

fn handle_clean(key: Option<String>) -> Result<(), AppError> {
    let outcome = commands::clean_context(key)?;
    println!("✅ {}", outcome.message);
    Ok(())
}

fn handle_list() -> Result<(), AppError> {
    let entries = commands::list_snippets()?;
    if entries.is_empty() {
        println!("(no snippets found)");
        return Ok(());
    }

    println!("📚 Available snippets:\n");
    for ListEntry { key, relative_path, title, description } in entries {
        println!("- {key} ({relative_path})");
        if let Some(title) = title {
            println!("  • {title}");
        }
        if let Some(description) = description {
            println!("    {description}");
        }
    }
    Ok(())
}
