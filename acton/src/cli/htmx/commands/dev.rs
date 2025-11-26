//! Development server command

use anyhow::{Context, Result};
use console::style;
use std::path::Path;
use std::process::{Command, Stdio};

/// Start development server with hot reload
pub struct DevCommand;

impl DevCommand {
    /// Create a new command instance
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Execute the command in the specified directory
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The project directory doesn't exist or is not a valid project
    /// - The development server fails to start
    pub fn execute(path: &Path) -> Result<()> {
        // Canonicalize the path to get absolute path and verify it exists
        let project_dir = path
            .canonicalize()
            .with_context(|| format!("Project directory not found: {}", path.display()))?;

        // Verify Cargo.toml exists in the target directory
        if !project_dir.join("Cargo.toml").exists() {
            anyhow::bail!(
                "No Cargo.toml found in {}. Is this an Acton HTMX project?",
                project_dir.display()
            );
        }

        println!(
            "{} {} in {}",
            style("Starting").green().bold(),
            style("development server").bold(),
            style(project_dir.display()).cyan()
        );
        println!();

        // Check if bacon is installed
        if !Self::is_bacon_installed() {
            println!(
                "{} is not installed.",
                style("bacon").yellow().bold()
            );
            println!();
            println!("Install it with:");
            println!(
                "  {} {}",
                style("$").dim(),
                style("cargo install bacon").cyan()
            );
            println!();
            println!("For now, starting without hot reload...");
            println!();

            return Self::run_without_watch(&project_dir);
        }

        // Run with bacon for hot reload
        Self::run_with_bacon(&project_dir)
    }

    /// Check if bacon is installed
    fn is_bacon_installed() -> bool {
        Command::new("bacon")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }

    /// Run with bacon for hot reload
    fn run_with_bacon(project_dir: &Path) -> Result<()> {
        println!(
            "{}",
            style("Hot reload enabled via bacon. Watching for changes...").green()
        );
        println!();

        let mut child = Command::new("bacon")
            .arg("run")
            .current_dir(project_dir)
            .spawn()
            .context("Failed to start bacon")?;

        // Wait for the process to complete
        let status = child.wait().context("Failed to wait for bacon")?;

        if !status.success() {
            anyhow::bail!("Development server exited with error");
        }

        Ok(())
    }

    /// Run without hot reload (fallback)
    fn run_without_watch(project_dir: &Path) -> Result<()> {
        let mut child = Command::new("cargo")
            .arg("run")
            .current_dir(project_dir)
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
