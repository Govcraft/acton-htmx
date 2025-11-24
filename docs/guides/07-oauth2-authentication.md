# OAuth2 Authentication

acton-htmx provides built-in OAuth2 authentication support for Google, GitHub, and generic OpenID Connect (OIDC) providers. This guide covers setup, configuration, and usage.

## Table of Contents

- [Overview](#overview)
- [Quick Start](#quick-start)
- [Provider Setup](#provider-setup)
  - [Google OAuth2](#google-oauth2)
  - [GitHub OAuth2](#github-oauth2)
  - [Generic OIDC](#generic-oidc)
- [Configuration](#configuration)
- [Database Setup](#database-setup)
- [Usage](#usage)
- [Security](#security)
- [Troubleshooting](#troubleshooting)

## Overview

The OAuth2 system includes:

- **CSRF Protection**: State tokens prevent cross-site request forgery attacks
- **PKCE Support**: Proof Key for Code Exchange for enhanced security
- **Account Linking**: Link multiple OAuth providers to a single user account
- **Type Safety**: Strongly typed providers and configurations
- **acton-reactive Integration**: State management via agents

## Quick Start

1. **Configure OAuth2 providers** in `config.toml`:

```toml
[oauth2.google]
client_id = "your-google-client-id.apps.googleusercontent.com"
client_secret = "your-google-client-secret"
redirect_uri = "http://localhost:3000/auth/google/callback"
scopes = ["openid", "email", "profile"]

[oauth2.github]
client_id = "your-github-client-id"
client_secret = "your-github-client-secret"
redirect_uri = "http://localhost:3000/auth/github/callback"
scopes = ["read:user", "user:email"]
```

2. **Run database migrations**:

```bash
acton-htmx db migrate
```

3. **Add OAuth2 routes** to your application:

```rust
use acton_htmx::oauth2::handlers::{handle_oauth_callback, initiate_oauth, unlink_oauth_account};

let app = Router::new()
    // OAuth2 routes
    .route("/auth/:provider", get(initiate_oauth))
    .route("/auth/:provider/callback", get(handle_oauth_callback))
    .route("/auth/:provider/unlink", delete(unlink_oauth_account))
    // Your other routes
    .with_state(state);
```

4. **Include login buttons** in your template:

```jinja2
{% include "oauth2/login_buttons.html" %}
```

## Provider Setup

### Google OAuth2

1. **Create a Google Cloud Project**:
   - Go to [Google Cloud Console](https://console.cloud.google.com/)
   - Create a new project or select an existing one

2. **Enable Google+ API**:
   - Navigate to "APIs & Services" > "Library"
   - Search for "Google+ API" and enable it

3. **Create OAuth2 Credentials**:
   - Go to "APIs & Services" > "Credentials"
   - Click "Create Credentials" > "OAuth client ID"
   - Choose "Web application"
   - Add authorized redirect URIs:
     - Development: `http://localhost:3000/auth/google/callback`
     - Production: `https://yourdomain.com/auth/google/callback`

4. **Configure in acton-htmx**:

```toml
[oauth2.google]
client_id = "YOUR_CLIENT_ID.apps.googleusercontent.com"
client_secret = "YOUR_CLIENT_SECRET"
redirect_uri = "http://localhost:3000/auth/google/callback"
scopes = ["openid", "email", "profile"]
```

**Scopes**:
- `openid`: Required for OpenID Connect
- `email`: Access to user's email address
- `profile`: Access to user's basic profile information

### GitHub OAuth2

1. **Create a GitHub OAuth App**:
   - Go to [GitHub Developer Settings](https://github.com/settings/developers)
   - Click "New OAuth App"
   - Fill in the form:
     - Application name: Your app name
     - Homepage URL: `http://localhost:3000` (development)
     - Authorization callback URL: `http://localhost:3000/auth/github/callback`

2. **Get Client ID and Secret**:
   - After creating the app, note the Client ID
   - Generate a new client secret

3. **Configure in acton-htmx**:

```toml
[oauth2.github]
client_id = "YOUR_GITHUB_CLIENT_ID"
client_secret = "YOUR_GITHUB_CLIENT_SECRET"
redirect_uri = "http://localhost:3000/auth/github/callback"
scopes = ["read:user", "user:email"]
```

**Scopes**:
- `read:user`: Read access to user profile data
- `user:email`: Access to user's email addresses

### Generic OIDC

For other OpenID Connect providers (Okta, Auth0, Keycloak, etc.):

```toml
[oauth2.oidc]
client_id = "YOUR_CLIENT_ID"
client_secret = "YOUR_CLIENT_SECRET"
redirect_uri = "http://localhost:3000/auth/oidc/callback"
scopes = ["openid", "email", "profile"]
auth_url = "https://your-provider.com/oauth2/authorize"
token_url = "https://your-provider.com/oauth2/token"
userinfo_url = "https://your-provider.com/oauth2/userinfo"
```

## Configuration

### Environment Variables

For production, use environment variables instead of config files:

```bash
# Google
export OAUTH2_GOOGLE_CLIENT_ID="your-client-id"
export OAUTH2_GOOGLE_CLIENT_SECRET="your-client-secret"
export OAUTH2_GOOGLE_REDIRECT_URI="https://yourdomain.com/auth/google/callback"

# GitHub
export OAUTH2_GITHUB_CLIENT_ID="your-client-id"
export OAUTH2_GITHUB_CLIENT_SECRET="your-client-secret"
export OAUTH2_GITHUB_REDIRECT_URI="https://yourdomain.com/auth/github/callback"
```

### Configuration File

`~/.config/acton-htmx/config.toml`:

```toml
[oauth2.google]
client_id = "${OAUTH2_GOOGLE_CLIENT_ID}"
client_secret = "${OAUTH2_GOOGLE_CLIENT_SECRET}"
redirect_uri = "${OAUTH2_GOOGLE_REDIRECT_URI}"
scopes = ["openid", "email", "profile"]

[oauth2.github]
client_id = "${OAUTH2_GITHUB_CLIENT_ID}"
client_secret = "${OAUTH2_GITHUB_CLIENT_SECRET}"
redirect_uri = "${OAUTH2_GITHUB_REDIRECT_URI}"
scopes = ["read:user", "user:email"]
```

## Database Setup

The OAuth2 system requires the `oauth_accounts` table:

```sql
CREATE TABLE oauth_accounts (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    provider TEXT NOT NULL CHECK (provider IN ('google', 'github', 'oidc')),
    provider_user_id TEXT NOT NULL,
    email TEXT NOT NULL,
    name TEXT,
    avatar_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_oauth_accounts_user FOREIGN KEY (user_id)
        REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT unique_oauth_account UNIQUE (provider, provider_user_id)
);

CREATE INDEX idx_oauth_accounts_user_id ON oauth_accounts(user_id);
CREATE INDEX idx_oauth_accounts_provider ON oauth_accounts(provider);
```

Run migrations:

```bash
acton-htmx db migrate
```

## Usage

### Login Flow

1. **User clicks OAuth login button**:

```html
<a href="/auth/google">Sign in with Google</a>
```

2. **User is redirected to provider** for authorization

3. **Provider redirects back** to callback URL with authorization code

4. **Backend exchanges code** for access token and fetches user info

5. **User is created/linked** and authenticated

### Account Linking

Allow authenticated users to link additional OAuth accounts:

```rust
use acton_htmx::{
    auth::extractors::Authenticated,
    oauth2::models::OAuthAccount,
};

async fn account_settings(
    State(state): State<ActonHtmxState>,
    Authenticated(user): Authenticated<User>,
) -> Result<impl IntoResponse> {
    // Get linked OAuth accounts
    let oauth_accounts = OAuthAccount::find_by_user_id(
        state.database_pool(),
        user.id
    ).await?;

    // Render template with accounts
    Ok(AccountSettingsTemplate { oauth_accounts })
}
```

### Account Unlinking

```rust
// Already implemented in handlers::unlink_oauth_account
// DELETE /auth/:provider/unlink
```

### Templates

**Login Buttons**:

```jinja2
{% include "oauth2/login_buttons.html" %}
```

**Account Settings**:

```jinja2
{% extends "base.html" %}

{% block content %}
<h1>Connected Accounts</h1>

{% for account in oauth_accounts %}
<div class="oauth-account">
    <span>{{ account.provider }} - {{ account.email }}</span>
    <button hx-delete="/auth/{{ account.provider }}/unlink"
            hx-confirm="Unlink {{ account.provider }}?">
        Unlink
    </button>
</div>
{% endfor %}
{% endblock %}
```

## Security

### CSRF Protection

State tokens are automatically generated and validated:

- **64-character random tokens** (32 bytes as hex)
- **10-minute expiration** for security
- **One-time use** - tokens removed after validation
- **Automatic cleanup** of expired tokens

### PKCE (Proof Key for Code Exchange)

All providers use PKCE to prevent authorization code interception:

- **Code verifier** generated and stored in session
- **Code challenge** sent to authorization endpoint
- **Verification** performed during token exchange

### Secure Cookie Storage

Session data is stored in HTTP-only cookies:

```rust
// Automatic via SessionMiddleware
session.set("oauth2_state", &state_token)?;
session.set("oauth2_pkce_verifier", &pkce_verifier)?;
```

### Best Practices

1. **Use HTTPS in production** - Required for secure OAuth2
2. **Keep secrets secret** - Never commit client secrets to git
3. **Use environment variables** - For production credentials
4. **Validate redirect URIs** - Match exactly in provider settings
5. **Monitor failed attempts** - Log and alert on OAuth errors

## Troubleshooting

### "redirect_uri_mismatch" Error

**Problem**: OAuth provider rejects the redirect URI.

**Solution**: Ensure redirect URI matches exactly:
- Check protocol (http vs https)
- Check domain and port
- Check path (must be `/auth/:provider/callback`)
- Update in provider settings

### "invalid_client" Error

**Problem**: Client ID or secret is incorrect.

**Solution**:
- Verify client ID and secret from provider
- Check for trailing spaces or newlines
- Ensure environment variables are loaded

### "State Mismatch" Error

**Problem**: CSRF state token validation failed.

**Solution**:
- Ensure cookies are enabled
- Check session middleware is configured
- Verify state token expiration (10 minutes)

### User Already Exists

**Problem**: OAuth email matches existing user with different provider.

**Solution**: Implement account merging logic or require manual linking.

### Provider Returns Error

**Problem**: OAuth provider returns error parameter.

**Solution**: Check provider documentation for error codes:
- `access_denied`: User cancelled authorization
- `invalid_scope`: Requested scope not available
- `server_error`: Provider experiencing issues

## Example: Complete Integration

```rust
use acton_htmx::{
    oauth2::{
        handlers::{handle_oauth_callback, initiate_oauth, unlink_oauth_account},
        models::OAuthAccount,
    },
    auth::extractors::Authenticated,
    prelude::*,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut runtime = ActonApp::launch();
    let state = ActonHtmxState::new(&mut runtime).await?;

    let app = Router::new()
        // OAuth2 routes
        .route("/auth/:provider", get(initiate_oauth))
        .route("/auth/:provider/callback", get(handle_oauth_callback))
        .route("/auth/:provider/unlink", delete(unlink_oauth_account))
        // Account settings
        .route("/settings/accounts", get(account_settings))
        .layer(SessionLayer::new(&state))
        .with_state(state);

    // Start server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await?;
    runtime.shutdown_all().await?;
    Ok(())
}

async fn account_settings(
    State(state): State<ActonHtmxState>,
    Authenticated(user): Authenticated<User>,
) -> Result<impl IntoResponse> {
    let oauth_accounts = OAuthAccount::find_by_user_id(
        state.database_pool(),
        user.id
    ).await?;

    Ok(AccountSettingsTemplate {
        oauth_accounts,
        google_available: state.config().oauth2.google.is_some(),
        github_available: state.config().oauth2.github.is_some(),
        has_google: oauth_accounts.iter().any(|a| a.provider == OAuthProvider::Google),
        has_github: oauth_accounts.iter().any(|a| a.provider == OAuthProvider::GitHub),
    })
}
```

## Next Steps

- [Session Management](03-authentication.md#sessions)
- [CSRF Protection](03-authentication.md#csrf)
- [User Management](03-authentication.md#user-model)
- [Deployment Guide](05-deployment.md)
