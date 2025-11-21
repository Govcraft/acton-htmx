//! Template file contents

/// Cargo.toml template for new projects
pub const CARGO_TOML: &str = r#"[package]
name = "{{project_name}}"
version = "0.1.0"
edition = "2021"
rust-version = "1.75"

[dependencies]
acton-htmx = { version = "0.1", features = ["full"] }
acton-reactive = "5"
axum = "0.8"
tokio = { version = "1", features = ["full"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["fs", "trace", "cors"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
validator = { version = "0.20", features = ["derive"] }
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "macros", "migrate"] }
figment = { version = "0.10", features = ["toml", "env"] }
anyhow = "1"
thiserror = "2"

[dev-dependencies]
http-body-util = "0.1"
tower = { version = "0.5", features = ["util"] }

[profile.dev]
opt-level = 0

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
"#;

/// README.md template for new projects
pub const README_MD: &str = r"# {{project_name}}

A web application built with [acton-htmx](https://github.com/govcraft/acton-htmx), an opinionated Rust framework for server-rendered HTMX applications.

## Quick Start

### Prerequisites

- Rust 1.75 or later
- PostgreSQL
- acton-htmx CLI: `cargo install acton-htmx-cli`

### Setup

1. Create database:
   ```bash
   createdb {{project_name_snake}}_dev
   ```

2. Run migrations:
   ```bash
   acton-htmx db migrate
   ```

3. Start development server:
   ```bash
   acton-htmx dev
   ```

4. Open http://localhost:3000

## Project Structure

```
{{project_name}}/
├── src/
│   ├── main.rs              # Application entry point
│   ├── handlers/            # HTTP request handlers
│   │   ├── mod.rs
│   │   ├── home.rs
│   │   └── auth.rs
│   └── models/              # Domain models
│       ├── mod.rs
│       └── user.rs
├── templates/               # Askama templates
│   ├── layouts/
│   ├── auth/
│   └── partials/
├── static/                  # Static assets
│   ├── css/
│   └── js/
├── config/                  # Configuration files
│   ├── development.toml
│   └── production.toml
└── migrations/              # Database migrations
```

## Development

### Running Tests

```bash
cargo test
```

### Database Commands

```bash
# Run migrations
acton-htmx db migrate

# Reset database
acton-htmx db reset

# Create new migration
acton-htmx db create <name>
```

### Building for Production

```bash
cargo build --release
```

## Features

- ✅ HTMX-first architecture with server-side rendering
- ✅ Session-based authentication with Argon2id
- ✅ CSRF protection enabled by default
- ✅ Security headers configured
- ✅ PostgreSQL with SQLx
- ✅ Askama templates with compile-time checking
- ✅ Form validation with validator crate
- ✅ Flash messages via acton-reactive agents

## License

MIT
";

/// .gitignore template for new projects
pub const GITIGNORE: &str = r"# Rust
/target
/Cargo.lock
**/*.rs.bk

# Environment
.env
.env.local

# Database
*.db
*.db-shm
*.db-wal

# IDE
.idea/
.vscode/
*.swp
*.swo
*~

# OS
.DS_Store
Thumbs.db

# Logs
*.log
";

/// Development configuration template
pub const CONFIG_DEV: &str = r#"# Development configuration

[server]
host = "127.0.0.1"
port = 3000

[database]
url = "postgres://localhost/{{project_name_snake}}_dev"
max_connections = 5

[session]
secret = "development-secret-change-in-production"
cookie_name = "{{project_name_snake}}_session"
cookie_secure = false
max_age_seconds = 86400

[csrf]
enabled = true

[security_headers]
preset = "development"

[logging]
level = "debug"
"#;

/// Production configuration template
pub const CONFIG_PROD: &str = r#"# Production configuration

[server]
host = "0.0.0.0"
port = 3000

[database]
url = "${DATABASE_URL}"
max_connections = 20

[session]
secret = "${SESSION_SECRET}"
cookie_name = "{{project_name_snake}}_session"
cookie_secure = true
max_age_seconds = 86400

[csrf]
enabled = true

[security_headers]
preset = "strict"

[logging]
level = "info"
"#;

/// Main.rs template for new projects
pub const MAIN_RS: &str = r#"//! {{project_name}} - Built with acton-htmx

#![forbid(unsafe_code)]
#![deny(clippy::all, clippy::pedantic, clippy::nursery)]
#![warn(clippy::cargo)]

use acton_htmx::{
    prelude::*,
    config::Config,
    state::AppState,
    middleware::{CsrfLayer, SecurityHeadersConfig, SecurityHeadersLayer, SessionLayer},
    agents::{CsrfManagerAgent, SessionManagerAgent},
};
use acton_reactive::prelude::*;
use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use tower_http::{
    services::ServeDir,
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod handlers;
mod models;

use handlers::{auth, home};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "{{project_name_snake}}=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::load()?;

    // Initialize database
    let db = PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .connect(&config.database.url)
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&db).await?;

    // Initialize acton-reactive runtime
    let acton_app = ActonApp::launch();

    // Spawn session manager agent
    let session_manager = acton_app
        .spawn_agent::<SessionManagerAgent>("session-manager")
        .await?;

    // Spawn CSRF manager agent
    let csrf_manager = acton_app
        .spawn_agent::<CsrfManagerAgent>("csrf-manager")
        .await?;

    // Create application state
    let state = AppState::new(
        db,
        acton_app.clone(),
        session_manager,
        csrf_manager,
        config.clone(),
    );

    // Build router
    let app = Router::new()
        // Public routes
        .route("/", get(home::index))
        .route("/login", get(auth::login_form).post(auth::login))
        .route("/register", get(auth::register_form).post(auth::register))
        .route("/logout", post(auth::logout))
        // Static files
        .nest_service("/static", ServeDir::new("static"))
        // Middleware
        .layer(SecurityHeadersLayer::new(
            SecurityHeadersConfig::from_preset(&config.security_headers.preset),
        ))
        .layer(CsrfLayer::new())
        .layer(SessionLayer::new(config.session.clone()))
        .layer(TraceLayer::new_for_http())
        // State
        .with_state(state);

    // Start server
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("Starting server on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
"#;

/// Handlers module template
pub const HANDLERS_MOD: &str = r"//! HTTP request handlers

pub mod home;
pub mod auth;
";

/// Home handler template
pub const HANDLERS_HOME: &str = r#"//! Home page handlers

use acton_htmx::prelude::*;
use askama::Template;
use axum::response::Html;

#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate {
    title: String,
}

/// Home page
pub async fn index() -> Html<String> {
    let template = HomeTemplate {
        title: "Welcome".to_string(),
    };

    Html(template.render().unwrap())
}
"#;

/// Authentication handler template
pub const HANDLERS_AUTH: &str = r#"//! Authentication handlers

use acton_htmx::{
    prelude::*,
    extractors::{Session, ValidatedForm},
};
use askama::Template;
use axum::{
    response::{Html, IntoResponse, Redirect},
};
use serde::Deserialize;
use validator::Validate;

use crate::models::User;

#[derive(Template)]
#[template(path = "auth/login.html")]
struct LoginTemplate {
    error: Option<String>,
}

#[derive(Template)]
#[template(path = "auth/register.html")]
struct RegisterTemplate {
    error: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginForm {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterForm {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8, max = 100))]
    pub password: String,
    #[validate(must_match(other = "password"))]
    pub password_confirmation: String,
}

/// Show login form
pub async fn login_form() -> Html<String> {
    let template = LoginTemplate { error: None };
    Html(template.render().unwrap())
}

/// Process login
pub async fn login(
    mut session: Session,
    ValidatedForm(form): ValidatedForm<LoginForm>,
) -> impl IntoResponse {
    // TODO: Implement actual authentication
    // For now, just set user_id in session
    session.set("user_id", "1").await.unwrap();

    Redirect::to("/")
}

/// Show registration form
pub async fn register_form() -> Html<String> {
    let template = RegisterTemplate { error: None };
    Html(template.render().unwrap())
}

/// Process registration
pub async fn register(
    mut session: Session,
    ValidatedForm(form): ValidatedForm<RegisterForm>,
) -> impl IntoResponse {
    // TODO: Implement actual registration
    // For now, just redirect to login
    Redirect::to("/login")
}

/// Logout
pub async fn logout(mut session: Session) -> impl IntoResponse {
    session.clear().await.unwrap();
    Redirect::to("/")
}
"#;

/// Models module template
pub const MODELS_MOD: &str = r"//! Domain models

pub mod user;
pub use user::User;
";

/// User model template
pub const MODELS_USER: &str = r"//! User model

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i32,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl User {
    /// Verify password
    pub fn verify_password(&self, password: &str) -> bool {
        // TODO: Implement actual password verification with Argon2
        false
    }
}
";

/// Base HTML template
pub const TEMPLATE_BASE: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{% block title %}{{project_name}}{% endblock %}</title>

    <!-- HTMX -->
    <script src="https://unpkg.com/htmx.org@2.0.4"></script>

    <!-- Styles -->
    <link rel="stylesheet" href="/static/css/app.css">

    {% block head %}{% endblock %}
</head>
<body>
    {% block body %}{% endblock %}
</body>
</html>
"#;

/// App layout template
pub const TEMPLATE_APP: &str = r#"{% extends "layouts/base.html" %}

{% block body %}
<div class="container">
    {% include "partials/nav.html" %}

    <div id="flash-messages">
        {% include "partials/flash.html" %}
    </div>

    <main>
        {% block content %}{% endblock %}
    </main>
</div>
{% endblock %}
"#;

/// Login page template
pub const TEMPLATE_LOGIN: &str = r#"{% extends "layouts/app.html" %}

{% block title %}Login - {{project_name}}{% endblock %}

{% block content %}
<div class="auth-form">
    <h1>Login</h1>

    {% if let Some(error) = error %}
    <div class="error">{{ error }}</div>
    {% endif %}

    <form hx-post="/login" hx-target="body">
        <div class="field">
            <label for="email">Email</label>
            <input type="email" id="email" name="email" required>
        </div>

        <div class="field">
            <label for="password">Password</label>
            <input type="password" id="password" name="password" required>
        </div>

        <button type="submit">Login</button>
    </form>

    <p>Don't have an account? <a href="/register">Register</a></p>
</div>
{% endblock %}
"#;

/// Registration page template
pub const TEMPLATE_REGISTER: &str = r#"{% extends "layouts/app.html" %}

{% block title %}Register - {{project_name}}{% endblock %}

{% block content %}
<div class="auth-form">
    <h1>Register</h1>

    {% if let Some(error) = error %}
    <div class="error">{{ error }}</div>
    {% endif %}

    <form hx-post="/register" hx-target="body">
        <div class="field">
            <label for="email">Email</label>
            <input type="email" id="email" name="email" required>
        </div>

        <div class="field">
            <label for="password">Password</label>
            <input type="password" id="password" name="password" required>
        </div>

        <div class="field">
            <label for="password_confirmation">Confirm Password</label>
            <input type="password" id="password_confirmation" name="password_confirmation" required>
        </div>

        <button type="submit">Register</button>
    </form>

    <p>Already have an account? <a href="/login">Login</a></p>
</div>
{% endblock %}
"#;

/// Flash messages partial template
pub const TEMPLATE_FLASH: &str = r"<!-- Flash messages will be rendered here -->
";

/// Navigation partial template
pub const TEMPLATE_NAV: &str = r#"<nav>
    <a href="/">Home</a>
    <a href="/login">Login</a>
    <a href="/register">Register</a>
</nav>
"#;

/// Home page template
pub const TEMPLATE_HOME: &str = r#"{% extends "layouts/app.html" %}

{% block title %}{{ title }} - {{project_name}}{% endblock %}

{% block content %}
<div class="home">
    <h1>{{ title }}</h1>
    <p>Welcome to your acton-htmx application!</p>

    <h2>Getting Started</h2>
    <ul>
        <li>Edit templates in <code>templates/</code></li>
        <li>Add handlers in <code>src/handlers/</code></li>
        <li>Define models in <code>src/models/</code></li>
        <li>Update routes in <code>src/main.rs</code></li>
    </ul>

    <h2>Documentation</h2>
    <ul>
        <li><a href="https://htmx.org" target="_blank">HTMX Documentation</a></li>
        <li><a href="https://github.com/govcraft/acton-htmx" target="_blank">acton-htmx Repository</a></li>
    </ul>
</div>
{% endblock %}
"#;

/// CSS stylesheet template
pub const STATIC_CSS: &str = r#"/* Basic styling for {{project_name}} */

* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: system-ui, -apple-system, sans-serif;
    line-height: 1.6;
    color: #333;
    background: #f5f5f5;
}

.container {
    max-width: 1200px;
    margin: 0 auto;
    padding: 20px;
}

nav {
    background: white;
    padding: 1rem;
    margin-bottom: 2rem;
    border-radius: 4px;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}

nav a {
    margin-right: 1rem;
    text-decoration: none;
    color: #0066cc;
}

nav a:hover {
    text-decoration: underline;
}

main {
    background: white;
    padding: 2rem;
    border-radius: 4px;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}

.auth-form {
    max-width: 400px;
    margin: 0 auto;
}

.field {
    margin-bottom: 1rem;
}

label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 600;
}

input[type="text"],
input[type="email"],
input[type="password"] {
    width: 100%;
    padding: 0.5rem;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-size: 1rem;
}

input.error {
    border-color: #dc3545;
}

.error {
    color: #dc3545;
    font-size: 0.875rem;
    margin-top: 0.25rem;
}

button[type="submit"] {
    width: 100%;
    padding: 0.75rem;
    background: #0066cc;
    color: white;
    border: none;
    border-radius: 4px;
    font-size: 1rem;
    cursor: pointer;
}

button[type="submit"]:hover {
    background: #0052a3;
}

code {
    background: #f5f5f5;
    padding: 0.2rem 0.4rem;
    border-radius: 3px;
    font-family: 'Courier New', monospace;
}

a {
    color: #0066cc;
}
"#;

/// Initial users table migration
pub const MIGRATION_USERS: &str = r"-- Create users table

CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE INDEX idx_users_email ON users(email);
";
