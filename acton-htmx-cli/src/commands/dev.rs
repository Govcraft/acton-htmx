//! Development server command

use anyhow::{Context, Result};
use console::style;
use std::process::{Command, Stdio};

/// Start development server with hot reload
pub struct DevCommand;

impl DevCommand {
    /// Create a new command instance
    pub const fn new() -> Self {
        Self
    }

    /// Execute the command
    pub fn execute() -> Result<()> {
        println!(
            "{} {}",
            style("Starting").green().bold(),
            style("development server...").bold()
        );
        println!();

        // Check if cargo-watch is installed
        if !Self::is_cargo_watch_installed() {
            println!(
                "{} is not installed.",
                style("cargo-watch").yellow().bold()
            );
            println!();
            println!("Install it with:");
            println!("  {} {}", style("$").dim(), style("cargo install cargo-watch").cyan());
            println!();
            println!("For now, starting without hot reload...");
            println!();

            return Self::run_without_watch();
        }

        // Run with cargo-watch for hot reload
        Self::run_with_watch()
    }

    /// Check if cargo-watch is installed
    fn is_cargo_watch_installed() -> bool {
        Command::new("cargo")
            .arg("watch")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }

    /// Run with cargo-watch for hot reload
    fn run_with_watch() -> Result<()> {
        println!(
            "{}",
            style("Hot reload enabled. Watching for changes...").green()
        );
        println!();

        let mut child = Command::new("cargo")
            .args([
                "watch",
                "-x",
                "run",
                "-w",
                "src",
                "-w",
                "templates",
                "-w",
                "config",
            ])
            .spawn()
            .context("Failed to start cargo watch")?;

        // Wait for the process to complete
        let status = child.wait().context("Failed to wait for cargo watch")?;

        if !status.success() {
            anyhow::bail!("Development server exited with error");
        }

        Ok(())
    }

    /// Run without hot reload (fallback)
    fn run_without_watch() -> Result<()> {
        let mut child = Command::new("cargo")
            .arg("run")
            .spawn()
            .context("Failed to start development server")?;

        let status = child
            .wait()
            .context("Failed to wait for development server")?;

        if !status.success() {
            anyhow::bail!("Development server exited with error");
        }

        Ok(())
    }
}

impl Default for DevCommand {
    fn default() -> Self {
        Self::new()
    }
}
