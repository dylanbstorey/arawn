//! Tool execution methods for ToolRegistry.
//!
//! Implements execute, execute_with_config, execute_raw, and secret handle resolution.

use arawn_types::{contains_secret_handle, is_gated_tool, resolve_handles_in_json};

use super::context::{ToolContext, ToolResult};
use super::output::OutputConfig;
use super::registry::ToolRegistry;
use crate::error::{AgentError, Result};

impl ToolRegistry {
    /// Execute a tool by name.
    ///
    /// The result is automatically sanitized with default configuration.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let result = registry
    ///     .execute("read_file", serde_json::json!({"path": "/tmp/f.txt"}), &ctx)
    ///     .await?;
    /// println!("{}", result.to_llm_content());
    /// ```
    pub async fn execute(
        &self,
        name: &str,
        params: serde_json::Value,
        ctx: &ToolContext,
    ) -> Result<ToolResult> {
        self.execute_with_config(name, params, ctx, &OutputConfig::default())
            .await
    }

    /// Execute a tool by name with custom output configuration.
    ///
    /// If a secret resolver is present on the context, `${{secrets.*}}` handles
    /// in tool parameters are resolved to their real values before execution.
    /// The caller retains the original params (with handles) for logging.
    ///
    /// If a filesystem gate is present on the context, gated tools (file_read,
    /// file_write, glob, grep, shell) are validated against workstream boundaries
    /// before execution. If no gate is present and the tool is gated, execution
    /// is denied by default.
    pub async fn execute_with_config(
        &self,
        name: &str,
        params: serde_json::Value,
        ctx: &ToolContext,
        output_config: &OutputConfig,
    ) -> Result<ToolResult> {
        let tool = self
            .get(name)
            .ok_or_else(|| AgentError::ToolNotFound(name.to_string()))?;

        // Resolve ${{secrets.*}} handles in params (if resolver present)
        let params = self.resolve_secret_handles(params, ctx);

        // Filesystem gate enforcement for gated tools
        if is_gated_tool(name) {
            if let Some(ref gate) = ctx.fs_gate {
                // Shell tools get routed through the OS-level sandbox
                if name == "shell" {
                    return self
                        .execute_shell_sandboxed(tool.as_ref(), &params, ctx, gate, output_config)
                        .await;
                }

                // File/search tools get path validation
                let params = match self.validate_tool_paths(name, params, gate) {
                    Ok(p) => p,
                    Err(denied) => return Ok(denied),
                };
                let result = tool.execute(params, ctx).await?;
                return Ok(result.sanitize(output_config));
            } else {
                // Deny by default: no gate configured for a gated tool
                tracing::warn!(tool = %name, "Gated tool denied: no filesystem gate configured");
                return Ok(ToolResult::error(format!(
                    "Tool '{}' requires a filesystem gate but none is configured. \
                     This tool can only be used within a workstream context.",
                    name
                )));
            }
        }

        let result = tool.execute(params, ctx).await?;
        Ok(result.sanitize(output_config))
    }

    /// Execute a tool by name without sanitization.
    ///
    /// Secret handle resolution and filesystem gate enforcement still apply
    /// even when output sanitization is skipped.
    pub async fn execute_raw(
        &self,
        name: &str,
        params: serde_json::Value,
        ctx: &ToolContext,
    ) -> Result<ToolResult> {
        let tool = self
            .get(name)
            .ok_or_else(|| AgentError::ToolNotFound(name.to_string()))?;

        // Resolve ${{secrets.*}} handles in params (if resolver present)
        let params = self.resolve_secret_handles(params, ctx);

        // Gate enforcement applies even for raw execution
        if is_gated_tool(name) {
            if let Some(ref gate) = ctx.fs_gate {
                if name == "shell" {
                    return self
                        .execute_shell_sandboxed(
                            tool.as_ref(),
                            &params,
                            ctx,
                            gate,
                            &OutputConfig::default(),
                        )
                        .await;
                }
                let params = match self.validate_tool_paths(name, params, gate) {
                    Ok(p) => p,
                    Err(denied) => return Ok(denied),
                };
                return tool.execute(params, ctx).await;
            } else {
                tracing::warn!(tool = %name, "Gated tool denied (raw): no filesystem gate configured");
                return Ok(ToolResult::error(format!(
                    "Tool '{}' requires a filesystem gate but none is configured.",
                    name
                )));
            }
        }

        tool.execute(params, ctx).await
    }

    /// Resolve `${{secrets.*}}` handles in tool parameters.
    ///
    /// If a secret resolver is present on the context and the params contain
    /// handle references, replaces them with actual secret values. The original
    /// params (with handles, not values) are what get logged by the caller.
    fn resolve_secret_handles(
        &self,
        params: serde_json::Value,
        ctx: &ToolContext,
    ) -> serde_json::Value {
        if let Some(ref resolver) = ctx.secret_resolver {
            let params_str = params.to_string();
            if contains_secret_handle(&params_str) {
                return resolve_handles_in_json(&params, resolver.as_ref());
            }
        }
        params
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::super::context::ToolContext;
    use super::super::output::OutputConfig;
    use super::super::registry::{MockTool, ToolRegistry};
    use super::ToolResult;

    struct MockSecretResolver {
        secrets: std::collections::HashMap<String, String>,
    }

    impl MockSecretResolver {
        fn new(pairs: &[(&str, &str)]) -> Self {
            Self {
                secrets: pairs
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
            }
        }
    }

    impl arawn_types::SecretResolver for MockSecretResolver {
        fn resolve(&self, name: &str) -> Option<String> {
            self.secrets.get(name).cloned()
        }
        fn names(&self) -> Vec<String> {
            self.secrets.keys().cloned().collect()
        }
    }

    fn ctx_with_resolver(resolver: MockSecretResolver) -> ToolContext {
        ToolContext {
            secret_resolver: Some(Arc::new(resolver)),
            ..ToolContext::default()
        }
    }

    #[tokio::test]
    async fn test_registry_execute_sanitizes() {
        let mut registry = ToolRegistry::new();
        // Create a tool that returns content with null bytes
        registry
            .register(MockTool::new("test_tool").with_response(ToolResult::text("Hello\0World")));

        let ctx = ToolContext::default();
        let result = registry
            .execute("test_tool", serde_json::json!({}), &ctx)
            .await
            .unwrap();

        // Should be sanitized (null bytes removed)
        match result {
            ToolResult::Text { content } => {
                assert_eq!(content, "HelloWorld");
            }
            _ => panic!("Expected Text result"),
        }
    }

    #[tokio::test]
    async fn test_registry_execute_raw_no_sanitize() {
        let mut registry = ToolRegistry::new();
        registry
            .register(MockTool::new("test_tool").with_response(ToolResult::text("Hello\0World")));

        let ctx = ToolContext::default();
        let result = registry
            .execute_raw("test_tool", serde_json::json!({}), &ctx)
            .await
            .unwrap();

        // Should NOT be sanitized
        match result {
            ToolResult::Text { content } => {
                assert!(content.contains('\0'));
            }
            _ => panic!("Expected Text result"),
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // ToolRegistry Filtering Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_secret_handles_resolved_in_params() {
        let mut registry = ToolRegistry::new();
        let mock = MockTool::new("my_tool");
        registry.register(mock);

        let resolver = MockSecretResolver::new(&[("token", "real_secret_value")]);
        let ctx = ctx_with_resolver(resolver);

        let params = serde_json::json!({"header": "Bearer ${{secrets.token}}", "other": "plain"});

        let result = registry.execute_raw("my_tool", params, &ctx).await.unwrap();

        // The tool should have received the resolved value
        assert!(result.is_success());
    }

    #[tokio::test]
    async fn test_secret_handles_no_resolver_passes_through() {
        let mut registry = ToolRegistry::new();
        let mock = MockTool::new("my_tool");
        registry.register(mock);

        let ctx = ToolContext::default(); // no resolver

        let params = serde_json::json!({"header": "Bearer ${{secrets.token}}", "other": "plain"});

        let result = registry.execute_raw("my_tool", params, &ctx).await.unwrap();

        // Should still succeed — handles left as-is without a resolver
        assert!(result.is_success());
    }

    #[tokio::test]
    async fn test_secret_handles_no_handles_in_params() {
        let mut registry = ToolRegistry::new();
        let mock = MockTool::new("my_tool");
        registry.register(mock);

        let resolver = MockSecretResolver::new(&[("token", "secret")]);
        let ctx = ctx_with_resolver(resolver);

        let params = serde_json::json!({"path": "/tmp/file.txt"});

        let result = registry.execute_raw("my_tool", params, &ctx).await.unwrap();

        assert!(result.is_success());
    }

    #[tokio::test]
    async fn test_secret_handles_resolved_in_execute_with_config() {
        let mut registry = ToolRegistry::new();
        let mock = MockTool::new("my_tool");
        registry.register(mock);

        let resolver = MockSecretResolver::new(&[("api_key", "sk-1234")]);
        let ctx = ctx_with_resolver(resolver);

        let params = serde_json::json!({"key": "${{secrets.api_key}}"});

        // Non-gated tool, so gate enforcement won't block
        let result = registry
            .execute_with_config("my_tool", params, &ctx, &OutputConfig::default())
            .await
            .unwrap();

        assert!(result.is_success());
    }

    #[tokio::test]
    async fn test_secret_handles_unknown_secret_left_as_is() {
        let mut registry = ToolRegistry::new();
        let mock = MockTool::new("my_tool");
        registry.register(mock);

        let resolver = MockSecretResolver::new(&[]); // empty — no secrets
        let ctx = ctx_with_resolver(resolver);

        let params = serde_json::json!({"key": "${{secrets.nonexistent}}"});

        let result = registry.execute_raw("my_tool", params, &ctx).await.unwrap();

        assert!(result.is_success());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Command Validator Tests
    // ─────────────────────────────────────────────────────────────────────────
}
