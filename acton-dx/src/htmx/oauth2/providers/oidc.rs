//! Generic OpenID Connect provider implementation
//!
//! This module implements a generic OIDC provider that can work with any
//! OpenID Connect compliant identity provider (Okta, Auth0, Keycloak, etc.).

use openidconnect::{
    core::CoreProviderMetadata,
    IssuerUrl,
};
use serde::{Deserialize, Serialize};

use super::base::BaseOAuthProvider;
use crate::htmx::oauth2::http::async_http_client;
use crate::htmx::oauth2::types::{OAuthError, OAuthToken, OAuthUserInfo, ProviderConfig};

/// Generic OpenID Connect provider
pub struct OidcProvider {
    base: BaseOAuthProvider,
}

impl OidcProvider {
    /// Create a new generic OIDC provider
    ///
    /// # Errors
    ///
    /// Returns error if the configuration is invalid or if discovery fails
    ///
    /// # Panics
    ///
    /// This function should not panic as all unwrap() calls are guarded by is_some() checks
    pub async fn new(config: &ProviderConfig) -> Result<Self, OAuthError> {
        // For generic OIDC, we require either:
        // 1. Manual configuration (all three URLs: auth_url, token_url, userinfo_url)
        // 2. Discovery via issuer URL (only auth_url provided)

        let base = if config.auth_url.is_some()
            && config.token_url.is_some()
            && config.userinfo_url.is_some()
        {
            // Manual configuration - all URLs provided
            // SAFETY: These unwraps are safe because we just checked is_some() above
            let auth_url = config.auth_url.as_ref().unwrap();
            let token_url = config.token_url.as_ref().unwrap();
            let userinfo_url = config.userinfo_url.as_ref().unwrap();

            BaseOAuthProvider::new(auth_url, token_url, config, userinfo_url.clone())?
        } else if let Some(issuer_url) = &config.auth_url {
            // Discovery - only issuer URL provided
            Self::discover_and_create(config, issuer_url).await?
        } else {
            return Err(OAuthError::Generic(
                "Either provide all URLs (auth_url, token_url, userinfo_url) for manual config, or just auth_url for discovery".to_string(),
            ));
        };

        Ok(Self { base })
    }

    /// Discover OIDC configuration from issuer and create base provider
    async fn discover_and_create(
        config: &ProviderConfig,
        issuer: &str,
    ) -> Result<BaseOAuthProvider, OAuthError> {
        let issuer_url = IssuerUrl::new(issuer.to_string())
            .map_err(|e| OAuthError::Generic(format!("Invalid issuer URL: {e}")))?;

        let provider_metadata = CoreProviderMetadata::discover_async(issuer_url, &async_http_client)
            .await
            .map_err(|e| OAuthError::Generic(format!("Failed to discover provider: {e}")))?;

        let userinfo_url = provider_metadata
            .userinfo_endpoint()
            .ok_or_else(|| OAuthError::Generic("Provider has no userinfo endpoint".to_string()))?
            .to_string();

        let auth_url = provider_metadata
            .authorization_endpoint()
            .to_string();

        let token_url = provider_metadata
            .token_endpoint()
            .ok_or_else(|| OAuthError::Generic("Provider has no token endpoint".to_string()))?
            .to_string();

        BaseOAuthProvider::new(&auth_url, &token_url, config, userinfo_url)
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

        let oidc_user: OidcUserInfo = serde_json::from_value(json)
            .map_err(|e| OAuthError::UserInfoFailed(format!("Failed to parse OIDC user info: {e}")))?;

        Ok(OAuthUserInfo {
            provider_user_id: oidc_user.sub,
            email: oidc_user.email,
            name: oidc_user.name,
            avatar_url: oidc_user.picture,
            email_verified: oidc_user.email_verified.unwrap_or(false),
        })
    }
}

/// OIDC user info response
#[derive(Debug, Deserialize, Serialize)]
struct OidcUserInfo {
    sub: String,
    email: String,
    email_verified: Option<bool>,
    name: Option<String>,
    picture: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oidc_manual_config() {
        let config = ProviderConfig {
            client_id: "test-client-id".to_string(),
            client_secret: "test-client-secret".to_string(),
            redirect_uri: "http://localhost:3000/auth/oidc/callback".to_string(),
            scopes: vec!["openid".to_string(), "email".to_string()],
            auth_url: Some("https://example.com/oauth/authorize".to_string()),
            token_url: Some("https://example.com/oauth/token".to_string()),
            userinfo_url: Some("https://example.com/oauth/userinfo".to_string()),
        };

        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let provider = OidcProvider::new(&config).await.unwrap();
            let (auth_url, csrf_state, pkce_verifier) = provider.authorization_url();

            assert!(auth_url.starts_with("https://example.com/oauth/authorize"));
            assert!(auth_url.contains("client_id=test-client-id"));
            assert!(!csrf_state.is_empty());
            assert!(!pkce_verifier.is_empty());
        });
    }
}
