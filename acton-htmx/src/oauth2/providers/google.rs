//! Google OAuth2 provider implementation
//!
//! This module implements OAuth2 authentication with Google using the OpenID Connect
//! discovery protocol.

use serde::{Deserialize, Serialize};

use super::base::BaseOAuthProvider;
use crate::oauth2::types::{OAuthError, OAuthToken, OAuthUserInfo, ProviderConfig};

/// Google OAuth2 provider
pub struct GoogleProvider {
    base: BaseOAuthProvider,
}

impl GoogleProvider {
    /// Create a new Google OAuth2 provider
    ///
    /// # Errors
    ///
    /// Returns error if the provider metadata cannot be fetched or if the
    /// configuration is invalid
    pub fn new(config: &ProviderConfig) -> Result<Self, OAuthError> {
        Ok(Self {
            base: BaseOAuthProvider::new(
                "https://accounts.google.com/o/oauth2/v2/auth",
                "https://oauth2.googleapis.com/token",
                config,
                "https://www.googleapis.com/oauth2/v2/userinfo".to_string(),
            )?,
        })
    }

    /// Generate authorization URL and CSRF state
    ///
    /// Returns tuple of (authorization_url, csrf_state, pkce_verifier)
    #[must_use]
    pub fn authorization_url(&self) -> (String, String, String) {
        self.base.authorization_url(&["openid", "email", "profile"])
    }

    /// Exchange authorization code for access token
    ///
    /// # Errors
    ///
    /// Returns error if the token exchange fails
    pub async fn exchange_code(
        &self,
        code: &str,
        pkce_verifier: &str,
    ) -> Result<OAuthToken, OAuthError> {
        self.base.exchange_code(code, pkce_verifier).await
    }

    /// Fetch user information using access token
    ///
    /// # Errors
    ///
    /// Returns error if the user info request fails
    pub async fn fetch_user_info(&self, access_token: &str) -> Result<OAuthUserInfo, OAuthError> {
        let json = self.base.fetch_user_info_json(access_token).await?;

        let google_user: GoogleUserInfo = serde_json::from_value(json)
            .map_err(|e| OAuthError::UserInfoFailed(format!("Failed to parse Google user info: {e}")))?;

        Ok(OAuthUserInfo {
            provider_user_id: google_user.id,
            email: google_user.email,
            name: google_user.name,
            avatar_url: google_user.picture,
            email_verified: google_user.verified_email.unwrap_or(false),
        })
    }
}

/// Google user info response
#[derive(Debug, Deserialize, Serialize)]
struct GoogleUserInfo {
    id: String,
    email: String,
    verified_email: Option<bool>,
    name: Option<String>,
    picture: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authorization_url_generation() {
        // This test requires a valid Google OAuth2 configuration
        // In a real scenario, you would use test credentials
        let config = ProviderConfig {
            client_id: "test-client-id".to_string(),
            client_secret: "test-client-secret".to_string(),
            redirect_uri: "http://localhost:3000/auth/google/callback".to_string(),
            scopes: vec!["openid".to_string(), "email".to_string()],
            auth_url: None,
            token_url: None,
            userinfo_url: None,
        };

        let provider = GoogleProvider::new(&config).unwrap();
        let (auth_url, csrf_state, pkce_verifier) = provider.authorization_url();

        assert!(auth_url.starts_with("https://accounts.google.com"));
        assert!(auth_url.contains("client_id=test-client-id"));
        assert!(auth_url.contains("redirect_uri"));
        assert!(auth_url.contains("scope=openid"));
        assert!(!csrf_state.is_empty());
        assert!(!pkce_verifier.is_empty());
    }
}
