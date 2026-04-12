//! Git Workers - Interactive Git Worktree Manager

use anyhow::Result;
use clap::Parser;

use git_workers::app;

#[derive(Parser)]
#[command(name = "gw")]
#[command(about = "Interactive Git Worktree Manager", long_about = None)]
struct Cli {
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
