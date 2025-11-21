//! CRUD scaffold generator orchestrator
//!
//! This module coordinates the generation of all files for a CRUD resource.
//! It uses the field type system, template system, and helpers to generate:
//! - Models
//! - Migrations
//! - Forms
//! - Handlers
//! - Templates
//! - Tests
//! - Route registration

use super::field_type::FieldDefinition;
use super::helpers::TemplateHelpers;
use super::templates::TemplateRegistry;
use anyhow::{Context, Result};
use std::path::PathBuf;

/// CRUD scaffold generator
#[allow(dead_code)] // Fields will be used in Week 2-3 implementation
pub struct ScaffoldGenerator {
    /// Model name (e.g., "Post", "UserProfile")
    model_name: String,
    /// Field definitions
    fields: Vec<FieldDefinition>,
    /// Template registry
    templates: TemplateRegistry,
    /// Project root directory
    project_root: PathBuf,
}

impl ScaffoldGenerator {
    /// Create a new scaffold generator
    ///
    /// # Arguments
    ///
    /// * `model_name` - Name of the model (e.g., "Post", "UserProfile")
    /// * `field_specs` - Field specifications (e.g., ["title:string", "content:text"])
    /// * `project_root` - Project root directory
    pub fn new(
        model_name: String,
        field_specs: Vec<String>,
        project_root: PathBuf,
    ) -> Result<Self> {
        // Validate model name
        if !model_name.chars().next().unwrap_or('0').is_uppercase() {
            anyhow::bail!(
                "Model name must be PascalCase (start with uppercase): '{}'",
                model_name
            );
        }

        // Parse field definitions
        let fields = field_specs
            .iter()
            .map(|spec| FieldDefinition::parse(spec))
            .collect::<Result<Vec<_>>>()
            .context("Failed to parse field definitions")?;

        if fields.is_empty() {
            anyhow::bail!("At least one field must be specified");
        }

        let templates = TemplateRegistry::new()?;

        Ok(Self {
            model_name,
            fields,
            templates,
            project_root,
        })
    }

    /// Generate all CRUD files
    ///
    /// This orchestrates the generation of:
    /// 1. Model file (src/models/{model}.rs)
    /// 2. Migration file (migrations/{timestamp}_create_{table}.sql)
    /// 3. Form file (src/forms/{model}.rs)
    /// 4. Handler file (src/handlers/{model}s.rs) - coming soon
    /// 5. Template files (templates/{model}s/*.html) - coming soon
    /// 6. Test file (tests/{model}s_test.rs) - coming soon
    /// 7. Updates to main.rs for route registration - coming soon
    pub fn generate(&self) -> Result<Vec<GeneratedFile>> {
        let mut generated_files = Vec::new();

        // Generate model, migration, and form files
        generated_files.push(self.generate_model()?);
        generated_files.push(self.generate_migration()?);
        generated_files.push(self.generate_forms()?);

        // Handler & Template generation coming soon
        // generated_files.push(self.generate_handlers()?);
        // generated_files.extend(self.generate_templates()?);
        // generated_files.push(self.generate_tests()?);
        // generated_files.push(self.update_routes()?);

        Ok(generated_files)
    }

    /// Get model metadata for templates
    ///
    /// This generates all the template variables needed for code generation
    fn model_metadata(&self) -> serde_json::Value {
        use super::field_type::FieldType;

        let table_name = TemplateHelpers::to_table_name(&self.model_name);
        let model_snake = TemplateHelpers::to_snake_case(&self.model_name);

        // Collect enum types
        let mut enums = Vec::new();
        for field in &self.fields {
            if let FieldType::Enum { name, variants } = &field.field_type {
                enums.push(serde_json::json!({
                    "name": name,
                    "variants": variants,
                }));
            }
        }

        // Check for special field types
        let has_date_fields = self.fields.iter().any(|f| {
            matches!(
                f.field_type,
                FieldType::Date | FieldType::DateTime | FieldType::Timestamp
            )
        });
        let has_decimal = self.fields.iter().any(|f| matches!(f.field_type, FieldType::Decimal));
        let has_uuid = self.fields.iter().any(|f| matches!(f.field_type, FieldType::Uuid));
        let has_enum = !enums.is_empty();

        // Collect relations
        let mut relations = Vec::new();
        let mut foreign_keys = Vec::new();
        for field in &self.fields {
            if let FieldType::Reference { model } = &field.field_type {
                let referenced_table = TemplateHelpers::to_table_name(model);
                let relation_name = TemplateHelpers::to_pascal_case(&field.name);
                let field_column = TemplateHelpers::to_foreign_key(&field.name, model);

                relations.push(serde_json::json!({
                    "field_name": field.name,
                    "relation_name": relation_name,
                    "referenced_table": referenced_table,
                    "field_column": field_column,
                }));

                foreign_keys.push(serde_json::json!({
                    "field_name": field.name,
                    "column_name": field_column,
                    "referenced_table": referenced_table,
                }));
            }
        }

        // Collect unique and indexed fields
        let unique_fields: Vec<_> = self
            .fields
            .iter()
            .filter(|f| f.unique)
            .map(|f| {
                serde_json::json!({
                    "name": f.name,
                    "column_name": TemplateHelpers::to_snake_case(&f.name),
                })
            })
            .collect();

        let indexed_fields: Vec<_> = self
            .fields
            .iter()
            .filter(|f| f.indexed && !f.unique) // Don't double-index unique fields
            .map(|f| {
                serde_json::json!({
                    "name": f.name,
                    "column_name": TemplateHelpers::to_snake_case(&f.name),
                })
            })
            .collect();

        // Build field metadata with additional info
        let fields: Vec<_> = self
            .fields
            .iter()
            .map(|f| {
                let column_name = if let FieldType::Reference { model } = &f.field_type {
                    TemplateHelpers::to_foreign_key(&f.name, model)
                } else {
                    TemplateHelpers::to_snake_case(&f.name)
                };

                let validations = self.get_validations(f);
                let default_value = self.get_default_value(f);

                serde_json::json!({
                    "name": f.name,
                    "column_name": column_name,
                    "rust_type": f.rust_type(),
                    "sql_type": f.sql_type(),
                    "optional": f.optional,
                    "unique": f.unique,
                    "indexed": f.indexed,
                    "validations": validations,
                    "default_value": default_value,
                })
            })
            .collect();

        serde_json::json!({
            "model_name": self.model_name,
            "model_snake": model_snake,
            "model_plural": TemplateHelpers::pluralize(&self.model_name),
            "table_name": table_name,
            "route_path": TemplateHelpers::to_route_path(&self.model_name),
            "title": TemplateHelpers::to_title(&self.model_name),
            "plural_title": TemplateHelpers::to_plural_title(&self.model_name),
            "fields": fields,
            "relations": relations,
            "foreign_keys": foreign_keys,
            "unique_fields": unique_fields,
            "indexed_fields": indexed_fields,
            "enums": enums,
            "has_date_fields": has_date_fields,
            "has_decimal": has_decimal,
            "has_uuid": has_uuid,
            "has_enum": has_enum,
        })
    }

    /// Get validation rules for a field
    fn get_validations(&self, field: &FieldDefinition) -> Vec<String> {
        use super::field_type::FieldType;

        let mut validations = Vec::new();

        match &field.field_type {
            FieldType::String if !field.optional => {
                validations.push("length(min = 1, max = 255)".to_string());
            }
            FieldType::Text if !field.optional => {
                validations.push("length(min = 1)".to_string());
            }
            _ => {}
        }

        // Add email validation if field name suggests email
        if field.name.to_lowercase().contains("email") {
            validations.push("email".to_string());
        }

        validations
    }

    /// Get default value for testing
    fn get_default_value(&self, field: &FieldDefinition) -> String {
        use super::field_type::FieldType;

        match &field.field_type {
            FieldType::String | FieldType::Text => "\"test\".to_string()".to_string(),
            FieldType::Integer => "0".to_string(),
            FieldType::BigInt => "0".to_string(),
            FieldType::Boolean => "false".to_string(),
            FieldType::Float => "0.0".to_string(),
            FieldType::Double => "0.0".to_string(),
            FieldType::Decimal => "rust_decimal::Decimal::ZERO".to_string(),
            FieldType::Date => "chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()".to_string(),
            FieldType::DateTime => {
                "chrono::NaiveDateTime::from_timestamp_opt(0, 0).unwrap()".to_string()
            }
            FieldType::Timestamp => "chrono::Utc::now()".to_string(),
            FieldType::Json => "serde_json::json!({})".to_string(),
            FieldType::Uuid => "uuid::Uuid::new_v4()".to_string(),
            FieldType::Reference { .. } => "1".to_string(),
            FieldType::Array { .. } => "vec![]".to_string(),
            FieldType::Enum { name, variants } => {
                format!("{}::{}", name, variants.first().unwrap_or(&"Unknown".to_string()))
            }
        }
    }

    /// Generate SeaORM model file
    fn generate_model(&self) -> Result<GeneratedFile> {
        let metadata = self.model_metadata();
        let content = self.templates.render("model", &metadata)?;

        let model_snake = TemplateHelpers::to_snake_case(&self.model_name);
        let path = PathBuf::from(format!("src/models/{}.rs", model_snake));

        Ok(GeneratedFile {
            path,
            content,
            description: format!("SeaORM model for {}", self.model_name),
        })
    }

    /// Generate database migration file
    fn generate_migration(&self) -> Result<GeneratedFile> {
        let metadata = self.model_metadata();
        let content = self.templates.render("migration", &metadata)?;

        let table_name = TemplateHelpers::to_table_name(&self.model_name);
        let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S");
        let path = PathBuf::from(format!("migrations/{}_{}.sql", timestamp, table_name));

        Ok(GeneratedFile {
            path,
            content,
            description: format!("Database migration for {} table", table_name),
        })
    }

    /// Generate form struct file
    fn generate_forms(&self) -> Result<GeneratedFile> {
        let metadata = self.model_metadata();
        let content = self.templates.render("form", &metadata)?;

        let model_snake = TemplateHelpers::to_snake_case(&self.model_name);
        let path = PathBuf::from(format!("src/forms/{}.rs", model_snake));

        Ok(GeneratedFile {
            path,
            content,
            description: format!("Form validation for {}", self.model_name),
        })
    }
}

/// Represents a generated file
#[derive(Debug)]
pub struct GeneratedFile {
    /// Relative path from project root
    pub path: PathBuf,
    /// File content
    pub content: String,
    /// File description for user feedback
    pub description: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_new_generator() {
        let temp_dir = tempdir().unwrap();
        let generator = ScaffoldGenerator::new(
            "Post".to_string(),
            vec!["title:string".to_string(), "content:text".to_string()],
            temp_dir.path().to_path_buf(),
        );

        assert!(generator.is_ok());
        let generator = generator.unwrap();
        assert_eq!(generator.model_name, "Post");
        assert_eq!(generator.fields.len(), 2);
    }

    #[test]
    fn test_invalid_model_name() {
        let temp_dir = tempdir().unwrap();
        let result = ScaffoldGenerator::new(
            "post".to_string(), // lowercase - should fail
            vec!["title:string".to_string()],
            temp_dir.path().to_path_buf(),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_no_fields() {
        let temp_dir = tempdir().unwrap();
        let result = ScaffoldGenerator::new(
            "Post".to_string(),
            vec![], // no fields - should fail
            temp_dir.path().to_path_buf(),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_model_metadata() {
        let temp_dir = tempdir().unwrap();
        let generator = ScaffoldGenerator::new(
            "Post".to_string(),
            vec!["title:string".to_string()],
            temp_dir.path().to_path_buf(),
        )
        .unwrap();

        let metadata = generator.model_metadata();
        assert_eq!(metadata["model_name"], "Post");
        assert_eq!(metadata["table_name"], "posts");
        assert_eq!(metadata["route_path"], "/posts");
    }

    #[test]
    fn test_generate_model() {
        let temp_dir = tempdir().unwrap();
        let generator = ScaffoldGenerator::new(
            "Post".to_string(),
            vec!["title:string".to_string(), "content:text".to_string()],
            temp_dir.path().to_path_buf(),
        )
        .unwrap();

        let generated = generator.generate_model().unwrap();
        assert!(generated.path.to_string_lossy().contains("post.rs"));
        assert!(generated.content.contains("pub struct Model"));
        assert!(generated.content.contains("pub title: String"));
        assert!(generated.content.contains("pub content: String"));
    }

    #[test]
    fn test_generate_migration() {
        let temp_dir = tempdir().unwrap();
        let generator = ScaffoldGenerator::new(
            "Post".to_string(),
            vec!["title:string".to_string(), "published:boolean".to_string()],
            temp_dir.path().to_path_buf(),
        )
        .unwrap();

        let generated = generator.generate_migration().unwrap();
        assert!(generated.path.to_string_lossy().contains("posts.sql"));
        assert!(generated.content.contains("CREATE TABLE posts"));
        assert!(generated.content.contains("title VARCHAR(255) NOT NULL"));
        assert!(generated.content.contains("published BOOLEAN NOT NULL"));
    }

    #[test]
    fn test_generate_forms() {
        let temp_dir = tempdir().unwrap();
        let generator = ScaffoldGenerator::new(
            "Post".to_string(),
            vec!["title:string".to_string(), "age:integer:optional".to_string()],
            temp_dir.path().to_path_buf(),
        )
        .unwrap();

        let generated = generator.generate_forms().unwrap();
        assert!(generated.path.to_string_lossy().contains("post.rs"));
        assert!(generated.content.contains("pub struct PostForm"));
        assert!(generated.content.contains("pub title: String"));
        assert!(generated.content.contains("pub age: Option<i32>"));
    }

    #[test]
    fn test_generate_with_enum() {
        let temp_dir = tempdir().unwrap();
        let generator = ScaffoldGenerator::new(
            "Post".to_string(),
            vec![
                "title:string".to_string(),
                "status:enum:Draft,Published,Archived".to_string(),
            ],
            temp_dir.path().to_path_buf(),
        )
        .unwrap();

        let generated = generator.generate_model().unwrap();
        assert!(generated.content.contains("pub enum Status"));
        assert!(generated.content.contains("Draft"));
        assert!(generated.content.contains("Published"));
        assert!(generated.content.contains("Archived"));
    }

    #[test]
    fn test_generate_with_references() {
        let temp_dir = tempdir().unwrap();
        let generator = ScaffoldGenerator::new(
            "Comment".to_string(),
            vec![
                "content:text".to_string(),
                "author:references:User".to_string(),
                "post:references:Post".to_string(),
            ],
            temp_dir.path().to_path_buf(),
        )
        .unwrap();

        let generated = generator.generate_model().unwrap();
        assert!(generated.content.contains("pub author: UserId"));
        assert!(generated.content.contains("pub post: PostId"));

        let migration = generator.generate_migration().unwrap();
        assert!(migration.content.contains("FOREIGN KEY (author_id)"));
        assert!(migration.content.contains("REFERENCES users(id)"));
    }

    #[test]
    fn test_generate_with_unique_and_indexed() {
        let temp_dir = tempdir().unwrap();
        let generator = ScaffoldGenerator::new(
            "User".to_string(),
            vec![
                "email:string:unique".to_string(),
                "username:string:indexed".to_string(),
            ],
            temp_dir.path().to_path_buf(),
        )
        .unwrap();

        let generated = generator.generate_model().unwrap();
        assert!(generated.content.contains("#[sea_orm(unique)]"));
        assert!(generated.content.contains("#[sea_orm(indexed)]"));

        let migration = generator.generate_migration().unwrap();
        assert!(migration.content.contains("users_email_unique"));
        assert!(migration.content.contains("users_username_idx"));
    }

    #[test]
    fn test_complete_generation() {
        let temp_dir = tempdir().unwrap();
        let generator = ScaffoldGenerator::new(
            "Post".to_string(),
            vec![
                "title:string:unique".to_string(),
                "content:text".to_string(),
                "published:boolean".to_string(),
                "author:references:User".to_string(),
            ],
            temp_dir.path().to_path_buf(),
        )
        .unwrap();

        let files = generator.generate().unwrap();
        assert_eq!(files.len(), 3); // model, migration, form
        assert!(files[0].path.to_string_lossy().contains("post.rs"));
        assert!(files[1].path.to_string_lossy().contains("posts.sql"));
        assert!(files[2].path.to_string_lossy().contains("post.rs"));
    }
}
