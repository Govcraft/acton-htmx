//! Database management commands

use anyhow::{Context, Result};
use console::style;
use std::process::{Command, Stdio};

/// Database command variants
pub enum DbCommand {
    /// Run pending migrations
    Migrate,
    /// Reset database (drop, create, migrate)
    Reset,
    /// Create a new migration file
    Create {
        /// Name of the migration to create
        name: String,
    },
}

impl DbCommand {
    /// Execute the command
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - `sqlx-cli` is not installed
    /// - Database operations fail
    pub fn execute(&self) -> Result<()> {
        // Check if sqlx-cli is installed
        if !Self::is_sqlx_cli_installed() {
            println!(
                "{} is not installed.",
                style("sqlx-cli").yellow().bold()
            );
            println!();
            println!("Install it with:");
            println!(
                "  {} {}",
                style("$").dim(),
                style("cargo install sqlx-cli --no-default-features --features postgres").cyan()
            );
            println!();
            anyhow::bail!("sqlx-cli is required for database commands");
        }

        match self {
            Self::Migrate => Self::migrate(),
            Self::Reset => Self::reset(),
            Self::Create { name } => Self::create(name),
        }
    }

    /// Run pending migrations
    fn migrate() -> Result<()> {
        println!(
            "{} {}",
            style("Running").green().bold(),
            style("database migrations...").bold()
        );
        println!();

        let status = Command::new("sqlx")
            .args(["migrate", "run"])
            .status()
            .context("Failed to run migrations")?;

        if !status.success() {
            anyhow::bail!("Migration failed");
        }

        println!();
        println!("{}", style("✓ Migrations completed successfully!").green().bold());

        Ok(())
    }

    /// Reset database (drop, create, migrate)
    fn reset() -> Result<()> {
        println!(
            "{} {}",
            style("Resetting").yellow().bold(),
            style("database...").bold()
        );
        println!();

        // Drop database
        println!("  {} Dropping database...", style("1.").cyan());
        let status = Command::new("sqlx")
            .args(["database", "drop", "-y"])
            .status()
            .context("Failed to drop database")?;

        if !status.success() {
            println!("    {} Database may not exist (continuing)", style("!").yellow());
        }

        // Create database
        println!("  {} Creating database...", style("2.").cyan());
        let status = Command::new("sqlx")
            .args(["database", "create"])
            .status()
            .context("Failed to create database")?;

        if !status.success() {
            anyhow::bail!("Failed to create database");
        }

        // Run migrations
        println!("  {} Running migrations...", style("3.").cyan());
        let status = Command::new("sqlx")
            .args(["migrate", "run"])
            .status()
            .context("Failed to run migrations")?;

        if !status.success() {
            anyhow::bail!("Failed to run migrations");
        }

        println!();
        println!("{}", style("✓ Database reset successfully!").green().bold());

        Ok(())
    }

    /// Create a new migration file
    fn create(name: &str) -> Result<()> {
        println!(
            "{} {}",
            style("Creating").green().bold(),
            style(format!("migration: {name}")).bold()
        );
        println!();

        let status = Command::new("sqlx")
            .args(["migrate", "add", name])
            .status()
            .context("Failed to create migration")?;

        if !status.success() {
            anyhow::bail!("Failed to create migration");
        }

        println!();
        println!(
            "{}",
            style("✓ Migration file created in migrations/").green().bold()
        );

        Ok(())
    }

    /// Check if sqlx-cli is installed
    fn is_sqlx_cli_installed() -> bool {
        Command::new("sqlx")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }
}
