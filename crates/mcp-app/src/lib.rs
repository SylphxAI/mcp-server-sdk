//! MCP application / legacy server product state builders.
//!
//! Parity with:
//! - `src/app/app.ts` (`createMcpApp` / `buildServerState`)
//! - `src/server/server.ts` (`createServer` / `buildState`)
//!
//! Pure inventory → capabilities + identity. Handler dispatch and transport
//! binding remain application-layer. WAVE4: rust_impl product surface.

use mcp_builders::{
    PromptDefinitionMeta, ResourceDefinitionMeta, ResourceTemplateDefinitionMeta,
    ToolDefinitionMeta,
};
use mcp_protocol::server_capabilities;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// Default server name (parity with TS).
pub const DEFAULT_SERVER_NAME: &str = "mcp-server";
/// Default server version (parity with TS).
pub const DEFAULT_SERVER_VERSION: &str = "1.0.0";

/// Capability emission flavor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CapabilityStyle {
    /// `createMcpApp`: tools/prompts as empty objects; resources subscribe only.
    #[default]
    McpApp,
    /// `createServer` (legacy): listChanged flags + resources subscribe+listChanged.
    LegacyCreateServer,
}

/// Product inventory counts used to derive capabilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct InventoryCounts {
    pub tools: usize,
    pub resources: usize,
    pub resource_templates: usize,
    pub prompts: usize,
}

impl InventoryCounts {
    #[must_use]
    pub fn has_resources_surface(self) -> bool {
        self.resources > 0 || self.resource_templates > 0
    }
}

/// Derive server capabilities JSON from inventory (parity with TS buildState variants).
#[must_use]
pub fn capabilities_from_inventory(counts: InventoryCounts, style: CapabilityStyle) -> Value {
    match style {
        CapabilityStyle::McpApp => {
            // createMcpApp: empty capability objects (no listChanged flags).
            mcp_app_capabilities(counts)
        }
        CapabilityStyle::LegacyCreateServer => {
            // createServer:
            // tools: { listChanged: true } when tools
            // resources: { subscribe: true, listChanged: true } when resources or templates
            // prompts: { listChanged: true } when prompts
            // always logging + completions
            server_capabilities(
                if counts.tools > 0 { Some(true) } else { None },
                if counts.has_resources_surface() {
                    Some(true)
                } else {
                    None
                },
                if counts.has_resources_surface() {
                    Some(true)
                } else {
                    None
                },
                if counts.prompts > 0 { Some(true) } else { None },
                true,
                true,
            )
        }
    }
}

/// createMcpApp capability shape (empty capability objects, no listChanged).
#[must_use]
pub fn mcp_app_capabilities(counts: InventoryCounts) -> Value {
    let mut map = serde_json::Map::new();
    if counts.tools > 0 {
        map.insert("tools".into(), serde_json::json!({}));
    }
    if counts.has_resources_surface() {
        map.insert(
            "resources".into(),
            serde_json::json!({ "subscribe": true }),
        );
    }
    if counts.prompts > 0 {
        map.insert("prompts".into(), serde_json::json!({}));
    }
    map.insert("logging".into(), serde_json::json!({}));
    map.insert("completions".into(), serde_json::json!({}));
    Value::Object(map)
}

/// Resolved server identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerIdentity {
    pub name: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
}

impl ServerIdentity {
    #[must_use]
    pub fn resolve(name: Option<String>, version: Option<String>, instructions: Option<String>) -> Self {
        Self {
            name: name.unwrap_or_else(|| DEFAULT_SERVER_NAME.into()),
            version: version.unwrap_or_else(|| DEFAULT_SERVER_VERSION.into()),
            instructions,
        }
    }
}

/// Full pure app/server state snapshot (handler-free).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStateMeta {
    pub name: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    pub tools: BTreeMap<String, ToolDefinitionMeta>,
    pub resources: BTreeMap<String, ResourceDefinitionMeta>,
    pub resource_templates: BTreeMap<String, ResourceTemplateDefinitionMeta>,
    pub prompts: BTreeMap<String, PromptDefinitionMeta>,
    pub capabilities: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<PaginationOptionsMeta>,
}

/// Pagination options meta (parity with PaginationOptions).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginationOptionsMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_page_size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_page_size: Option<u32>,
}

/// Config for building app state (parity with McpAppConfig / ServerConfig fields).
#[derive(Debug, Clone, Default)]
pub struct AppConfig {
    pub name: Option<String>,
    pub version: Option<String>,
    pub instructions: Option<String>,
    pub tools: BTreeMap<String, ToolDefinitionMeta>,
    pub resources: BTreeMap<String, ResourceDefinitionMeta>,
    pub resource_templates: BTreeMap<String, ResourceTemplateDefinitionMeta>,
    pub prompts: BTreeMap<String, PromptDefinitionMeta>,
    pub pagination: Option<PaginationOptionsMeta>,
    pub style: CapabilityStyle,
}

impl AppConfig {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    #[must_use]
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    #[must_use]
    pub fn with_instructions(mut self, instructions: impl Into<String>) -> Self {
        self.instructions = Some(instructions.into());
        self
    }

    #[must_use]
    pub fn with_tool(mut self, name: impl Into<String>, def: ToolDefinitionMeta) -> Self {
        self.tools.insert(name.into(), def);
        self
    }

    #[must_use]
    pub fn with_resource(mut self, name: impl Into<String>, def: ResourceDefinitionMeta) -> Self {
        self.resources.insert(name.into(), def);
        self
    }

    #[must_use]
    pub fn with_resource_template(
        mut self,
        name: impl Into<String>,
        def: ResourceTemplateDefinitionMeta,
    ) -> Self {
        self.resource_templates.insert(name.into(), def);
        self
    }

    #[must_use]
    pub fn with_prompt(mut self, name: impl Into<String>, def: PromptDefinitionMeta) -> Self {
        self.prompts.insert(name.into(), def);
        self
    }

    #[must_use]
    pub fn with_style(mut self, style: CapabilityStyle) -> Self {
        self.style = style;
        self
    }

    #[must_use]
    pub fn with_pagination(mut self, pagination: PaginationOptionsMeta) -> Self {
        self.pagination = Some(pagination);
        self
    }

    /// Build pure app state (parity: `createMcpApp` / `createServer` state).
    #[must_use]
    pub fn build(self) -> AppStateMeta {
        let identity = ServerIdentity::resolve(self.name, self.version, self.instructions);
        let counts = InventoryCounts {
            tools: self.tools.len(),
            resources: self.resources.len(),
            resource_templates: self.resource_templates.len(),
            prompts: self.prompts.len(),
        };
        AppStateMeta {
            name: identity.name,
            version: identity.version,
            instructions: identity.instructions,
            tools: self.tools,
            resources: self.resources,
            resource_templates: self.resource_templates,
            prompts: self.prompts,
            capabilities: capabilities_from_inventory(counts, self.style),
            pagination: self.pagination,
        }
    }
}

/// Convenience: createMcpApp-style state.
#[must_use]
pub fn create_mcp_app_state(config: AppConfig) -> AppStateMeta {
    config.with_style(CapabilityStyle::McpApp).build()
}

/// Convenience: legacy createServer-style state.
#[must_use]
pub fn create_server_state(config: AppConfig) -> AppStateMeta {
    config
        .with_style(CapabilityStyle::LegacyCreateServer)
        .build()
}

/// Initialize result serverInfo envelope from identity.
#[must_use]
pub fn server_info_body(identity: &ServerIdentity) -> Value {
    serde_json::json!({
        "name": identity.name,
        "version": identity.version,
    })
}

/// Count helpers for inventories.
#[must_use]
pub fn inventory_counts(state: &AppStateMeta) -> InventoryCounts {
    InventoryCounts {
        tools: state.tools.len(),
        resources: state.resources.len(),
        resource_templates: state.resource_templates.len(),
        prompts: state.prompts.len(),
    }
}

// ─── Conformance example catalog (delivery/conformance-examples) ────────────

/// Conformance tool names required by `@modelcontextprotocol/conformance`.
/// Pure catalog — parity with `examples/conformance-server.ts` tool keys.
pub const CONFORMANCE_TOOL_NAMES: &[&str] = &[
    "test_simple_text",
    "test_image_content",
    "test_audio_content",
    "test_multiple_content_types",
    "test_error_handling",
    "test_embedded_resource",
    "test_tool_with_logging",
    "test_tool_with_progress",
];

/// Conformance resource URIs present in the example server.
pub const CONFORMANCE_RESOURCE_URIS: &[&str] = &[
    "test://static/resource",
    "test://static/resource/blob",
];

/// Conformance prompt names.
pub const CONFORMANCE_PROMPT_NAMES: &[&str] = &["test_simple_prompt", "test_prompt_with_args"];

/// True when a tool name is part of the conformance catalog.
#[must_use]
pub fn is_conformance_tool(name: &str) -> bool {
    CONFORMANCE_TOOL_NAMES.contains(&name)
}

/// Minimal conformance app config (metadata only — no handlers).
#[must_use]
pub fn conformance_app_config() -> AppConfig {
    use mcp_builders::{prompt, prompt_arg, resource, resource_template, tool};
    let mut cfg = AppConfig::new()
        .with_name("conformance-server")
        .with_version("1.0.0")
        .with_style(CapabilityStyle::McpApp);
    for name in CONFORMANCE_TOOL_NAMES {
        cfg = cfg.with_tool(*name, tool().description(*name).finish());
    }
    if let Some(res) = resource()
        .uri(CONFORMANCE_RESOURCE_URIS[0])
        .mime_type("text/plain")
        .finish()
    {
        cfg = cfg.with_resource("static", res);
    }
    if let Some(res) = resource()
        .uri(CONFORMANCE_RESOURCE_URIS[1])
        .mime_type("application/octet-stream")
        .finish()
    {
        cfg = cfg.with_resource("static_blob", res);
    }
    if let Some(tmpl) = resource_template()
        .uri_template("test://dynamic/{id}")
        .finish()
    {
        cfg = cfg.with_resource_template("template", tmpl);
    }
    cfg = cfg.with_prompt(
        CONFORMANCE_PROMPT_NAMES[0],
        prompt().description("simple").finish(),
    );
    cfg = cfg.with_prompt(
        CONFORMANCE_PROMPT_NAMES[1],
        prompt()
            .description("with args")
            .argument(prompt_arg("name", Some("Name".into()), Some(true)))
            .finish(),
    );
    cfg
}

#[cfg(test)]
mod tests {
    use super::*;
    use mcp_builders::{prompt, resource, resource_template, tool};

    #[test]
    fn defaults_identity() {
        let id = ServerIdentity::resolve(None, None, None);
        assert_eq!(id.name, DEFAULT_SERVER_NAME);
        assert_eq!(id.version, DEFAULT_SERVER_VERSION);
        let id2 = ServerIdentity::resolve(Some("x".into()), Some("9".into()), Some("hi".into()));
        assert_eq!(id2.name, "x");
        assert_eq!(id2.instructions.as_deref(), Some("hi"));
    }

    #[test]
    fn mcp_app_capabilities_empty_objects() {
        let caps = mcp_app_capabilities(InventoryCounts {
            tools: 1,
            resources: 0,
            resource_templates: 1,
            prompts: 1,
        });
        assert_eq!(caps["tools"], serde_json::json!({}));
        assert_eq!(caps["resources"]["subscribe"], true);
        assert!(caps["resources"].get("listChanged").is_none());
        assert_eq!(caps["prompts"], serde_json::json!({}));
        assert!(caps["logging"].is_object());
        assert!(caps["completions"].is_object());
    }

    #[test]
    fn legacy_create_server_capabilities_list_changed() {
        let caps = capabilities_from_inventory(
            InventoryCounts {
                tools: 2,
                resources: 1,
                resource_templates: 0,
                prompts: 1,
            },
            CapabilityStyle::LegacyCreateServer,
        );
        assert_eq!(caps["tools"]["listChanged"], true);
        assert_eq!(caps["resources"]["subscribe"], true);
        assert_eq!(caps["resources"]["listChanged"], true);
        assert_eq!(caps["prompts"]["listChanged"], true);
    }

    #[test]
    fn create_mcp_app_state_builds_inventory() {
        let state = create_mcp_app_state(
            AppConfig::new()
                .with_name("demo")
                .with_tool("ping", tool().description("p").finish())
                .with_resource(
                    "readme",
                    resource()
                        .uri("file:///README.md")
                        .finish()
                        .expect("uri"),
                )
                .with_resource_template(
                    "file",
                    resource_template()
                        .uri_template("file:///{path}")
                        .finish()
                        .expect("t"),
                )
                .with_prompt("hello", prompt().description("h").finish()),
        );
        assert_eq!(state.name, "demo");
        assert_eq!(state.version, DEFAULT_SERVER_VERSION);
        assert_eq!(state.tools.len(), 1);
        assert_eq!(state.resources.len(), 1);
        assert_eq!(state.resource_templates.len(), 1);
        assert_eq!(state.prompts.len(), 1);
        assert_eq!(state.capabilities["tools"], serde_json::json!({}));
        assert_eq!(state.capabilities["resources"]["subscribe"], true);
        let counts = inventory_counts(&state);
        assert_eq!(counts.tools, 1);
        assert!(counts.has_resources_surface());
    }

    #[test]
    fn create_server_state_legacy_style() {
        let state = create_server_state(
            AppConfig::new()
                .with_version("2.0.0")
                .with_tool("t", tool().finish())
                .with_prompt("p", prompt().finish()),
        );
        assert_eq!(state.version, "2.0.0");
        assert_eq!(state.capabilities["tools"]["listChanged"], true);
        assert_eq!(state.capabilities["prompts"]["listChanged"], true);
        assert!(state.capabilities.get("resources").is_none());
    }

    #[test]
    fn empty_inventory_still_has_logging() {
        let state = create_mcp_app_state(AppConfig::new());
        assert!(state.capabilities["logging"].is_object());
        assert!(state.capabilities["completions"].is_object());
        assert!(state.capabilities.get("tools").is_none());
        let info = server_info_body(&ServerIdentity::resolve(
            Some(state.name.clone()),
            Some(state.version.clone()),
            None,
        ));
        assert_eq!(info["name"], DEFAULT_SERVER_NAME);
    }

    #[test]
    fn wave4_product_app_pagination_meta() {
        let state = AppConfig::new()
            .with_pagination(PaginationOptionsMeta {
                default_page_size: Some(10),
                max_page_size: Some(50),
            })
            .build();
        assert_eq!(
            state.pagination.as_ref().and_then(|p| p.default_page_size),
            Some(10)
        );
    }

    #[test]
    fn conformance_catalog_and_config() {
        assert!(is_conformance_tool("test_simple_text"));
        assert!(!is_conformance_tool("not_a_tool"));
        let state = create_mcp_app_state(conformance_app_config());
        assert_eq!(state.name, "conformance-server");
        assert_eq!(state.tools.len(), CONFORMANCE_TOOL_NAMES.len());
        assert!(state.capabilities["tools"].is_object());
        assert!(state.capabilities["resources"]["subscribe"].as_bool().unwrap_or(false));
        assert!(state.prompts.contains_key(CONFORMANCE_PROMPT_NAMES[1]));
    }
}
