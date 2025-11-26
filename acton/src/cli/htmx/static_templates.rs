//! Static template constants for code generation
//!
//! These are inline templates used by the scaffold and generate commands.
//! This will be migrated to external XDG-compliant templates in a future release.

/// Dockerfile template for generated projects
pub const DOCKERFILE: &str = r#"# Build stage
FROM rust:1.75-bookworm as builder

WORKDIR /app
COPY . .
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/{{project_name}} /app/
COPY --from=builder /app/templates /app/templates
COPY --from=builder /app/static /app/static
COPY --from=builder /app/migrations /app/migrations

ENV RUST_LOG=info
EXPOSE 3000

CMD ["/app/{{project_name}}"]
"#;

/// Docker Compose template
pub const DOCKER_COMPOSE: &str = r#"version: '3.8'

services:
  app:
    build: .
    ports:
      - "3000:3000"
    environment:
      - DATABASE_URL=${DATABASE_URL}
      - SESSION_SECRET=${SESSION_SECRET}
      - RUST_LOG=info
    depends_on:
      - db
    restart: unless-stopped

  db:
    image: postgres:16-alpine
    volumes:
      - postgres_data:/var/lib/postgresql/data
    environment:
      - POSTGRES_USER=${POSTGRES_USER:-app}
      - POSTGRES_PASSWORD=${POSTGRES_PASSWORD:-changeme}
      - POSTGRES_DB=${POSTGRES_DB:-{{project_name}}_prod}
    restart: unless-stopped

volumes:
  postgres_data:
"#;

/// Dockerignore template
pub const DOCKERIGNORE: &str = r"target/
.git/
.env
.env.local
*.db
*.db-shm
*.db-wal
.idea/
.vscode/
*.swp
*.swo
*~
.DS_Store
";

/// Nginx configuration template
pub const NGINX_CONF: &str = r#"upstream app {
    server app:3000;
}

server {
    listen 80;
    server_name _;

    location / {
        proxy_pass http://app;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    location /static/ {
        alias /app/static/;
        expires 1y;
        add_header Cache-Control "public, immutable";
    }
}
"#;

/// Production environment template
pub const ENV_PRODUCTION: &str = r"# Production environment configuration
# Copy to .env and customize

DATABASE_URL=postgres://user:password@localhost/{{project_name}}_prod
SESSION_SECRET=generate-a-secure-random-string-here
RUST_LOG=info
";

/// Deployment README template
pub const DEPLOYMENT_README: &str = r"# Deployment Guide for {{project_name}}

## Docker Deployment

1. Build the image:
   ```bash
   docker build -t {{project_name}} .
   ```

2. Run with docker-compose:
   ```bash
   # Create .env from template
   cp .env.production .env
   # Edit .env with your settings
   docker-compose up -d
   ```

## Manual Deployment

1. Build release:
   ```bash
   cargo build --release
   ```

2. Copy files to server:
   - `target/release/{{project_name}}`
   - `templates/`
   - `static/`
   - `migrations/`

3. Set environment variables (see `.env.production`)

4. Run:
   ```bash
   ./{{project_name}}
   ```
";

/// Background job template (MiniJinja/Jinja2 syntax)
pub const JOB_TEMPLATE: &str = r#"//! {{ job_description }}

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// {{ job_name }}Job - {{ job_description }}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {{ job_name }}Job {
{%- for field in fields %}
    /// {{ field.description }}
    pub {{ field.name }}: {{ field.rust_type }},
{%- endfor %}
}

impl {{ job_name }}Job {
    /// Create a new {{ job_name }}Job
    pub const fn new({% for field in fields %}{{ field.name }}: {{ field.rust_type }}{% if not loop.last %}, {% endif %}{% endfor %}) -> Self {
        Self {
            {%- for field in fields %}
            {{ field.name }},
            {%- endfor %}
        }
    }

    /// Execute the job
    ///
    /// # Errors
    ///
    /// Returns an error if the job fails
    pub async fn execute(&self) -> Result<{{ result_type }}> {
        // TODO: Implement job logic
        tracing::info!(
            "Executing {{ job_name }}Job"
        );

        Ok({{ result_default }})
    }
}

/// Job configuration
impl {{ job_name }}Job {
    /// Maximum number of retry attempts
    pub const MAX_RETRIES: u32 = {{ max_retries }};

    /// Timeout in seconds
    pub const TIMEOUT_SECS: u64 = {{ timeout_secs }};

    /// Job priority (higher = more priority)
    pub const PRIORITY: i32 = {{ priority }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_{{ job_name_snake }}_job() {
        // TODO: Add test implementation
    }
}
"#;
