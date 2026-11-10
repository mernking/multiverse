//! Creative Engine CLI
//!
//! Command-line interface for the Creative Engine.

use clap::Parser;

#[derive(Parser)]
#[command(name = "creative", version, about = "Creative Engine CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Initialize a new project
    Init { path: String },
    /// Show version
    Version,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init { path } => {
            println!("Initializing project at: {}", path);
        }
        Commands::Version => {
            println!("Creative Engine CLI v0.1.0");
        }
    }
}