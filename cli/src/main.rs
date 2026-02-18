mod commands;

use std::io;

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;

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
    /// Crop image with optional format conversion
    Crop(commands::crop::CropArgs),
    /// Extend image by adding padding with optional format conversion
    Extend(commands::extend::ExtendArgs),
    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Convert(args) => commands::convert::run(args),
        Commands::Optimize(args) => commands::optimize::run(args),
        Commands::Resize(args) => commands::resize::run(args),
        Commands::Crop(args) => commands::crop::run(args),
        Commands::Extend(args) => commands::extend::run(args),
        Commands::Completions { shell } => {
            clap_complete::generate(shell, &mut Cli::command(), "slimg", &mut io::stdout());
            Ok(())
        }
    }
}
