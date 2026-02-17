mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "slimg", version, about = "Image optimization CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert image to a different format with compression
    Convert(commands::convert::ConvertArgs),
    /// Optimize image in the same format (reduce file size)
    Optimize(commands::optimize::OptimizeArgs),
    /// Resize image with optional format conversion
    Resize(commands::resize::ResizeArgs),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Convert(args) => commands::convert::run(args),
        Commands::Optimize(args) => commands::optimize::run(args),
        Commands::Resize(args) => commands::resize::run(args),
    }
}
