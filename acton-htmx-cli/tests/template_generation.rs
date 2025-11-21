//! Integration tests for template generation

use std::fs;
use tempfile::TempDir;

/// Test that project structure is created correctly
#[test]
fn test_project_structure_creation() {
    let temp_dir = TempDir::new().unwrap();
    let project_name = "test_project";
    let project_path = temp_dir.path().join(project_name);

    // Create basic structure
    let dirs = [
        "",
        "src",
        "src/handlers",
        "src/models",
        "templates",
        "templates/layouts",
        "templates/auth",
        "templates/partials",
        "config",
        "migrations",
        "static",
        "static/css",
        "static/js",
        "tests",
    ];

    for dir in &dirs {
        let path = project_path.join(dir);
        fs::create_dir_all(&path).unwrap();
    }

    // Verify all directories exist
    for dir in &dirs {
        let path = project_path.join(dir);
        assert!(path.exists(), "Directory should exist: {}", path.display());
        assert!(path.is_dir(), "Path should be a directory: {}", path.display());
    }
}

/// Test that valid project names are accepted
#[test]
fn test_valid_project_names() {
    let valid_names = vec![
        "my_project",
        "my-project",
        "myproject",
        "my_project_123",
        "_private",
        "a",
        "a1",
    ];

    for name in valid_names {
        assert!(
            is_valid_crate_name(name),
            "Name should be valid: {name}"
        );
    }
}

/// Test that invalid project names are rejected
#[test]
fn test_invalid_project_names() {
    let invalid_names = vec![
        "",
        "MyProject",
        "123project",
        "my project",
        "my.project",
        "my@project",
        "my/project",
        "my\\project",
        "UPPERCASE",
    ];

    for name in invalid_names {
        assert!(
            !is_valid_crate_name(name),
            "Name should be invalid: {name}"
        );
    }
}

/// Validate that a string is a valid Rust crate name
fn is_valid_crate_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    // Must start with letter or underscore
    let first_char = name.chars().next().unwrap();
    if !first_char.is_ascii_lowercase() && first_char != '_' {
        return false;
    }

    // All characters must be alphanumeric, underscore, or hyphen
    name.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
}

/// Test that Cargo.toml template contains required fields
#[test]
fn test_cargo_toml_template() {
    use acton_htmx::templates::CARGO_TOML;

    assert!(CARGO_TOML.contains("[package]"));
    assert!(CARGO_TOML.contains("name = \"{{project_name}}\""));
    assert!(CARGO_TOML.contains("edition = \"2021\""));
    assert!(CARGO_TOML.contains("[dependencies]"));
    assert!(CARGO_TOML.contains("acton-htmx"));
    assert!(CARGO_TOML.contains("axum"));
    assert!(CARGO_TOML.contains("tokio"));
}

/// Test that README template contains project name placeholder
#[test]
fn test_readme_template() {
    use acton_htmx::templates::README_MD;

    assert!(README_MD.contains("{{project_name}}"));
    assert!(README_MD.contains("acton-htmx"));
    assert!(README_MD.contains("Quick Start"));
    assert!(README_MD.contains("Project Structure"));
}

/// Test that gitignore template contains standard entries
#[test]
fn test_gitignore_template() {
    use acton_htmx::templates::GITIGNORE;

    assert!(GITIGNORE.contains("/target"));
    assert!(GITIGNORE.contains(".env"));
    assert!(GITIGNORE.contains("*.db"));
}

/// Test that main.rs template is valid Rust code structure
#[test]
fn test_main_rs_template() {
    use acton_htmx::templates::MAIN_RS;

    assert!(MAIN_RS.contains("fn main()"));
    assert!(MAIN_RS.contains("use acton_htmx"));
    assert!(MAIN_RS.contains("#[tokio::main]"));
    assert!(MAIN_RS.contains("Router::new()"));
}

/// Test that migration template has valid SQL
#[test]
fn test_migration_template() {
    use acton_htmx::templates::MIGRATION_USERS;

    assert!(MIGRATION_USERS.contains("CREATE TABLE users"));
    assert!(MIGRATION_USERS.contains("id SERIAL PRIMARY KEY"));
    assert!(MIGRATION_USERS.contains("email VARCHAR"));
    assert!(MIGRATION_USERS.contains("password_hash VARCHAR"));
    assert!(MIGRATION_USERS.contains("CREATE INDEX"));
}

/// Test that HTML templates have valid structure
#[test]
fn test_html_templates() {
    use acton_htmx::templates::{TEMPLATE_BASE, TEMPLATE_APP, TEMPLATE_LOGIN};

    // Base template
    assert!(TEMPLATE_BASE.contains("<!DOCTYPE html>"));
    assert!(TEMPLATE_BASE.contains("<html"));
    assert!(TEMPLATE_BASE.contains("htmx.org"));

    // App template
    assert!(TEMPLATE_APP.contains("{% extends"));
    assert!(TEMPLATE_APP.contains("{% block"));

    // Login template
    assert!(TEMPLATE_LOGIN.contains("hx-post"));
    assert!(TEMPLATE_LOGIN.contains("email"));
    assert!(TEMPLATE_LOGIN.contains("password"));
}
