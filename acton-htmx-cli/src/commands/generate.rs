//! Code generation commands (jobs, models, etc.)

use anyhow::{bail, Context, Result};
use clap::Subcommand;
use console::{style, Emoji};
use convert_case::{Case, Casing};
use handlebars::Handlebars;
use serde_json::json;
use std::fs;
use std::path::PathBuf;

use crate::templates::JOB_TEMPLATE;

static SUCCESS: Emoji = Emoji("✓", "√");

/// Code generation commands
#[derive(Debug, Subcommand)]
pub enum GenerateCommand {
    /// Generate a new background job
    ///
    /// Examples:
    ///   acton-htmx generate job WelcomeEmail user_id:i64 email:string
    ///   acton-htmx generate job GenerateReport report_id:i64 --priority=high
    ///   acton-htmx generate job CleanupOldData days:u32 --timeout=600
    Job {
        /// Job name (PascalCase, will be suffixed with 'Job')
        name: String,

        /// Job fields in format: name:type
        /// Supported types: i8, i16, i32, i64, u8, u16, u32, u64, f32, f64,
        /// bool, string, vec_string, option_string, etc.
        #[arg(value_name = "FIELD:TYPE")]
        fields: Vec<String>,

        /// Maximum number of retries (default: 3)
        #[arg(long, default_value = "3")]
        max_retries: u32,

        /// Timeout in seconds (default: 300)
        #[arg(long, default_value = "300")]
        timeout: u64,

        /// Job priority (higher = more priority, default: 128)
        #[arg(long, default_value = "128")]
        priority: i32,

        /// Output directory (default: src/jobs)
        #[arg(short, long, default_value = "src/jobs")]
        output: PathBuf,
    },
}

impl GenerateCommand {
    /// Execute the generate command
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Invalid field format
    /// - Failed to create output directory
    /// - Failed to write generated file
    pub fn execute(&self) -> Result<()> {
        match self {
            Self::Job {
                name,
                fields,
                max_retries,
                timeout,
                priority,
                output,
            } => {
                self.generate_job(name, fields, *max_retries, *timeout, *priority, output)
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn generate_job(
        &self,
        name: &str,
        fields: &[String],
        max_retries: u32,
        timeout: u64,
        priority: i32,
        output: &PathBuf,
    ) -> Result<()> {
        println!(
            "\n{} Generating background job: {}",
            style("📦").bold(),
            style(name).cyan().bold()
        );

        // Parse and validate job name
        let job_name = name.to_case(Case::Pascal);
        let job_name_snake = job_name.to_case(Case::Snake);

        // Parse fields
        let parsed_fields = self.parse_fields(fields)?;

        // Check if we need database or email dependencies
        let needs_db = parsed_fields
            .iter()
            .any(|f| f.rust_type.contains("Pool") || f.rust_type.contains("Connection"));
        let needs_email = job_name.to_lowercase().contains("email");

        // Prepare template context
        let context = json!({
            "job_name": job_name,
            "job_name_snake": job_name_snake,
            "job_description": format!("Background job for {}", job_name_snake.replace('_', " ")),
            "fields": parsed_fields,
            "needs_db": needs_db,
            "needs_email": needs_email,
            "result_type": "()",
            "result_default": "()",
            "max_retries": max_retries,
            "timeout_secs": timeout,
            "priority": priority,
        });

        // Render template
        let mut handlebars = Handlebars::new();
        handlebars.register_escape_fn(handlebars::no_escape);

        let rendered = handlebars
            .render_template(JOB_TEMPLATE, &context)
            .context("Failed to render job template")?;

        // Create output directory if it doesn't exist
        fs::create_dir_all(output).context("Failed to create output directory")?;

        // Write job file
        let job_file = output.join(format!("{job_name_snake}.rs"));
        fs::write(&job_file, rendered)
            .with_context(|| format!("Failed to write job file: {}", job_file.display()))?;

        println!();
        println!(
            "  {} Created job file: {}",
            SUCCESS,
            style(job_file.display()).green()
        );

        // Show next steps
        println!();
        println!("{}", style("Next steps:").bold().underlined());
        println!("  1. Add to src/jobs/mod.rs:");
        println!(
            "     {}",
            style(format!("pub mod {job_name_snake};")).cyan()
        );
        println!(
            "     {}",
            style(format!("pub use {job_name_snake}::{job_name}Job;")).cyan()
        );
        println!();
        println!("  2. Implement the execute() method logic");
        println!();
        println!("  3. Enqueue the job from your handlers:");
        println!(
            "     {}",
            style(format!(
                "state.jobs.enqueue({job_name}Job {{ ... }}).await?;"
            ))
            .cyan()
        );
        println!();

        Ok(())
    }

    fn parse_fields(&self, fields: &[String]) -> Result<Vec<FieldDefinition>> {
        fields.iter().map(|f| self.parse_field(f)).collect()
    }

    fn parse_field(&self, field: &str) -> Result<FieldDefinition> {
        let parts: Vec<&str> = field.split(':').collect();

        if parts.len() != 2 {
            bail!("Invalid field format: '{}'. Expected 'name:type'", field);
        }

        let name = parts[0].to_case(Case::Snake);
        let type_str = parts[1];

        let (rust_type, test_value) = self.map_type(type_str)?;

        Ok(FieldDefinition {
            name,
            rust_type,
            test_value,
            doc: None,
        })
    }

    fn map_type(&self, type_str: &str) -> Result<(String, String)> {
        let (rust_type, test_value) = match type_str.to_lowercase().as_str() {
            "i8" => ("i8", "0_i8"),
            "i16" => ("i16", "0_i16"),
            "i32" => ("i32", "0_i32"),
            "i64" => ("i64", "0_i64"),
            "u8" => ("u8", "0_u8"),
            "u16" => ("u16", "0_u16"),
            "u32" => ("u32", "0_u32"),
            "u64" => ("u64", "0_u64"),
            "f32" => ("f32", "0.0_f32"),
            "f64" => ("f64", "0.0_f64"),
            "bool" | "boolean" => ("bool", "false"),
            "str" | "string" => ("String", r#"String::from("test")"#),
            "vec_string" | "vec<string>" => ("Vec<String>", "vec![]"),
            "option_string" | "option<string>" => ("Option<String>", "None"),
            "option_i64" | "option<i64>" => ("Option<i64>", "None"),
            _ => bail!("Unsupported type: '{}'. Supported types: i8, i16, i32, i64, u8, u16, u32, u64, f32, f64, bool, string, vec_string, option_string, option_i64", type_str),
        };

        Ok((rust_type.to_string(), test_value.to_string()))
    }
}

#[derive(Debug, Clone, serde::Serialize)]
struct FieldDefinition {
    name: String,
    rust_type: String,
    test_value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    doc: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_field_i64() {
        let gen = GenerateCommand::Job {
            name: "Test".to_string(),
            fields: vec![],
            max_retries: 3,
            timeout: 300,
            priority: 128,
            output: PathBuf::from("src/jobs"),
        };

        let field = gen.parse_field("user_id:i64").unwrap();
        assert_eq!(field.name, "user_id");
        assert_eq!(field.rust_type, "i64");
        assert_eq!(field.test_value, "0_i64");
    }

    #[test]
    fn test_parse_field_string() {
        let gen = GenerateCommand::Job {
            name: "Test".to_string(),
            fields: vec![],
            max_retries: 3,
            timeout: 300,
            priority: 128,
            output: PathBuf::from("src/jobs"),
        };

        let field = gen.parse_field("email:string").unwrap();
        assert_eq!(field.name, "email");
        assert_eq!(field.rust_type, "String");
        assert_eq!(field.test_value, r#"String::from("test")"#);
    }

    #[test]
    fn test_parse_field_invalid_format() {
        let gen = GenerateCommand::Job {
            name: "Test".to_string(),
            fields: vec![],
            max_retries: 3,
            timeout: 300,
            priority: 128,
            output: PathBuf::from("src/jobs"),
        };

        let result = gen.parse_field("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_map_type_unsupported() {
        let gen = GenerateCommand::Job {
            name: "Test".to_string(),
            fields: vec![],
            max_retries: 3,
            timeout: 300,
            priority: 128,
            output: PathBuf::from("src/jobs"),
        };

        let result = gen.map_type("unsupported");
        assert!(result.is_err());
    }
}
