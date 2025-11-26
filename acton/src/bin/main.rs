//! Acton CLI - Unified command-line interface for Acton framework
//!
//! # Usage
//!
//! ```bash
//! # HTMX web framework commands
//! acton htmx new my-app
//! acton htmx dev
//! acton htmx scaffold crud Post title:string content:text
//! ```

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "acton")]
#[command(version)]
#[command(about = "Acton framework CLI - build web applications in Rust", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// HTMX web framework commands
    Htmx {
        #[command(subcommand)]
        command: acton::cli::HtmxCommand,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Htmx { command } => acton::cli::htmx::run(command),
    }
}
