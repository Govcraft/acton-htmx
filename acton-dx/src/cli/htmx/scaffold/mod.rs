//! CRUD scaffold generator implementation
//!
//! This module implements the intelligent code generation system for Acton HTMX.
//! It transforms field specifications into complete CRUD resources with models,
//! handlers, templates, migrations, and tests.

pub mod field_type;
pub mod generator;
pub mod helpers;
pub mod templates;

pub use field_type::{FieldDefinition, FieldType};
pub use generator::ScaffoldGenerator;
pub use helpers::TemplateHelpers;
