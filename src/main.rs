use clap::{Parser, Subcommand};

mod commands;
mod platform;

#[derive(Parser)]
#[command(name = "kiln", about = "An opinionated project scaffolding CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new project
    Create,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Create => commands::create::run(),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
