//! Product builders for tools, resources, and prompts.
//!
//! Parity with `src/builders/{tool,resource,prompt}.ts` for pure definition
//! shapes, protocol conversion, content helpers, and template/interpolation
//! utilities. Handler execution (async closures + Vex schema validation I/O)
//! stays at the application layer; definition metadata is Rust product authority.
//!
//! WAVE4: rust_impl product surface. TS is read-only oracle.

use mcp_protocol::{
    audio_content, image_content, interpolate, matches_template, normalize_tool_result,
    protocol_prompt, protocol_resource, protocol_template, protocol_tool, prompt_message,
    prompt_result, resource_blob, resource_text_result, text_content, tool_error,
    user_message, Content, Prompt, PromptArgument, Resource, ResourceTemplate, Tool,
    ToolAnnotations,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;

// ─── Empty / default schemas ────────────────────────────────────────────────

/// Default tool input schema when no input is declared (`{ type: object, properties: {} }`).
#[must_use]
pub fn empty_object_schema() -> Value {
    json!({ "type": "object", "properties": {} })
}

// ─── Tool definition (metadata only) ────────────────────────────────────────

/// Tool definition metadata (handler-free product shape).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolDefinitionMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub input_schema: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<ToolAnnotations>,
}

impl ToolDefinitionMeta {
    #[must_use]
    pub fn new() -> Self {
        Self {
            name: None,
            description: None,
            input_schema: empty_object_schema(),
            annotations: None,
        }
    }

    #[must_use]
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    #[must_use]
    pub fn with_input_schema(mut self, schema: Value) -> Self {
        self.input_schema = schema;
        self
    }

    #[must_use]
    pub fn with_annotations(mut self, annotations: ToolAnnotations) -> Self {
        self.annotations = Some(annotations);
        self
    }

    /// Convert to MCP protocol tool (parity: `toProtocolTool`).
    #[must_use]
    pub fn to_protocol(&self, name: impl Into<String>) -> Tool {
        protocol_tool(
            name,
            self.description.clone(),
            self.input_schema.clone(),
            self.annotations.clone(),
        )
    }
}

impl Default for ToolDefinitionMeta {
    fn default() -> Self {
        Self::new()
    }
}

/// Fluent tool builder (pure metadata).
#[derive(Debug, Clone, Default)]
pub struct ToolBuilder {
    description: Option<String>,
    annotations: Option<ToolAnnotations>,
    input_schema: Option<Value>,
}

impl ToolBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    #[must_use]
    pub fn annotations(mut self, annotations: ToolAnnotations) -> Self {
        self.annotations = Some(annotations);
        self
    }

    #[must_use]
    pub fn input_schema(mut self, schema: Value) -> Self {
        self.input_schema = Some(schema);
        self
    }

    /// Finish without a handler — returns definition metadata.
    #[must_use]
    pub fn finish(self) -> ToolDefinitionMeta {
        ToolDefinitionMeta {
            name: None,
            description: self.description,
            input_schema: self.input_schema.unwrap_or_else(empty_object_schema),
            annotations: self.annotations,
        }
    }
}

/// Start a tool builder (parity: `tool()`).
#[must_use]
pub fn tool() -> ToolBuilder {
    ToolBuilder::new()
}

// ─── Resource definition ────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceDefinitionMeta {
    pub uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

impl ResourceDefinitionMeta {
    #[must_use]
    pub fn to_protocol(&self, name: impl Into<String>) -> Resource {
        protocol_resource(
            name,
            self.uri.clone(),
            self.description.clone(),
            self.mime_type.clone(),
        )
    }
}

#[derive(Debug, Clone, Default)]
pub struct ResourceBuilder {
    uri: Option<String>,
    description: Option<String>,
    mime_type: Option<String>,
}

impl ResourceBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn uri(mut self, uri: impl Into<String>) -> Self {
        self.uri = Some(uri.into());
        self
    }

    #[must_use]
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    #[must_use]
    pub fn mime_type(mut self, mime: impl Into<String>) -> Self {
        self.mime_type = Some(mime.into());
        self
    }

    /// Finish resource definition. Returns None when uri is missing.
    #[must_use]
    pub fn finish(self) -> Option<ResourceDefinitionMeta> {
        Some(ResourceDefinitionMeta {
            uri: self.uri?,
            description: self.description,
            mime_type: self.mime_type,
        })
    }
}

#[must_use]
pub fn resource() -> ResourceBuilder {
    ResourceBuilder::new()
}

// ─── Resource template ──────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceTemplateDefinitionMeta {
    pub uri_template: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

impl ResourceTemplateDefinitionMeta {
    #[must_use]
    pub fn to_protocol(&self, name: impl Into<String>) -> ResourceTemplate {
        protocol_template(
            name,
            self.uri_template.clone(),
            self.description.clone(),
            self.mime_type.clone(),
        )
    }

    /// Extract path params from a concrete URI against this template.
    #[must_use]
    pub fn extract_params(&self, uri: &str) -> Option<BTreeMap<String, String>> {
        mcp_protocol::extract_params(&self.uri_template, uri)
    }

    #[must_use]
    pub fn matches(&self, uri: &str) -> bool {
        matches_template(&self.uri_template, uri)
    }
}

#[derive(Debug, Clone, Default)]
pub struct ResourceTemplateBuilder {
    uri_template: Option<String>,
    description: Option<String>,
    mime_type: Option<String>,
}

impl ResourceTemplateBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn uri_template(mut self, t: impl Into<String>) -> Self {
        self.uri_template = Some(t.into());
        self
    }

    #[must_use]
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    #[must_use]
    pub fn mime_type(mut self, mime: impl Into<String>) -> Self {
        self.mime_type = Some(mime.into());
        self
    }

    #[must_use]
    pub fn finish(self) -> Option<ResourceTemplateDefinitionMeta> {
        Some(ResourceTemplateDefinitionMeta {
            uri_template: self.uri_template?,
            description: self.description,
            mime_type: self.mime_type,
        })
    }
}

#[must_use]
pub fn resource_template() -> ResourceTemplateBuilder {
    ResourceTemplateBuilder::new()
}

// ─── Prompt definition ──────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptDefinitionMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub arguments: Vec<PromptArgument>,
}

impl PromptDefinitionMeta {
    #[must_use]
    pub fn to_protocol(&self, name: impl Into<String>) -> Prompt {
        let args = if self.arguments.is_empty() {
            None
        } else {
            Some(self.arguments.clone())
        };
        protocol_prompt(name, self.description.clone(), args)
    }
}

#[derive(Debug, Clone, Default)]
pub struct PromptBuilder {
    description: Option<String>,
    arguments: Vec<PromptArgument>,
}

impl PromptBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    #[must_use]
    pub fn argument(mut self, arg: PromptArgument) -> Self {
        self.arguments.push(arg);
        self
    }

    #[must_use]
    pub fn arguments(mut self, args: Vec<PromptArgument>) -> Self {
        self.arguments = args;
        self
    }

    #[must_use]
    pub fn finish(self) -> PromptDefinitionMeta {
        PromptDefinitionMeta {
            description: self.description,
            arguments: self.arguments,
        }
    }
}

#[must_use]
pub fn prompt() -> PromptBuilder {
    PromptBuilder::new()
}

/// Build a prompt argument descriptor.
#[must_use]
pub fn prompt_arg(
    name: impl Into<String>,
    description: Option<String>,
    required: Option<bool>,
) -> PromptArgument {
    PromptArgument {
        name: name.into(),
        description,
        required,
    }
}

// ─── Content / result helpers (product re-exports + densify) ────────────────

/// Text content (parity: `text`).
#[must_use]
pub fn text(content: impl Into<String>) -> Content {
    text_content(content)
}

/// Image content (parity: `image`).
#[must_use]
pub fn image(data: impl Into<String>, mime_type: impl Into<String>) -> Content {
    image_content(data, mime_type)
}

/// Audio content (parity: `audio`).
#[must_use]
pub fn audio(data: impl Into<String>, mime_type: impl Into<String>) -> Content {
    audio_content(data, mime_type)
}

/// tools/call error result (parity: `toolError`).
#[must_use]
pub fn tool_err(message: impl Into<String>) -> Value {
    tool_error(message)
}

/// Pretty JSON text content (parity: `json`).
#[must_use]
pub fn json_content(data: &Value) -> Content {
    mcp_protocol::json_text_content(data)
}

/// Normalize a handler result value into ToolsCallResult shape.
#[must_use]
pub fn normalize_result(result: &Value) -> Value {
    normalize_tool_result(result)
}

/// resourceText envelope.
#[must_use]
pub fn resource_text(uri: impl Into<String>, body: impl Into<String>, mime: Option<String>) -> Value {
    resource_text_result(uri, body, mime)
}

/// resourceBlob envelope.
#[must_use]
pub fn resource_blob_b64(
    uri: impl Into<String>,
    blob: impl Into<String>,
    mime: impl Into<String>,
) -> Value {
    resource_blob(uri, blob, mime)
}

/// user prompt message helper.
#[must_use]
pub fn user(text: impl Into<String>) -> mcp_protocol::PromptMessage {
    user_message(text)
}

/// assistant prompt message helper.
#[must_use]
pub fn assistant(text: impl Into<String>) -> mcp_protocol::PromptMessage {
    mcp_protocol::assistant_message(text)
}

/// messages(...) → PromptsGetResult.
#[must_use]
pub fn messages(msgs: &[mcp_protocol::PromptMessage]) -> Value {
    prompt_result("", msgs)
}

/// promptResult(description, ...msgs).
#[must_use]
pub fn prompt_result_with_desc(
    description: impl Into<String>,
    msgs: &[mcp_protocol::PromptMessage],
) -> Value {
    prompt_result(description, msgs)
}

/// message(role, content).
#[must_use]
pub fn message(role: impl Into<String>, content: Content) -> mcp_protocol::PromptMessage {
    prompt_message(role, content)
}

/// Template interpolate `{{key}}`.
#[must_use]
pub fn interpolate_template(template: &str, args: &BTreeMap<String, String>) -> String {
    interpolate(template, args)
}

/// Convert tool definition map entry names into protocol tools list.
#[must_use]
pub fn tools_to_protocol(defs: &BTreeMap<String, ToolDefinitionMeta>) -> Vec<Tool> {
    defs.iter()
        .map(|(name, def)| def.to_protocol(name.clone()))
        .collect()
}

/// Convert resource definition map into protocol resources.
#[must_use]
pub fn resources_to_protocol(defs: &BTreeMap<String, ResourceDefinitionMeta>) -> Vec<Resource> {
    defs.iter()
        .map(|(name, def)| def.to_protocol(name.clone()))
        .collect()
}

/// Convert prompt definition map into protocol prompts.
#[must_use]
pub fn prompts_to_protocol(defs: &BTreeMap<String, PromptDefinitionMeta>) -> Vec<Prompt> {
    defs.iter()
        .map(|(name, def)| def.to_protocol(name.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_builder_empty_schema_and_protocol() {
        let def = tool()
            .description("Ping")
            .finish();
        assert_eq!(def.description.as_deref(), Some("Ping"));
        assert_eq!(def.input_schema, empty_object_schema());
        let pt = def.to_protocol("ping");
        assert_eq!(pt.name, "ping");
        assert_eq!(pt.description.as_deref(), Some("Ping"));
        assert_eq!(pt.input_schema["type"], "object");
    }

    #[test]
    fn tool_builder_with_schema() {
        let schema = json!({"type":"object","properties":{"name":{"type":"string"}},"required":["name"]});
        let def = tool().input_schema(schema.clone()).description("Greet").finish();
        assert_eq!(def.input_schema, schema);
        let pt = def.to_protocol("greet");
        assert_eq!(pt.input_schema["required"][0], "name");
    }

    #[test]
    fn resource_builder_requires_uri() {
        assert!(resource().description("x").finish().is_none());
        let def = resource()
            .uri("file:///README.md")
            .description("README")
            .mime_type("text/markdown")
            .finish()
            .expect("uri set");
        let pr = def.to_protocol("readme");
        assert_eq!(pr.uri, "file:///README.md");
        assert_eq!(pr.name, "readme");
        assert_eq!(pr.mime_type.as_deref(), Some("text/markdown"));
    }

    #[test]
    fn template_builder_match_and_extract() {
        let def = resource_template()
            .uri_template("file:///{path}")
            .description("Any file")
            .finish()
            .expect("template");
        assert!(def.matches("file:///tmp"));
        assert!(!def.matches("file:///tmp/a"));
        let p = def.extract_params("file:///tmp").expect("params");
        assert_eq!(p.get("path").map(String::as_str), Some("tmp"));
        let pt = def.to_protocol("file");
        assert_eq!(pt.uri_template, "file:///{path}");
    }

    #[test]
    fn prompt_builder_args_and_messages() {
        let def = prompt()
            .description("Review")
            .argument(prompt_arg("language", Some("Lang".into()), Some(true)))
            .finish();
        assert_eq!(def.arguments.len(), 1);
        let pp = def.to_protocol("review");
        assert_eq!(pp.name, "review");
        let msgs = [user("hi"), assistant("yo")];
        let body = prompt_result_with_desc("d", &msgs);
        assert_eq!(body["description"], "d");
        assert_eq!(body["messages"].as_array().map(|a| a.len()), Some(2));
        let mut args = BTreeMap::new();
        args.insert("name".into(), "Ada".into());
        assert_eq!(interpolate_template("Hello {{name}}", &args), "Hello Ada");
        assert_eq!(interpolate_template("{{missing}}", &args), "{{missing}}");
    }

    #[test]
    fn content_helpers_and_normalize() {
        let t = text("hello");
        assert!(matches!(t, Content::Text { .. }));
        let img = image("AAAA", "image/png");
        assert!(matches!(img, Content::Image { .. }));
        let aud = audio("BBBB", "audio/wav");
        assert!(matches!(aud, Content::Audio { .. }));
        let err = tool_err("boom");
        assert_eq!(err["isError"], true);
        // single content item → wrap
        let single = json!({"type":"text","text":"x"});
        let norm = normalize_result(&single);
        assert!(norm["content"].is_array());
        // already full result
        let full = json!({"content":[{"type":"text","text":"x"}]});
        assert_eq!(normalize_result(&full)["content"][0]["text"], "x");
        let rt = resource_text("u://x", "body", Some("text/plain".into()));
        assert_eq!(rt["contents"][0]["uri"], "u://x");
        let rb = resource_blob_b64("u://b", "QQ==", "application/octet-stream");
        assert!(rb["contents"][0]["blob"].is_string());
    }

    #[test]
    fn inventory_maps_to_protocol() {
        let mut tools = BTreeMap::new();
        tools.insert("ping".into(), tool().description("p").finish());
        let list = tools_to_protocol(&tools);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "ping");

        let mut resources = BTreeMap::new();
        resources.insert(
            "r".into(),
            resource().uri("u://r").finish().expect("uri"),
        );
        assert_eq!(resources_to_protocol(&resources)[0].name, "r");

        let mut prompts = BTreeMap::new();
        prompts.insert("p".into(), prompt().description("d").finish());
        assert_eq!(prompts_to_protocol(&prompts)[0].name, "p");
    }

    #[test]
    fn wave4_product_builders_surface() {
        let jc = json_content(&json!({"a":1}));
        assert!(matches!(jc, Content::Text { .. }));
        let m = message("user", text("z"));
        assert_eq!(m.role, "user");
        assert_eq!(messages(&[m])["messages"].as_array().map(Vec::len), Some(1));
    }
}
