//! Middleware layer construction helpers
//!
//! This module provides helper utilities for creating middleware layers with
//! consistent patterns across the framework. These helpers reduce boilerplate
//! while maintaining clarity and idiomatic Rust code.

/// Helper macro for creating standard middleware layer constructors
///
/// This macro generates the common constructor patterns that most middleware
/// layers need: `new()`, `with_config()`, and `from_handle()`.
///
/// # Example
///
/// ```rust,ignore
/// use acton_htmx::middleware::middleware_constructors;
/// use acton_htmx::state::ActonHtmxState;
/// use acton_reactive::prelude::AgentHandle;
///
/// pub struct MyMiddleware {
///     agent_handle: AgentHandle<MyAgent>,
/// }
///
/// middleware_constructors!(
///     MyMiddleware,        // Middleware type
///     MyAgent,             // Agent type
///     agent_handle,        // Field name for the handle
///     MyConfig             // Config type
/// );
/// ```
///
/// This generates:
/// ```rust,ignore
/// impl MyMiddleware {
///     pub fn new(state: &ActonHtmxState) -> Self {
///         Self {
///             agent_handle: state.my_agent().clone(),
///         }
///     }
///
///     pub fn with_config(state: &ActonHtmxState, _config: MyConfig) -> Self {
///         Self::new(state)
///     }
///
///     pub fn from_handle(handle: AgentHandle<MyAgent>) -> Self {
///         Self {
///             agent_handle: handle,
///         }
///     }
/// }
/// ```
#[macro_export]
macro_rules! middleware_constructors {
    ($middleware:ty, $agent:ty, $field:ident, $config:ty, $state_method:ident) => {
        impl $middleware {
            /// Create middleware from application state
            ///
            /// This is the standard way to create middleware when adding it to your router.
            #[must_use]
            pub fn new(state: &$crate::state::ActonHtmxState) -> Self {
                Self {
                    $field: state.$state_method().clone(),
                }
            }

            /// Create middleware with custom configuration
            ///
            /// Note: Most middleware layers ignore the config parameter and use
            /// the configuration from the agent initialization. This method exists
            /// for API consistency.
            #[must_use]
            pub fn with_config(state: &$crate::state::ActonHtmxState, _config: $config) -> Self {
                Self::new(state)
            }

            /// Create middleware directly from an agent handle
            ///
            /// This is useful for testing or when you have a custom agent instance.
            #[must_use]
            pub const fn from_handle(handle: acton_reactive::prelude::AgentHandle<$agent>) -> Self {
                Self { $field: handle }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    // Macro usage is tested within the actual middleware implementations
    // (session, csrf, auth) which use this macro.
}
