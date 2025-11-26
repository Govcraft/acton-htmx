//! Acton DX CLI - Developer experience focused web framework CLI
//!
//! # Usage
//!
//! ```bash
//! # HTMX web framework commands
//! acton-dx htmx new my-app
//! acton-dx htmx dev
//! acton-dx htmx scaffold crud Post title:string content:text
//! ```

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "acton-dx")]
#[command(version)]
#[command(about = "Acton DX - Developer experience focused web framework for Rust", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// HTMX web framework commands
    Htmx {
        #[command(subcommand)]
        command: acton_dx::cli::HtmxCommand,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Htmx { command } => acton_dx::cli::htmx::run(command),
    }
}
