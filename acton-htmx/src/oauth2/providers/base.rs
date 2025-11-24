//! Base OAuth2 provider implementation with shared logic
//!
//! This module provides `BaseOAuthProvider` which contains all the common
//! OAuth2 logic shared between Google, GitHub, and OIDC providers.

use oauth2::{
    basic::BasicClient, AuthUrl, AuthorizationCode, ClientId,
    ClientSecret, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope,
    TokenResponse, TokenUrl,
};

use crate::oauth2::http::async_http_client;
use crate::oauth2::types::{ConfiguredClient, OAuthError, OAuthToken, ProviderConfig};

/// Base OAuth2 provider containing shared logic for all providers
pub struct BaseOAuthProvider {
    /// Configured OAuth2 client
    client: ConfiguredClient,
    /// Reusable HTTP client for userinfo requests
    http_client: reqwest::Client,
    /// Userinfo endpoint URL
    userinfo_url: String,
}

impl BaseOAuthProvider {
    /// Create a new base OAuth2 provider
    ///
    /// # Arguments
    ///
    /// * `auth_url` - Authorization endpoint URL
    /// * `token_url` - Token endpoint URL
    /// * `config` - Provider configuration containing client credentials
    /// * `userinfo_url` - Userinfo endpoint URL
    ///
    /// # Errors
    ///
    /// Returns error if any URL is invalid
    pub fn new(
        auth_url: &str,
        token_url: &str,
        config: &ProviderConfig,
        userinfo_url: String,
    ) -> Result<Self, OAuthError> {
        // oauth2 5.0 API: BasicClient::new() only takes ClientId
        let client = BasicClient::new(ClientId::new(config.client_id.clone()))
            .set_client_secret(ClientSecret::new(config.client_secret.clone()))
            .set_auth_uri(
                AuthUrl::new(auth_url.to_string())
                    .map_err(|e| OAuthError::Generic(format!("Invalid auth URL: {e}")))?,
            )
            .set_token_uri(
                TokenUrl::new(token_url.to_string())
                    .map_err(|e| OAuthError::Generic(format!("Invalid token URL: {e}")))?,
            )
            .set_redirect_uri(
                RedirectUrl::new(config.redirect_uri.clone())
                    .map_err(|e| OAuthError::Generic(format!("Invalid redirect URI: {e}")))?,
            );

        Ok(Self {
            client,
            http_client: reqwest::Client::new(),
            userinfo_url,
        })
    }

    /// Generate authorization URL with PKCE
    ///
    /// # Arguments
    ///
    /// * `scopes` - OAuth scopes to request
    ///
    /// # Returns
    ///
    /// Tuple of (authorization_url, csrf_state, pkce_verifier)
    #[must_use]
    pub fn authorization_url(&self, scopes: &[&str]) -> (String, String, String) {
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let mut auth_url_builder = self.client.authorize_url(CsrfToken::new_random);

        // Add all requested scopes
        for scope in scopes {
            auth_url_builder = auth_url_builder.add_scope(Scope::new((*scope).to_string()));
        }

        let (auth_url, csrf_state) = auth_url_builder
            .set_pkce_challenge(pkce_challenge)
            .url();

        (
            auth_url.to_string(),
            csrf_state.secret().clone(),
            pkce_verifier.secret().clone(),
        )
    }

    /// Exchange authorization code for access token
    ///
    /// # Arguments
    ///
    /// * `code` - Authorization code from callback
    /// * `pkce_verifier` - PKCE verifier from authorization
    ///
    /// # Errors
    ///
    /// Returns error if token exchange fails
    pub async fn exchange_code(
        &self,
        code: &str,
        pkce_verifier: &str,
    ) -> Result<OAuthToken, OAuthError> {
        let token_response = self
            .client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .set_pkce_verifier(PkceCodeVerifier::new(pkce_verifier.to_string()))
            .request_async(&async_http_client)
            .await
            .map_err(|e| OAuthError::TokenExchangeFailed(e.to_string()))?;

        Ok(OAuthToken {
            access_token: token_response.access_token().secret().clone(),
            refresh_token: token_response
                .refresh_token()
                .map(|t| t.secret().clone()),
            token_type: "Bearer".to_string(),
            expires_at: token_response.expires_in().map(|duration| {
                std::time::SystemTime::now() + std::time::Duration::from_secs(duration.as_secs())
            }),
            scopes: token_response
                .scopes()
                .map(|scopes| scopes.iter().map(|s| s.to_string()).collect()),
        })
    }

    /// Fetch user info JSON from the configured endpoint
    ///
    /// # Arguments
    ///
    /// * `access_token` - OAuth access token
    ///
    /// # Errors
    ///
    /// Returns error if the HTTP request fails or returns non-success status
    pub async fn fetch_user_info_json(
        &self,
        access_token: &str,
    ) -> Result<serde_json::Value, OAuthError> {
        let response = self.http_client
            .get(&self.userinfo_url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| OAuthError::UserInfoFailed(e.to_string()))?;

        Self::check_http_response(response).await
    }

    /// Fetch user info JSON from a custom endpoint with optional headers
    ///
    /// This method allows providers like GitHub to make multiple API calls
    /// with custom headers.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to fetch
    /// * `access_token` - OAuth access token
    /// * `headers` - Optional additional headers (e.g., User-Agent for GitHub)
    ///
    /// # Errors
    ///
    /// Returns error if the HTTP request fails or returns non-success status
    pub async fn fetch_json_with_headers(
        &self,
        url: &str,
        access_token: &str,
        headers: Option<&[(&str, &str)]>,
    ) -> Result<serde_json::Value, OAuthError> {
        let mut request = self.http_client
            .get(url)
            .bearer_auth(access_token);

        if let Some(headers) = headers {
            for (key, value) in headers {
                request = request.header(*key, *value);
            }
        }

        let response = request
            .send()
            .await
            .map_err(|e| OAuthError::UserInfoFailed(e.to_string()))?;

        Self::check_http_response(response).await
    }

    /// Check HTTP response status and parse JSON
    async fn check_http_response(response: reqwest::Response) -> Result<serde_json::Value, OAuthError> {
        if !response.status().is_success() {
            return Err(OAuthError::UserInfoFailed(format!(
                "HTTP {}",
                response.status()
            )));
        }

        response
            .json()
            .await
            .map_err(|e| OAuthError::UserInfoFailed(format!("Failed to parse JSON: {e}")))
    }

    /// Get reference to the userinfo URL
    #[must_use]
    pub fn userinfo_url(&self) -> &str {
        &self.userinfo_url
    }

    /// Get reference to the HTTP client
    #[must_use]
    pub const fn http_client(&self) -> &reqwest::Client {
        &self.http_client
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_provider_creation() {
        let config = ProviderConfig {
            client_id: "test-client-id".to_string(),
            client_secret: "test-client-secret".to_string(),
            redirect_uri: "http://localhost:3000/auth/callback".to_string(),
            scopes: vec![],
            auth_url: None,
            token_url: None,
            userinfo_url: None,
        };

        let provider = BaseOAuthProvider::new(
            "https://example.com/oauth/authorize",
            "https://example.com/oauth/token",
            &config,
            "https://example.com/oauth/userinfo".to_string(),
        );

        assert!(provider.is_ok());
    }

    #[test]
    fn test_authorization_url_generation() {
        let config = ProviderConfig {
            client_id: "test-client-id".to_string(),
            client_secret: "test-client-secret".to_string(),
            redirect_uri: "http://localhost:3000/auth/callback".to_string(),
            scopes: vec![],
            auth_url: None,
            token_url: None,
            userinfo_url: None,
        };

        let provider = BaseOAuthProvider::new(
            "https://example.com/oauth/authorize",
            "https://example.com/oauth/token",
            &config,
            "https://example.com/oauth/userinfo".to_string(),
        )
        .unwrap();

        let (auth_url, csrf_state, pkce_verifier) = provider.authorization_url(&["openid", "email"]);

        assert!(auth_url.starts_with("https://example.com/oauth/authorize"));
        assert!(auth_url.contains("client_id=test-client-id"));
        assert!(auth_url.contains("scope=openid"));
        assert!(!csrf_state.is_empty());
        assert!(!pkce_verifier.is_empty());
    }
}
