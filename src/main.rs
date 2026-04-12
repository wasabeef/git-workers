//! Git Workers - Interactive Git Worktree Manager

use anyhow::Result;
use clap::Parser;

use git_workers::app;

/// Command-line arguments for Git Workers
///
/// Currently supports minimal CLI arguments as the application is primarily
/// interactive. Future versions may add support for direct command execution.
#[derive(Parser)]
#[command(name = "gw")]
#[command(about = "Interactive Git Worktree Manager", long_about = None)]
struct Cli {
    /// Print version information and exit
    ///
    /// When specified, prints the version number from Cargo.toml and exits
    /// without entering the interactive mode.
    #[arg(short, long)]
    version: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.version {
        let version = env!("CARGO_PKG_VERSION");
        println!("git-workers v{version}");
        return Ok(());
    }

    app::run()
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn test_help_includes_version_description() {
        let mut command = Cli::command();
        let help = command.render_long_help().to_string();

        assert!(help.contains("Print version information and exit"));
        assert!(
            help.contains("When specified, prints the version number from Cargo.toml and exits")
        );
    }
}
