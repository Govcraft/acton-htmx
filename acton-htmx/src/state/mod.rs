//! Application state management
//!
//! Combines acton-service infrastructure with acton-reactive actors and
//! HTMX-specific components.

use crate::{config::ActonHtmxConfig, observability::ObservabilityConfig};
use std::sync::Arc;

/// Application state for acton-htmx applications
///
/// Combines:
/// - Configuration (from acton-service)
/// - Observability (from acton-service)
/// - Database pools (from acton-service) - TODO
/// - Redis cache (from acton-service) - TODO
/// - Actor runtime (from acton-reactive) - TODO
/// - Agents (session, CSRF, flash, jobs) - TODO
/// - Template registry - TODO
///
/// # Example
///
/// ```rust,ignore
/// use acton_htmx::state::ActonHtmxState;
///
/// async fn example() -> anyhow::Result<()> {
///     let state = ActonHtmxState::new()?;
///
///     // Use in Axum
///     let app = axum::Router::new()
///         .route("/", axum::routing::get(|| async { "Hello!" }))
///         .with_state(state);
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct ActonHtmxState {
    /// Application configuration
    config: Arc<ActonHtmxConfig>,

    /// Observability configuration
    observability: Arc<ObservabilityConfig>,
    // TODO: Add database, redis, actor runtime, agents, templates
}

impl ActonHtmxState {
    /// Create new application state with defaults
    ///
    /// # Example
    ///
    /// ```rust
    /// use acton_htmx::state::ActonHtmxState;
    ///
    /// # fn example() -> anyhow::Result<()> {
    /// let state = ActonHtmxState::new()?;
    /// # Ok(())
    /// # }
    /// ```
    // TODO: Will become async when actor runtime initialization is added
    pub fn new() -> anyhow::Result<Self> {
        let config = ActonHtmxConfig::default();
        let observability = ObservabilityConfig::default();

        Ok(Self {
            config: Arc::new(config),
            observability: Arc::new(observability),
        })
    }

    /// Create application state with custom configuration
    ///
    /// # Example
    ///
    /// ```rust
    /// use acton_htmx::{config::ActonHtmxConfig, state::ActonHtmxState};
    ///
    /// # fn example() -> anyhow::Result<()> {
    /// let config = ActonHtmxConfig::load_for_service("my-app")?;
    /// let state = ActonHtmxState::with_config(config)?;
    /// # Ok(())
    /// # }
    /// ```
    // TODO: Will become async when actor runtime initialization is added
    pub fn with_config(config: ActonHtmxConfig) -> anyhow::Result<Self> {
        let observability = ObservabilityConfig::new("acton-htmx");

        Ok(Self {
            config: Arc::new(config),
            observability: Arc::new(observability),
        })
    }

    /// Get configuration reference
    ///
    /// # Example
    ///
    /// ```rust
    /// use acton_htmx::state::ActonHtmxState;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let state = ActonHtmxState::new()?;
    /// let config = state.config();
    ///
    /// let timeout = config.htmx.request_timeout_ms;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn config(&self) -> &ActonHtmxConfig {
        &self.config
    }

    /// Get observability configuration
    #[must_use]
    pub fn observability(&self) -> &ObservabilityConfig {
        &self.observability
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_new_state() {
        let state = ActonHtmxState::new().expect("Failed to create state");
        assert_eq!(state.config().htmx.request_timeout_ms, 5000);
    }

    #[test]
    fn test_with_config() {
        let mut config = ActonHtmxConfig::default();
        config.htmx.request_timeout_ms = 10000;

        let state =
            ActonHtmxState::with_config(config).expect("Failed to create state");

        assert_eq!(state.config().htmx.request_timeout_ms, 10000);
    }

    #[test]
    fn test_clone_state() {
        let state = ActonHtmxState::new().expect("Failed to create state");
        let cloned = state.clone();

        // Both should reference the same Arc
        assert!(Arc::ptr_eq(&state.config, &cloned.config));
    }
}
