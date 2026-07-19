//! Pure MCP protocol helpers (parity with `src/protocol/mcp.ts` version surface).
//!
//! BW1 pure residual for `api/mcp-protocol` — types + version negotiation only.
//! Full handler dispatch stays TS until a later train.

use serde::{Deserialize, Serialize};

/// Latest protocol version (parity with TS `LATEST_PROTOCOL_VERSION`).
pub const LATEST_PROTOCOL_VERSION: &str = "2025-03-26";

/// Supported protocol versions, newest first.
pub const SUPPORTED_PROTOCOL_VERSIONS: &[&str] = &["2025-03-26", "2024-11-05"];

/// True when `version` is in the supported set.
#[must_use]
pub fn is_supported_protocol_version(version: &str) -> bool {
    SUPPORTED_PROTOCOL_VERSIONS.contains(&version)
}

/// Negotiate: pick client version if supported, else latest server version.
#[must_use]
pub fn negotiate_protocol_version(client_version: Option<&str>) -> &'static str {
    match client_version {
        Some(v) if is_supported_protocol_version(v) => {
            // Return the static entry matching client (same string content).
            SUPPORTED_PROTOCOL_VERSIONS
                .iter()
                .copied()
                .find(|s| *s == v)
                .unwrap_or(LATEST_PROTOCOL_VERSION)
        }
        _ => LATEST_PROTOCOL_VERSION,
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentAnnotations {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audience: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Content {
    Text {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        annotations: Option<ContentAnnotations>,
    },
    Image {
        data: String,
        #[serde(rename = "mimeType")]
        mime_type: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        annotations: Option<ContentAnnotations>,
    },
    Audio {
        data: String,
        #[serde(rename = "mimeType")]
        mime_type: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        annotations: Option<ContentAnnotations>,
    },
    Resource {
        resource: EmbeddedResource,
        #[serde(skip_serializing_if = "Option::is_none")]
        annotations: Option<ContentAnnotations>,
    },
}

/// Embedded resource payload (parity with MCP `EmbeddedResource` / resource contents).
///
/// Wire form may include `type: "resource"` (TS `resourceText` / `resourceBlob`).
/// Pure residual stores optional text and/or base64 blob — no I/O.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmbeddedResource {
    /// Present on wire contents items from builders (`type: "resource"`).
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    pub uri: String,
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Base64 blob body (parity with TS `resourceBlob` / EmbeddedResource.blob).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blob: Option<String>,
}

impl EmbeddedResource {
    /// Text contents item with `type: "resource"` (parity with builders `resourceText` item).
    #[must_use]
    pub fn text_item(
        uri: impl Into<String>,
        text: impl Into<String>,
        mime_type: Option<String>,
    ) -> Self {
        Self {
            kind: Some("resource".into()),
            uri: uri.into(),
            mime_type: mime_type.or_else(|| Some("text/plain".into())),
            text: Some(text.into()),
            blob: None,
        }
    }

    /// Blob contents item with `type: "resource"` (parity with builders `resourceBlob` item).
    #[must_use]
    pub fn blob_item(
        uri: impl Into<String>,
        blob_b64: impl Into<String>,
        mime_type: impl Into<String>,
    ) -> Self {
        Self {
            kind: Some("resource".into()),
            uri: uri.into(),
            mime_type: Some(mime_type.into()),
            text: None,
            blob: Some(blob_b64.into()),
        }
    }
}

impl Content {
    #[must_use]
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text {
            text: text.into(),
            annotations: None,
        }
    }

    #[must_use]
    pub fn is_text(&self) -> bool {
        matches!(self, Self::Text { .. })
    }

    #[must_use]
    pub fn is_image(&self) -> bool {
        matches!(self, Self::Image { .. })
    }

    #[must_use]
    pub fn is_audio(&self) -> bool {
        matches!(self, Self::Audio { .. })
    }

    #[must_use]
    pub fn is_resource(&self) -> bool {
        matches!(self, Self::Resource { .. })
    }

    #[must_use]
    pub fn image(data: impl Into<String>, mime_type: impl Into<String>) -> Self {
        Self::Image {
            data: data.into(),
            mime_type: mime_type.into(),
            annotations: None,
        }
    }

    #[must_use]
    pub fn audio(data: impl Into<String>, mime_type: impl Into<String>) -> Self {
        Self::Audio {
            data: data.into(),
            mime_type: mime_type.into(),
            annotations: None,
        }
    }

    #[must_use]
    pub fn with_annotations(mut self, annotations: ContentAnnotations) -> Self {
        match &mut self {
            Self::Text { annotations: a, .. }
            | Self::Image { annotations: a, .. }
            | Self::Audio { annotations: a, .. }
            | Self::Resource { annotations: a, .. } => *a = Some(annotations),
        }
        self
    }
}

/// Text content free function (parity with builders/tool.ts `text`).
#[must_use]
pub fn text_content(text: impl Into<String>) -> Content {
    Content::text(text)
}

/// Tool annotations hints (pure data).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ToolAnnotations {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_only_hint: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destructive_hint: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotent_hint: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_world_hint: Option<bool>,
}

/// Minimal tool descriptor for list/call envelopes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tool {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub input_schema: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<ToolAnnotations>,
}

/// Resource descriptor (parity with `src/protocol/mcp.ts` `Resource`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    pub uri: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
}

/// Resource template descriptor (parity with `ResourceTemplate`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceTemplate {
    pub uri_template: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

/// Prompt argument descriptor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptArgument {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
}

/// Prompt descriptor (parity with `src/protocol/mcp.ts` `Prompt`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Prompt {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<PromptArgument>>,
}

/// Root descriptor (parity with `Root`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Build a tools/list result envelope body.
#[must_use]
pub fn tools_list_result(tools: &[Tool], next_cursor: Option<String>) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert(
        "tools".into(),
        serde_json::to_value(tools).unwrap_or_else(|_| serde_json::json!([])),
    );
    if let Some(c) = next_cursor {
        map.insert("nextCursor".into(), serde_json::Value::String(c));
    }
    serde_json::Value::Object(map)
}

/// resources/list result envelope (cursor-aware).
#[must_use]
pub fn resources_list_result(
    resources: &[serde_json::Value],
    next_cursor: Option<String>,
) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert(
        "resources".into(),
        serde_json::Value::Array(resources.to_vec()),
    );
    if let Some(c) = next_cursor {
        map.insert("nextCursor".into(), serde_json::Value::String(c));
    }
    serde_json::Value::Object(map)
}

/// prompts/list result envelope (cursor-aware).
#[must_use]
pub fn prompts_list_result(
    prompts: &[serde_json::Value],
    next_cursor: Option<String>,
) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert("prompts".into(), serde_json::Value::Array(prompts.to_vec()));
    if let Some(c) = next_cursor {
        map.insert("nextCursor".into(), serde_json::Value::String(c));
    }
    serde_json::Value::Object(map)
}

/// Empty tools/call success (no content items).
#[must_use]
pub fn empty_tool_result() -> serde_json::Value {
    tool_content_result(&[], false)
}

/// Whether a tools/call result body marks `isError: true`.
#[must_use]
pub fn is_tool_error_result(body: &serde_json::Value) -> bool {
    body.get("isError")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

/// tools/call error result envelope (parity with `toolError`).
#[must_use]
pub fn tool_error(message: impl Into<String>) -> serde_json::Value {
    serde_json::json!({
        "content": [{ "type": "text", "text": message.into() }],
        "isError": true,
    })
}

/// tools/call success with single text content.
#[must_use]
pub fn tool_text_result(message: impl Into<String>, is_error: bool) -> serde_json::Value {
    serde_json::json!({
        "content": [{ "type": "text", "text": message.into() }],
        "isError": is_error,
    })
}

/// JSON pretty text content (parity with builders `json`).
#[must_use]
pub fn json_text_content(value: &serde_json::Value) -> Content {
    Content::text(serde_json::to_string_pretty(value).unwrap_or_else(|_| "{}".into()))
}

/// Notification constructors (parity with `src/notifications/helpers.ts`) — pure data only.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Notification {
    #[serde(rename = "progress")]
    Progress {
        #[serde(rename = "progressToken")]
        progress_token: serde_json::Value,
        progress: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        total: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },
    #[serde(rename = "log")]
    Log {
        level: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        logger: Option<String>,
        data: serde_json::Value,
    },
    #[serde(rename = "resources/list_changed")]
    ResourcesListChanged {},
    #[serde(rename = "tools/list_changed")]
    ToolsListChanged {},
    #[serde(rename = "prompts/list_changed")]
    PromptsListChanged {},
    #[serde(rename = "resource/updated")]
    ResourceUpdated { uri: String },
    #[serde(rename = "cancelled")]
    Cancelled {
        #[serde(rename = "requestId")]
        request_id: serde_json::Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
    },
    /// Client→server: roots list changed (parity with Method.RootsListChanged).
    #[serde(rename = "roots/list_changed")]
    RootsListChanged {},
}

#[must_use]
pub fn progress_notification(
    progress_token: serde_json::Value,
    current: f64,
    total: Option<f64>,
    message: Option<String>,
) -> Notification {
    Notification::Progress {
        progress_token,
        progress: current,
        total,
        message,
    }
}

#[must_use]
pub fn tools_list_changed() -> Notification {
    Notification::ToolsListChanged {}
}

#[must_use]
pub fn resources_list_changed() -> Notification {
    Notification::ResourcesListChanged {}
}

#[must_use]
pub fn prompts_list_changed() -> Notification {
    Notification::PromptsListChanged {}
}

#[must_use]
pub fn resource_updated(uri: impl Into<String>) -> Notification {
    Notification::ResourceUpdated { uri: uri.into() }
}

#[must_use]
pub fn cancelled(request_id: serde_json::Value, reason: Option<String>) -> Notification {
    Notification::Cancelled { request_id, reason }
}

/// Roots list changed notification (parity with Method.RootsListChanged).
#[must_use]
pub fn roots_list_changed() -> Notification {
    Notification::RootsListChanged {}
}

/// Log notification (parity with src/notifications/helpers.ts `log`).
#[must_use]
pub fn log_notification(
    level: impl Into<String>,
    data: serde_json::Value,
    logger: Option<String>,
) -> Notification {
    Notification::Log {
        level: level.into(),
        logger,
        data,
    }
}

// ============================================================================
// Log levels (parity with src/protocol/mcp.ts `LogLevel`)
// ============================================================================

/// MCP log levels, most-verbose first (spec order is not severity-sorted).
pub const LOG_LEVELS: &[&str] = &[
    "debug",
    "info",
    "notice",
    "warning",
    "error",
    "critical",
    "alert",
    "emergency",
];

/// True when `level` is a recognized MCP log level string.
#[must_use]
pub fn is_valid_log_level(level: &str) -> bool {
    LOG_LEVELS.contains(&level)
}

// ============================================================================
// Notification → JSON-RPC (parity with src/notifications/emitter.ts `toJsonRpc`)
// ============================================================================

/// Pure JSON-RPC wire form of a high-level [`Notification`].
///
/// Parity with TS `toJsonRpc` in `src/notifications/emitter.ts` — method name +
/// optional params object. No I/O; transports remain TS product authority.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JsonRpcNotificationWire {
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

/// Convert a pure notification shape to MCP JSON-RPC method + params.
#[must_use]
pub fn notification_to_jsonrpc(n: &Notification) -> JsonRpcNotificationWire {
    match n {
        Notification::Progress {
            progress_token,
            progress,
            total,
            message,
        } => {
            let mut map = serde_json::Map::new();
            map.insert("progressToken".into(), progress_token.clone());
            map.insert(
                "progress".into(),
                serde_json::Value::Number(
                    serde_json::Number::from_f64(*progress)
                        .unwrap_or_else(|| serde_json::Number::from(0)),
                ),
            );
            if let Some(t) = total {
                if let Some(num) = serde_json::Number::from_f64(*t) {
                    map.insert("total".into(), serde_json::Value::Number(num));
                }
            }
            if let Some(m) = message {
                map.insert("message".into(), serde_json::Value::String(m.clone()));
            }
            JsonRpcNotificationWire {
                method: methods::PROGRESS_NOTIFICATION.into(),
                params: Some(serde_json::Value::Object(map)),
            }
        }
        Notification::Log {
            level,
            logger,
            data,
        } => {
            let mut map = serde_json::Map::new();
            map.insert("level".into(), serde_json::Value::String(level.clone()));
            if let Some(l) = logger {
                map.insert("logger".into(), serde_json::Value::String(l.clone()));
            }
            map.insert("data".into(), data.clone());
            JsonRpcNotificationWire {
                method: methods::LOG_MESSAGE.into(),
                params: Some(serde_json::Value::Object(map)),
            }
        }
        Notification::ResourcesListChanged {} => JsonRpcNotificationWire {
            method: methods::RESOURCES_LIST_CHANGED.into(),
            params: None,
        },
        Notification::ToolsListChanged {} => JsonRpcNotificationWire {
            method: methods::TOOLS_LIST_CHANGED.into(),
            params: None,
        },
        Notification::PromptsListChanged {} => JsonRpcNotificationWire {
            method: methods::PROMPTS_LIST_CHANGED.into(),
            params: None,
        },
        Notification::ResourceUpdated { uri } => JsonRpcNotificationWire {
            method: methods::RESOURCES_UPDATED.into(),
            params: Some(serde_json::json!({ "uri": uri })),
        },
        Notification::Cancelled { request_id, reason } => {
            let mut map = serde_json::Map::new();
            map.insert("requestId".into(), request_id.clone());
            if let Some(r) = reason {
                map.insert("reason".into(), serde_json::Value::String(r.clone()));
            }
            JsonRpcNotificationWire {
                method: methods::CANCELLED_NOTIFICATION.into(),
                params: Some(serde_json::Value::Object(map)),
            }
        }
        Notification::RootsListChanged {} => JsonRpcNotificationWire {
            method: methods::ROOTS_LIST_CHANGED.into(),
            params: None,
        },
    }
}

// ============================================================================
// Tool result normalize (parity with src/builders/tool.ts `normalizeResult`)
// ============================================================================

/// Normalize a tools/call handler result into a `ToolsCallResult` body.
///
/// Accepts:
/// - full result object `{ "content": [...], "isError"?: bool, ... }`
/// - array of content items
/// - single content object
#[must_use]
pub fn normalize_tool_result(result: &serde_json::Value) -> serde_json::Value {
    if let Some(obj) = result.as_object() {
        if obj.get("content").and_then(|c| c.as_array()).is_some() {
            return result.clone();
        }
    }
    if result.is_array() {
        return serde_json::json!({ "content": result });
    }
    // Single content item (or unknown shape → wrap as sole content element).
    serde_json::json!({ "content": [result] })
}

// ============================================================================
// Initialize + remaining list envelopes (pure protocol builders)
// ============================================================================

/// `initialize` result body (parity with `InitializeResult` shape).
#[must_use]
pub fn initialize_result(
    protocol_version: impl Into<String>,
    server_name: impl Into<String>,
    server_version: impl Into<String>,
    capabilities: serde_json::Value,
    instructions: Option<String>,
) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert(
        "protocolVersion".into(),
        serde_json::Value::String(protocol_version.into()),
    );
    map.insert("capabilities".into(), capabilities);
    map.insert(
        "serverInfo".into(),
        serde_json::json!({
            "name": server_name.into(),
            "version": server_version.into(),
        }),
    );
    if let Some(i) = instructions {
        map.insert("instructions".into(), serde_json::Value::String(i));
    }
    serde_json::Value::Object(map)
}

/// Default empty server capabilities object.
#[must_use]
pub fn empty_server_capabilities() -> serde_json::Value {
    serde_json::json!({})
}

/// `resources/templates/list` result envelope.
#[must_use]
pub fn resource_templates_list_result(
    templates: &[serde_json::Value],
    next_cursor: Option<String>,
) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert(
        "resourceTemplates".into(),
        serde_json::Value::Array(templates.to_vec()),
    );
    if let Some(c) = next_cursor {
        map.insert("nextCursor".into(), serde_json::Value::String(c));
    }
    serde_json::Value::Object(map)
}

/// `roots/list` result envelope.
#[must_use]
pub fn roots_list_result(roots: &[serde_json::Value]) -> serde_json::Value {
    serde_json::json!({ "roots": roots })
}

/// `completion/complete` result envelope.
#[must_use]
pub fn completion_complete_result(
    values: &[String],
    total: Option<u64>,
    has_more: Option<bool>,
) -> serde_json::Value {
    let mut completion = serde_json::Map::new();
    completion.insert(
        "values".into(),
        serde_json::Value::Array(
            values
                .iter()
                .map(|s| serde_json::Value::String(s.clone()))
                .collect(),
        ),
    );
    if let Some(t) = total {
        completion.insert("total".into(), serde_json::Value::Number(t.into()));
    }
    if let Some(h) = has_more {
        completion.insert("hasMore".into(), serde_json::Value::Bool(h));
    }
    serde_json::json!({ "completion": completion })
}

/// Protocol tool descriptor from name + fields (parity with `toProtocolTool` data shape).
#[must_use]
pub fn protocol_tool(
    name: impl Into<String>,
    description: Option<String>,
    input_schema: serde_json::Value,
    annotations: Option<ToolAnnotations>,
) -> Tool {
    Tool {
        name: name.into(),
        title: None,
        description,
        input_schema,
        output_schema: None,
        annotations,
    }
}

/// Protocol resource descriptor (parity with `toProtocolResource`).
#[must_use]
pub fn protocol_resource(
    name: impl Into<String>,
    uri: impl Into<String>,
    description: Option<String>,
    mime_type: Option<String>,
) -> Resource {
    Resource {
        uri: uri.into(),
        name: name.into(),
        description,
        mime_type,
        size: None,
    }
}

/// Protocol resource template (parity with `toProtocolTemplate`).
#[must_use]
pub fn protocol_template(
    name: impl Into<String>,
    uri_template: impl Into<String>,
    description: Option<String>,
    mime_type: Option<String>,
) -> ResourceTemplate {
    ResourceTemplate {
        uri_template: uri_template.into(),
        name: name.into(),
        description,
        mime_type,
    }
}

/// Protocol prompt descriptor (parity with `toProtocolPrompt`).
#[must_use]
pub fn protocol_prompt(
    name: impl Into<String>,
    description: Option<String>,
    arguments: Option<Vec<PromptArgument>>,
) -> Prompt {
    Prompt {
        name: name.into(),
        title: None,
        description,
        arguments,
    }
}

/// Root entry (parity with `Root`).
#[must_use]
pub fn protocol_root(uri: impl Into<String>, name: Option<String>) -> Root {
    Root {
        uri: uri.into(),
        name,
    }
}

/// Embedded resource content (parity with builders `embedded`).
#[must_use]
pub fn embedded_resource_content(
    uri: impl Into<String>,
    mime_type: Option<String>,
    text: Option<String>,
) -> Content {
    Content::Resource {
        resource: EmbeddedResource {
            kind: None,
            uri: uri.into(),
            mime_type,
            text,
            blob: None,
        },
        annotations: None,
    }
}

/// resources/read result envelope from one or more embedded resources.
#[must_use]
pub fn resources_read_result(items: &[EmbeddedResource]) -> serde_json::Value {
    serde_json::json!({
        "contents": items,
    })
}

/// Multi-item resources/read (parity with builders `resourceContents`).
#[must_use]
pub fn resource_contents(items: &[EmbeddedResource]) -> serde_json::Value {
    resources_read_result(items)
}

/// Text resource contents item (parity with builders `resourceText` item shape).
#[must_use]
pub fn resource_text(
    uri: impl Into<String>,
    text: impl Into<String>,
    mime_type: Option<String>,
) -> EmbeddedResource {
    EmbeddedResource::text_item(uri, text, mime_type)
}

/// Full resources/read envelope for a single text resource (parity with `resourceText` return).
#[must_use]
pub fn resource_text_result(
    uri: impl Into<String>,
    text: impl Into<String>,
    mime_type: Option<String>,
) -> serde_json::Value {
    resources_read_result(&[resource_text(uri, text, mime_type)])
}

/// Blob resource contents envelope (parity with builders `resourceBlob`).
#[must_use]
pub fn resource_blob(
    uri: impl Into<String>,
    blob_b64: impl Into<String>,
    mime_type: impl Into<String>,
) -> serde_json::Value {
    resources_read_result(&[EmbeddedResource::blob_item(uri, blob_b64, mime_type)])
}

/// Empty `ping` result body.
#[must_use]
pub fn ping_result() -> serde_json::Value {
    serde_json::json!({})
}

/// tools/call result that already includes optional `structuredContent`.
#[must_use]
pub fn tool_result_with_structured(
    content: &[Content],
    is_error: bool,
    structured_content: Option<serde_json::Value>,
) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert(
        "content".into(),
        serde_json::to_value(content).unwrap_or_else(|_| serde_json::json!([])),
    );
    map.insert("isError".into(), serde_json::Value::Bool(is_error));
    if let Some(sc) = structured_content {
        map.insert("structuredContent".into(), sc);
    }
    serde_json::Value::Object(map)
}

/// Common server capabilities object (pure builder — no runtime enablement).
#[must_use]
pub fn server_capabilities(
    tools_list_changed: Option<bool>,
    resources_subscribe: Option<bool>,
    resources_list_changed: Option<bool>,
    prompts_list_changed: Option<bool>,
    logging: bool,
    completions: bool,
) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    if tools_list_changed.is_some() {
        let mut tools = serde_json::Map::new();
        if let Some(v) = tools_list_changed {
            tools.insert("listChanged".into(), serde_json::Value::Bool(v));
        }
        map.insert("tools".into(), serde_json::Value::Object(tools));
    }
    if resources_subscribe.is_some() || resources_list_changed.is_some() {
        let mut resources = serde_json::Map::new();
        if let Some(v) = resources_subscribe {
            resources.insert("subscribe".into(), serde_json::Value::Bool(v));
        }
        if let Some(v) = resources_list_changed {
            resources.insert("listChanged".into(), serde_json::Value::Bool(v));
        }
        map.insert("resources".into(), serde_json::Value::Object(resources));
    }
    if prompts_list_changed.is_some() {
        let mut prompts = serde_json::Map::new();
        if let Some(v) = prompts_list_changed {
            prompts.insert("listChanged".into(), serde_json::Value::Bool(v));
        }
        map.insert("prompts".into(), serde_json::Value::Object(prompts));
    }
    if logging {
        map.insert("logging".into(), serde_json::json!({}));
    }
    if completions {
        map.insert("completions".into(), serde_json::json!({}));
    }
    serde_json::Value::Object(map)
}

/// `sampling/createMessage` params envelope (pure data — no client RPC).
#[must_use]
pub fn sampling_create_params(
    messages: &[serde_json::Value],
    max_tokens: u64,
    system_prompt: Option<String>,
    temperature: Option<f64>,
) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert(
        "messages".into(),
        serde_json::Value::Array(messages.to_vec()),
    );
    map.insert(
        "maxTokens".into(),
        serde_json::Value::Number(max_tokens.into()),
    );
    if let Some(sp) = system_prompt {
        map.insert("systemPrompt".into(), serde_json::Value::String(sp));
    }
    if let Some(t) = temperature {
        if let Some(num) = serde_json::Number::from_f64(t) {
            map.insert("temperature".into(), serde_json::Value::Number(num));
        }
    }
    serde_json::Value::Object(map)
}

/// `sampling/createMessage` result envelope.
#[must_use]
pub fn sampling_create_result(
    role: impl Into<String>,
    content: Content,
    model: impl Into<String>,
    stop_reason: Option<String>,
) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert("role".into(), serde_json::Value::String(role.into()));
    map.insert(
        "content".into(),
        serde_json::to_value(content).unwrap_or_else(|_| serde_json::json!({})),
    );
    map.insert("model".into(), serde_json::Value::String(model.into()));
    if let Some(sr) = stop_reason {
        map.insert("stopReason".into(), serde_json::Value::String(sr));
    }
    serde_json::Value::Object(map)
}

/// `elicitation/create` params envelope (pure data — no client RPC).
#[must_use]
pub fn elicitation_create_params(
    message: impl Into<String>,
    requested_schema: serde_json::Value,
) -> serde_json::Value {
    serde_json::json!({
        "message": message.into(),
        "requestedSchema": requested_schema,
    })
}

/// `elicitation/create` result envelope (`action` + optional content).
#[must_use]
pub fn elicitation_create_result(
    action: impl Into<String>,
    content: Option<serde_json::Value>,
) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert("action".into(), serde_json::Value::String(action.into()));
    if let Some(c) = content {
        map.insert("content".into(), c);
    }
    serde_json::Value::Object(map)
}

/// Valid elicitation actions (parity with `ElicitationAction`).
pub const ELICITATION_ACTIONS: &[&str] = &["accept", "decline", "cancel"];

/// True when `action` is a recognized elicitation action.
#[must_use]
pub fn is_valid_elicitation_action(action: &str) -> bool {
    ELICITATION_ACTIONS.contains(&action)
}

/// tools/call multi-content result envelope.
#[must_use]
pub fn tool_content_result(content: &[Content], is_error: bool) -> serde_json::Value {
    serde_json::json!({
        "content": content,
        "isError": is_error,
    })
}

/// Create image content (base64) — parity: builders/tool.ts#image.
#[must_use]
pub fn image_content(data: impl Into<String>, mime_type: impl Into<String>) -> Content {
    Content::image(data, mime_type)
}

/// Create audio content (base64) — parity: builders/tool.ts#audio.
#[must_use]
pub fn audio_content(data: impl Into<String>, mime_type: impl Into<String>) -> Content {
    Content::audio(data, mime_type)
}

/// Prompt message with arbitrary content (parity: builders/prompt.ts#message).
#[must_use]
pub fn prompt_message(role: impl Into<String>, content: Content) -> PromptMessage {
    PromptMessage {
        role: role.into(),
        content,
    }
}

/// Pretty-printed JSON as text content (parity: builders/tool.ts#json).
#[must_use]
pub fn json_text_pretty(value: &serde_json::Value) -> Content {
    Content::text(serde_json::to_string_pretty(value).unwrap_or_else(|_| "null".into()))
}

// ============================================================================
// MCP method name constants (parity with src/protocol/mcp.ts Method)
// ============================================================================

pub mod methods {
    pub const INITIALIZE: &str = "initialize";
    pub const INITIALIZED: &str = "notifications/initialized";
    pub const PING: &str = "ping";
    pub const RESOURCES_LIST: &str = "resources/list";
    pub const RESOURCES_TEMPLATES_LIST: &str = "resources/templates/list";
    pub const RESOURCES_READ: &str = "resources/read";
    pub const RESOURCES_SUBSCRIBE: &str = "resources/subscribe";
    pub const RESOURCES_UNSUBSCRIBE: &str = "resources/unsubscribe";
    pub const RESOURCES_UPDATED: &str = "notifications/resources/updated";
    pub const RESOURCES_LIST_CHANGED: &str = "notifications/resources/list_changed";
    pub const PROMPTS_LIST: &str = "prompts/list";
    pub const PROMPTS_GET: &str = "prompts/get";
    pub const PROMPTS_LIST_CHANGED: &str = "notifications/prompts/list_changed";
    pub const TOOLS_LIST: &str = "tools/list";
    pub const TOOLS_CALL: &str = "tools/call";
    pub const TOOLS_LIST_CHANGED: &str = "notifications/tools/list_changed";
    pub const LOGGING_SET_LEVEL: &str = "logging/setLevel";
    pub const LOG_MESSAGE: &str = "notifications/message";
    pub const COMPLETION_COMPLETE: &str = "completion/complete";
    pub const SAMPLING_CREATE_MESSAGE: &str = "sampling/createMessage";
    pub const ELICITATION_CREATE: &str = "elicitation/create";
    pub const PROGRESS_NOTIFICATION: &str = "notifications/progress";
    pub const CANCELLED_NOTIFICATION: &str = "notifications/cancelled";
    pub const ROOTS_LIST: &str = "roots/list";
    pub const ROOTS_LIST_CHANGED: &str = "notifications/roots/list_changed";

    /// All known MCP method strings (parity with TS `Method` values).
    pub const ALL: &[&str] = &[
        INITIALIZE,
        INITIALIZED,
        PING,
        RESOURCES_LIST,
        RESOURCES_TEMPLATES_LIST,
        RESOURCES_READ,
        RESOURCES_SUBSCRIBE,
        RESOURCES_UNSUBSCRIBE,
        RESOURCES_UPDATED,
        RESOURCES_LIST_CHANGED,
        PROMPTS_LIST,
        PROMPTS_GET,
        PROMPTS_LIST_CHANGED,
        TOOLS_LIST,
        TOOLS_CALL,
        TOOLS_LIST_CHANGED,
        LOGGING_SET_LEVEL,
        LOG_MESSAGE,
        COMPLETION_COMPLETE,
        SAMPLING_CREATE_MESSAGE,
        ELICITATION_CREATE,
        PROGRESS_NOTIFICATION,
        CANCELLED_NOTIFICATION,
        ROOTS_LIST,
        ROOTS_LIST_CHANGED,
    ];
}

/// True when `method` is a known MCP method string (parity with TS `Method` catalog).
#[must_use]
pub fn is_mcp_method(method: &str) -> bool {
    methods::ALL.contains(&method)
}

/// True when `method` is a known MCP **notification** method (`notifications/*`).
#[must_use]
pub fn is_mcp_notification_method(method: &str) -> bool {
    is_mcp_method(method) && method.starts_with("notifications/")
}

/// True when `method` is a known MCP **request** method (not a notification).
#[must_use]
pub fn is_mcp_request_method(method: &str) -> bool {
    is_mcp_method(method) && !method.starts_with("notifications/")
}

// ============================================================================
// Request param envelopes (pure data — parity with src/protocol/mcp.ts params)
// ============================================================================

/// Implementation info (`name` + `version`) — parity with `Implementation`.
#[must_use]
pub fn implementation_info(
    name: impl Into<String>,
    version: impl Into<String>,
) -> serde_json::Value {
    serde_json::json!({
        "name": name.into(),
        "version": version.into(),
    })
}

/// Client capabilities object (pure builder — no runtime enablement).
#[must_use]
pub fn client_capabilities(
    roots_list_changed: Option<bool>,
    sampling: bool,
    elicitation: bool,
) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    if let Some(v) = roots_list_changed {
        map.insert("roots".into(), serde_json::json!({ "listChanged": v }));
    }
    if sampling {
        map.insert("sampling".into(), serde_json::json!({}));
    }
    if elicitation {
        map.insert("elicitation".into(), serde_json::json!({}));
    }
    serde_json::Value::Object(map)
}

/// `initialize` request params envelope (parity with `InitializeParams`).
#[must_use]
pub fn initialize_params(
    protocol_version: impl Into<String>,
    client_name: impl Into<String>,
    client_version: impl Into<String>,
    capabilities: serde_json::Value,
) -> serde_json::Value {
    serde_json::json!({
        "protocolVersion": protocol_version.into(),
        "capabilities": capabilities,
        "clientInfo": implementation_info(client_name, client_version),
    })
}

/// Cursor list params (`ListParams`).
#[must_use]
pub fn list_params(cursor: Option<String>) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    if let Some(c) = cursor {
        map.insert("cursor".into(), serde_json::Value::String(c));
    }
    serde_json::Value::Object(map)
}

/// `tools/call` params envelope.
#[must_use]
pub fn tools_call_params(
    name: impl Into<String>,
    arguments: Option<serde_json::Value>,
) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert("name".into(), serde_json::Value::String(name.into()));
    if let Some(args) = arguments {
        map.insert("arguments".into(), args);
    }
    serde_json::Value::Object(map)
}

/// `resources/read` params envelope.
#[must_use]
pub fn resources_read_params(uri: impl Into<String>) -> serde_json::Value {
    serde_json::json!({ "uri": uri.into() })
}

/// `resources/subscribe` params envelope.
#[must_use]
pub fn resources_subscribe_params(uri: impl Into<String>) -> serde_json::Value {
    serde_json::json!({ "uri": uri.into() })
}

/// `resources/unsubscribe` params envelope.
#[must_use]
pub fn resources_unsubscribe_params(uri: impl Into<String>) -> serde_json::Value {
    serde_json::json!({ "uri": uri.into() })
}

/// `prompts/get` params envelope.
#[must_use]
pub fn prompts_get_params(
    name: impl Into<String>,
    arguments: Option<std::collections::BTreeMap<String, String>>,
) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert("name".into(), serde_json::Value::String(name.into()));
    if let Some(args) = arguments {
        map.insert(
            "arguments".into(),
            serde_json::to_value(args).unwrap_or_else(|_| serde_json::json!({})),
        );
    }
    serde_json::Value::Object(map)
}

/// `logging/setLevel` params envelope.
#[must_use]
pub fn logging_set_level_params(level: impl Into<String>) -> serde_json::Value {
    serde_json::json!({ "level": level.into() })
}

/// `completion/complete` params envelope (pure data — no completion engine).
#[must_use]
pub fn completion_complete_params(
    ref_type: impl Into<String>,
    ref_name: Option<String>,
    ref_uri: Option<String>,
    argument_name: impl Into<String>,
    argument_value: impl Into<String>,
) -> serde_json::Value {
    let mut r = serde_json::Map::new();
    r.insert("type".into(), serde_json::Value::String(ref_type.into()));
    if let Some(n) = ref_name {
        r.insert("name".into(), serde_json::Value::String(n));
    }
    if let Some(u) = ref_uri {
        r.insert("uri".into(), serde_json::Value::String(u));
    }
    serde_json::json!({
        "ref": serde_json::Value::Object(r),
        "argument": {
            "name": argument_name.into(),
            "value": argument_value.into(),
        }
    })
}

/// Progress notification params envelope (parity with `ProgressParams`).
#[must_use]
pub fn progress_params(
    progress_token: serde_json::Value,
    progress: f64,
    total: Option<f64>,
    message: Option<String>,
) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert("progressToken".into(), progress_token);
    map.insert(
        "progress".into(),
        serde_json::Value::Number(
            serde_json::Number::from_f64(progress).unwrap_or_else(|| serde_json::Number::from(0)),
        ),
    );
    if let Some(t) = total {
        if let Some(num) = serde_json::Number::from_f64(t) {
            map.insert("total".into(), serde_json::Value::Number(num));
        }
    }
    if let Some(m) = message {
        map.insert("message".into(), serde_json::Value::String(m));
    }
    serde_json::Value::Object(map)
}

/// Sampling message helper (role + content) — pure data, no client RPC.
#[must_use]
pub fn sampling_message(role: impl Into<String>, content: Content) -> serde_json::Value {
    serde_json::json!({
        "role": role.into(),
        "content": content,
    })
}

// ============================================================================
// WAVE13 pure residual deepen (params meta, domains, sampling prefs, log entry)
// ============================================================================

/// `notifications/cancelled` params envelope (parity with `CancelledNotificationParams`).
#[must_use]
pub fn cancelled_params(
    request_id: serde_json::Value,
    reason: Option<String>,
) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert("requestId".into(), request_id);
    if let Some(r) = reason {
        map.insert("reason".into(), serde_json::Value::String(r));
    }
    serde_json::Value::Object(map)
}

/// `tools/call` params with optional `_meta.progressToken` (parity with ToolsCallParams._meta).
#[must_use]
pub fn tools_call_params_with_progress(
    name: impl Into<String>,
    arguments: Option<serde_json::Value>,
    progress_token: Option<serde_json::Value>,
) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert("name".into(), serde_json::Value::String(name.into()));
    if let Some(args) = arguments {
        map.insert("arguments".into(), args);
    }
    if let Some(tok) = progress_token {
        map.insert("_meta".into(), serde_json::json!({ "progressToken": tok }));
    }
    serde_json::Value::Object(map)
}

/// Model hint object (parity with `ModelHint`).
#[must_use]
pub fn model_hint(name: Option<String>) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    if let Some(n) = name {
        map.insert("name".into(), serde_json::Value::String(n));
    }
    serde_json::Value::Object(map)
}

/// Model preferences object (parity with `ModelPreferences`) — pure data only.
#[must_use]
pub fn model_preferences(
    cost_priority: Option<f64>,
    speed_priority: Option<f64>,
    intelligence_priority: Option<f64>,
    hints: Option<&[serde_json::Value]>,
) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    if let Some(h) = hints {
        map.insert("hints".into(), serde_json::Value::Array(h.to_vec()));
    }
    if let Some(c) = cost_priority {
        if let Some(num) = serde_json::Number::from_f64(c) {
            map.insert("costPriority".into(), serde_json::Value::Number(num));
        }
    }
    if let Some(s) = speed_priority {
        if let Some(num) = serde_json::Number::from_f64(s) {
            map.insert("speedPriority".into(), serde_json::Value::Number(num));
        }
    }
    if let Some(i) = intelligence_priority {
        if let Some(num) = serde_json::Number::from_f64(i) {
            map.insert(
                "intelligencePriority".into(),
                serde_json::Value::Number(num),
            );
        }
    }
    serde_json::Value::Object(map)
}

/// Sampling `includeContext` enum values (parity with SamplingCreateParams.includeContext).
pub const INCLUDE_CONTEXTS: &[&str] = &["none", "thisServer", "allServers"];

/// True when `ctx` is a recognized sampling includeContext value.
#[must_use]
pub fn is_valid_include_context(ctx: &str) -> bool {
    INCLUDE_CONTEXTS.contains(&ctx)
}

/// Canonical sampling stopReason values (spec examples; freeform strings also allowed on wire).
pub const CANONICAL_STOP_REASONS: &[&str] = &["endTurn", "stopSequence", "maxTokens"];

/// True when `reason` is a canonical sampling stopReason.
#[must_use]
pub fn is_canonical_stop_reason(reason: &str) -> bool {
    CANONICAL_STOP_REASONS.contains(&reason)
}

/// Content annotations builder (parity with `ContentAnnotations`).
#[must_use]
pub fn content_annotations(
    audience: Option<Vec<String>>,
    priority: Option<f64>,
) -> ContentAnnotations {
    ContentAnnotations { audience, priority }
}

/// Log entry object (parity with `LogEntry`) — pure data, no emit.
#[must_use]
pub fn log_entry(
    level: impl Into<String>,
    data: Option<serde_json::Value>,
    logger: Option<String>,
) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert("level".into(), serde_json::Value::String(level.into()));
    if let Some(l) = logger {
        map.insert("logger".into(), serde_json::Value::String(l));
    }
    if let Some(d) = data {
        map.insert("data".into(), d);
    }
    serde_json::Value::Object(map)
}

/// Empty client capabilities object.
#[must_use]
pub fn empty_client_capabilities() -> serde_json::Value {
    serde_json::json!({})
}

/// Resource content free function wrapping an [`EmbeddedResource`].
#[must_use]
pub fn resource_content(resource: EmbeddedResource) -> Content {
    Content::Resource {
        resource,
        annotations: None,
    }
}

/// Lifecycle MCP methods (`initialize`, `notifications/initialized`, `ping`).
pub const LIFECYCLE_METHODS: &[&str] = &[methods::INITIALIZE, methods::INITIALIZED, methods::PING];

/// True when `method` is a lifecycle method.
#[must_use]
pub fn is_lifecycle_method(method: &str) -> bool {
    LIFECYCLE_METHODS.contains(&method)
}

/// Coarse domain for a known MCP method string (pure residual classifier).
///
/// Returns one of: `lifecycle`, `resources`, `prompts`, `tools`, `logging`,
/// `completion`, `sampling`, `elicitation`, `progress`, `cancellation`,
/// `roots`, or `unknown`.
#[must_use]
pub fn method_domain(method: &str) -> &'static str {
    if !is_mcp_method(method) {
        return "unknown";
    }
    if is_lifecycle_method(method) {
        return "lifecycle";
    }
    if method.starts_with("resources/") || method.starts_with("notifications/resources/") {
        return "resources";
    }
    if method.starts_with("prompts/") || method.starts_with("notifications/prompts/") {
        return "prompts";
    }
    if method.starts_with("tools/") || method.starts_with("notifications/tools/") {
        return "tools";
    }
    if method == methods::LOGGING_SET_LEVEL || method == methods::LOG_MESSAGE {
        return "logging";
    }
    if method == methods::COMPLETION_COMPLETE {
        return "completion";
    }
    if method == methods::SAMPLING_CREATE_MESSAGE {
        return "sampling";
    }
    if method == methods::ELICITATION_CREATE {
        return "elicitation";
    }
    if method == methods::PROGRESS_NOTIFICATION {
        return "progress";
    }
    if method == methods::CANCELLED_NOTIFICATION {
        return "cancellation";
    }
    if method == methods::ROOTS_LIST || method == methods::ROOTS_LIST_CHANGED {
        return "roots";
    }
    "unknown"
}

/// Sampling create params with optional model preferences + includeContext (pure data).
#[must_use]
pub fn sampling_create_params_ext(
    messages: &[serde_json::Value],
    max_tokens: u64,
    system_prompt: Option<String>,
    temperature: Option<f64>,
    include_context: Option<String>,
    model_prefs: Option<serde_json::Value>,
) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert(
        "messages".into(),
        serde_json::Value::Array(messages.to_vec()),
    );
    map.insert(
        "maxTokens".into(),
        serde_json::Value::Number(max_tokens.into()),
    );
    if let Some(sp) = system_prompt {
        map.insert("systemPrompt".into(), serde_json::Value::String(sp));
    }
    if let Some(t) = temperature {
        if let Some(num) = serde_json::Number::from_f64(t) {
            map.insert("temperature".into(), serde_json::Value::Number(num));
        }
    }
    if let Some(ic) = include_context {
        map.insert("includeContext".into(), serde_json::Value::String(ic));
    }
    if let Some(mp) = model_prefs {
        map.insert("modelPreferences".into(), mp);
    }
    serde_json::Value::Object(map)
}

// ============================================================================
// WAVE14 pure residual catalogs + extractors
// ============================================================================

/// Prompt / sampling message roles (parity with PromptMessage.role / SamplingMessage.role).
pub const MESSAGE_ROLES: &[&str] = &["user", "assistant"];

/// True when `role` is a recognized message role.
#[must_use]
pub fn is_valid_message_role(role: &str) -> bool {
    MESSAGE_ROLES.contains(&role)
}

/// Content type discriminators (parity with Content union).
pub const CONTENT_TYPES: &[&str] = &["text", "image", "audio", "resource"];

/// True when `ty` is a recognized content type string.
#[must_use]
pub fn is_valid_content_type(ty: &str) -> bool {
    CONTENT_TYPES.contains(&ty)
}

/// ContentAnnotations.audience values.
pub const AUDIENCE_VALUES: &[&str] = &["user", "assistant"];

/// True when `audience` is a recognized content audience value.
#[must_use]
pub fn is_valid_audience(audience: &str) -> bool {
    AUDIENCE_VALUES.contains(&audience)
}

/// Server→client request methods (sampling + elicitation).
pub const SERVER_TO_CLIENT_METHODS: &[&str] = &[
    methods::SAMPLING_CREATE_MESSAGE,
    methods::ELICITATION_CREATE,
];

/// True when `method` is a server-initiated request toward the client.
#[must_use]
pub fn is_server_to_client_method(method: &str) -> bool {
    SERVER_TO_CLIENT_METHODS.contains(&method)
}

/// Completion reference types (parity with CompletionReference.type).
pub const COMPLETION_REF_TYPES: &[&str] = &["ref/prompt", "ref/resource"];

/// True when `ty` is a recognized completion ref type.
#[must_use]
pub fn is_valid_completion_ref_type(ty: &str) -> bool {
    COMPLETION_REF_TYPES.contains(&ty)
}

/// Elicitation property JSON Schema types (parity with ElicitationProperty.type).
pub const ELICITATION_PROPERTY_TYPES: &[&str] = &["string", "number", "integer", "boolean"];

/// True when `ty` is a recognized elicitation property type.
#[must_use]
pub fn is_valid_elicitation_property_type(ty: &str) -> bool {
    ELICITATION_PROPERTY_TYPES.contains(&ty)
}

/// Tool annotations free constructor (parity with `ToolAnnotations`).
#[must_use]
pub fn tool_annotations(
    title: Option<String>,
    read_only_hint: Option<bool>,
    destructive_hint: Option<bool>,
    idempotent_hint: Option<bool>,
    open_world_hint: Option<bool>,
) -> ToolAnnotations {
    ToolAnnotations {
        title,
        read_only_hint,
        destructive_hint,
        idempotent_hint,
        open_world_hint,
    }
}

/// Prompt argument free constructor (parity with `PromptArgument`).
#[must_use]
pub fn prompt_argument(
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

/// Build a completion `ref` object (parity with CompletionReference).
#[must_use]
pub fn completion_ref(
    ref_type: impl Into<String>,
    name: Option<String>,
    uri: Option<String>,
) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert("type".into(), serde_json::Value::String(ref_type.into()));
    if let Some(n) = name {
        map.insert("name".into(), serde_json::Value::String(n));
    }
    if let Some(u) = uri {
        map.insert("uri".into(), serde_json::Value::String(u));
    }
    serde_json::Value::Object(map)
}

/// Extract `_meta.progressToken` from a tools/call params object (pure residual).
///
/// Returns `None` when `_meta` or `progressToken` is absent.
#[must_use]
pub fn progress_token_from_call_params(params: &serde_json::Value) -> Option<&serde_json::Value> {
    params.get("_meta").and_then(|m| m.get("progressToken"))
}

/// Index of a log level in [`LOG_LEVELS`] (0 = most verbose). `None` if unknown.
#[must_use]
pub fn log_level_index(level: &str) -> Option<usize> {
    LOG_LEVELS.iter().position(|l| *l == level)
}

/// True when `level` is at least as severe as `minimum` (higher index in LOG_LEVELS).
///
/// Unknown levels return `false`.
#[must_use]
pub fn log_level_at_least(level: &str, minimum: &str) -> bool {
    match (log_level_index(level), log_level_index(minimum)) {
        (Some(l), Some(m)) => l >= m,
        _ => false,
    }
}

/// True when `method` is a `*/list_changed` notification method.
#[must_use]
pub fn is_list_changed_notification(method: &str) -> bool {
    matches!(
        method,
        methods::RESOURCES_LIST_CHANGED
            | methods::TOOLS_LIST_CHANGED
            | methods::PROMPTS_LIST_CHANGED
            | methods::ROOTS_LIST_CHANGED
    )
}

// ============================================================================
// WAVE15 pure residual catalogs + extractors
// ============================================================================

/// Client→server request methods (MCP request methods excluding server→client).
pub const CLIENT_TO_SERVER_REQUEST_METHODS: &[&str] = &[
    methods::INITIALIZE,
    methods::PING,
    methods::RESOURCES_LIST,
    methods::RESOURCES_TEMPLATES_LIST,
    methods::RESOURCES_READ,
    methods::RESOURCES_SUBSCRIBE,
    methods::RESOURCES_UNSUBSCRIBE,
    methods::PROMPTS_LIST,
    methods::PROMPTS_GET,
    methods::TOOLS_LIST,
    methods::TOOLS_CALL,
    methods::LOGGING_SET_LEVEL,
    methods::COMPLETION_COMPLETE,
    methods::ROOTS_LIST,
];

/// True when `method` is a client-initiated MCP request (not server→client, not notification).
#[must_use]
pub fn is_client_to_server_request_method(method: &str) -> bool {
    CLIENT_TO_SERVER_REQUEST_METHODS.contains(&method)
}

/// Known MCP notification method strings (parity with `notifications/*` in Method catalog).
pub const NOTIFICATION_METHODS: &[&str] = &[
    methods::INITIALIZED,
    methods::RESOURCES_UPDATED,
    methods::RESOURCES_LIST_CHANGED,
    methods::PROMPTS_LIST_CHANGED,
    methods::TOOLS_LIST_CHANGED,
    methods::LOG_MESSAGE,
    methods::PROGRESS_NOTIFICATION,
    methods::CANCELLED_NOTIFICATION,
    methods::ROOTS_LIST_CHANGED,
];

/// Message direction for a known MCP method string (pure residual classifier).
///
/// Returns one of: `client_to_server`, `server_to_client`, `notification`, or `unknown`.
#[must_use]
pub fn method_direction(method: &str) -> &'static str {
    if is_server_to_client_method(method) {
        return "server_to_client";
    }
    if is_mcp_notification_method(method) {
        return "notification";
    }
    if is_client_to_server_request_method(method) {
        return "client_to_server";
    }
    "unknown"
}

/// Extract `progressToken` from a bare `_meta` object (pure residual).
#[must_use]
pub fn progress_token_from_meta(meta: &serde_json::Value) -> Option<&serde_json::Value> {
    meta.get("progressToken")
}

/// Extract `nextCursor` from a list-result body (tools/resources/prompts list shapes).
#[must_use]
pub fn list_next_cursor(result: &serde_json::Value) -> Option<&str> {
    result.get("nextCursor").and_then(|v| v.as_str())
}

/// True when a list-result body has a non-empty `nextCursor` string.
#[must_use]
pub fn list_has_next_cursor(result: &serde_json::Value) -> bool {
    list_next_cursor(result).is_some_and(|c| !c.is_empty())
}

/// ContentAnnotations.priority is defined on `[0.0, 1.0]` (MCP spec).
#[must_use]
pub fn is_valid_priority(priority: f64) -> bool {
    (0.0..=1.0).contains(&priority) && priority.is_finite()
}

/// Clamp a priority into `[0.0, 1.0]`. Non-finite values become `0.0`.
#[must_use]
pub fn clamp_priority(priority: f64) -> f64 {
    if !priority.is_finite() {
        return 0.0;
    }
    priority.clamp(0.0, 1.0)
}

/// Completion argument free constructor (parity with `CompletionArgument`).
#[must_use]
pub fn completion_argument(name: impl Into<String>, value: impl Into<String>) -> serde_json::Value {
    serde_json::json!({
        "name": name.into(),
        "value": value.into(),
    })
}

/// First text body from a tools/call result `content` array (pure residual extractor).
///
/// Returns `None` when `content` is missing/empty or the first item is not text.
#[must_use]
pub fn tool_result_first_text(result: &serde_json::Value) -> Option<&str> {
    result
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|item| {
            if item.get("type").and_then(|t| t.as_str()) == Some("text") {
                item.get("text").and_then(|t| t.as_str())
            } else {
                None
            }
        })
}

/// True when a roots URI uses the `file://` scheme (MCP roots convention).
#[must_use]
pub fn is_file_uri(uri: &str) -> bool {
    uri.starts_with("file://")
}

// ============================================================================
// WAVE16 pure residual catalogs + extractors
// ============================================================================

/// Resource-domain MCP methods (requests + notifications).
pub const RESOURCES_METHODS: &[&str] = &[
    methods::RESOURCES_LIST,
    methods::RESOURCES_TEMPLATES_LIST,
    methods::RESOURCES_READ,
    methods::RESOURCES_SUBSCRIBE,
    methods::RESOURCES_UNSUBSCRIBE,
    methods::RESOURCES_UPDATED,
    methods::RESOURCES_LIST_CHANGED,
];

/// Tools-domain MCP methods (requests + notifications).
pub const TOOLS_METHODS: &[&str] = &[
    methods::TOOLS_LIST,
    methods::TOOLS_CALL,
    methods::TOOLS_LIST_CHANGED,
];

/// Prompts-domain MCP methods (requests + notifications).
pub const PROMPTS_METHODS: &[&str] = &[
    methods::PROMPTS_LIST,
    methods::PROMPTS_GET,
    methods::PROMPTS_LIST_CHANGED,
];

/// True when `method` is a resources-domain MCP method.
#[must_use]
pub fn is_resources_method(method: &str) -> bool {
    RESOURCES_METHODS.contains(&method)
}

/// True when `method` is a tools-domain MCP method.
#[must_use]
pub fn is_tools_method(method: &str) -> bool {
    TOOLS_METHODS.contains(&method)
}

/// True when `method` is a prompts-domain MCP method.
#[must_use]
pub fn is_prompts_method(method: &str) -> bool {
    PROMPTS_METHODS.contains(&method)
}

/// Content type discriminator string (`text` | `image` | `audio` | `resource`).
#[must_use]
pub fn content_type_str(content: &Content) -> &'static str {
    match content {
        Content::Text { .. } => "text",
        Content::Image { .. } => "image",
        Content::Audio { .. } => "audio",
        Content::Resource { .. } => "resource",
    }
}

/// Text body from a [`Content::Text`] item; `None` for other variants.
#[must_use]
pub fn content_text_body(content: &Content) -> Option<&str> {
    match content {
        Content::Text { text, .. } => Some(text.as_str()),
        _ => None,
    }
}

/// Number of items in a tools/call result `content` array (`0` when missing).
#[must_use]
pub fn tool_result_content_len(result: &serde_json::Value) -> usize {
    result
        .get("content")
        .and_then(|c| c.as_array())
        .map(|a| a.len())
        .unwrap_or(0)
}

/// Concatenate all `type: "text"` content bodies from a tools/call result.
///
/// Returns `None` when no text items are present. Bodies are joined with no separator
/// (parity with a pure residual text extraction; product formatting stays TS).
#[must_use]
pub fn tool_result_all_text(result: &serde_json::Value) -> Option<String> {
    let arr = result.get("content").and_then(|c| c.as_array())?;
    let mut out = String::new();
    let mut any = false;
    for item in arr {
        if item.get("type").and_then(|t| t.as_str()) == Some("text") {
            if let Some(t) = item.get("text").and_then(|t| t.as_str()) {
                out.push_str(t);
                any = true;
            }
        }
    }
    if any {
        Some(out)
    } else {
        None
    }
}

/// Extract `name` from a `tools/call` params object.
#[must_use]
pub fn tools_call_name(params: &serde_json::Value) -> Option<&str> {
    params.get("name").and_then(|v| v.as_str())
}

/// Extract `uri` from resource-oriented params (`resources/read|subscribe|unsubscribe`).
#[must_use]
pub fn params_uri(params: &serde_json::Value) -> Option<&str> {
    params.get("uri").and_then(|v| v.as_str())
}

/// Extract a list-result array by key (`tools` | `resources` | `prompts` | …).
#[must_use]
pub fn list_result_array<'a>(
    result: &'a serde_json::Value,
    key: &str,
) -> Option<&'a Vec<serde_json::Value>> {
    result.get(key).and_then(|v| v.as_array())
}

/// Length of a list-result array by key (`0` when missing or non-array).
#[must_use]
pub fn list_result_len(result: &serde_json::Value, key: &str) -> usize {
    list_result_array(result, key).map(|a| a.len()).unwrap_or(0)
}

/// URI scheme before the first `:` (e.g. `file`, `https`); `None` when absent.
#[must_use]
pub fn uri_scheme(uri: &str) -> Option<&str> {
    let idx = uri.find(':')?;
    let scheme = &uri[..idx];
    if scheme.is_empty() {
        return None;
    }
    // RFC 3986 scheme: ALPHA *( ALPHA / DIGIT / "+" / "-" / "." )
    if !scheme.chars().enumerate().all(|(i, c)| {
        if i == 0 {
            c.is_ascii_alphabetic()
        } else {
            c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.'
        }
    }) {
        return None;
    }
    Some(scheme)
}

/// True when the URI scheme is `http` or `https` (case-sensitive per MCP examples).
#[must_use]
pub fn is_http_or_https_uri(uri: &str) -> bool {
    matches!(uri_scheme(uri), Some("http") | Some("https"))
}

/// Build a `_meta` object carrying `progressToken` (pure residual).
#[must_use]
pub fn meta_with_progress_token(progress_token: serde_json::Value) -> serde_json::Value {
    serde_json::json!({ "progressToken": progress_token })
}

/// Extract `protocolVersion` from an `initialize` result body.
#[must_use]
pub fn initialize_result_protocol_version(result: &serde_json::Value) -> Option<&str> {
    result.get("protocolVersion").and_then(|v| v.as_str())
}

/// Extract `serverInfo.name` from an `initialize` result body.
#[must_use]
pub fn initialize_result_server_name(result: &serde_json::Value) -> Option<&str> {
    result
        .get("serverInfo")
        .and_then(|s| s.get("name"))
        .and_then(|v| v.as_str())
}

/// Check if a URI matches a template pattern (`{param}` → single path segment).
#[must_use]
pub fn matches_template(template: &str, uri: &str) -> bool {
    extract_params(template, uri).is_some()
}

/// Extract `{param}` path segments from a URI against a template.
/// Returns None when the URI does not match.
#[must_use]
pub fn extract_params(
    template: &str,
    uri: &str,
) -> Option<std::collections::BTreeMap<String, String>> {
    // Split template into literal/param alternating segments.
    let mut segments: Vec<(bool, String)> = Vec::new(); // (is_param, text)
    let mut rest = template;
    while let Some(start) = rest.find('{') {
        let lit = &rest[..start];
        if !lit.is_empty() {
            segments.push((false, lit.to_string()));
        }
        let after = &rest[start + 1..];
        let end = after.find('}')?;
        let name = &after[..end];
        if name.is_empty() || name.contains('/') {
            return None;
        }
        segments.push((true, name.to_string()));
        rest = &after[end + 1..];
    }
    if !rest.is_empty() {
        segments.push((false, rest.to_string()));
    }

    let mut params = std::collections::BTreeMap::new();
    let mut cursor = 0usize;
    for (i, (is_param, text)) in segments.iter().enumerate() {
        if !is_param {
            let lit = text.as_str();
            if !uri[cursor..].starts_with(lit) {
                return None;
            }
            cursor += lit.len();
        } else {
            // param consumes until next literal or end; must be single segment (no /)
            let next_lit = segments
                .get(i + 1)
                .and_then(|(ip, t)| if !ip { Some(t.as_str()) } else { None });
            let end = if let Some(lit) = next_lit {
                let relative = uri[cursor..].find(lit)?;
                cursor + relative
            } else {
                uri.len()
            };
            if end < cursor {
                return None;
            }
            let value = &uri[cursor..end];
            if value.is_empty() || value.contains('/') {
                return None;
            }
            params.insert(text.clone(), value.to_string());
            cursor = end;
        }
    }
    if cursor != uri.len() {
        return None;
    }
    Some(params)
}

/// Interpolate `{{key}}` placeholders (prompt template helper).
/// Missing keys remain as `{{key}}`.
#[must_use]
pub fn interpolate(template: &str, args: &std::collections::BTreeMap<String, String>) -> String {
    let mut out = String::with_capacity(template.len());
    let mut i = 0;
    let chars: Vec<char> = template.chars().collect();
    while i < chars.len() {
        if i + 1 < chars.len() && chars[i] == '{' && chars[i + 1] == '{' {
            // find closing }}
            let mut j = i + 2;
            while j + 1 < chars.len() {
                if chars[j] == '}' && chars[j + 1] == '}' {
                    break;
                }
                j += 1;
            }
            if j + 1 < chars.len() && chars[j] == '}' && chars[j + 1] == '}' {
                let key: String = chars[i + 2..j].iter().collect();
                if !key.is_empty() && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                    if let Some(val) = args.get(&key) {
                        out.push_str(val);
                    } else {
                        out.push_str("{{");
                        out.push_str(&key);
                        out.push_str("}}");
                    }
                    i = j + 2;
                    continue;
                }
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// Prompt message helpers (pure builders — no IO).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptMessage {
    pub role: String,
    pub content: Content,
}

#[must_use]
pub fn user_message(text: impl Into<String>) -> PromptMessage {
    PromptMessage {
        role: "user".into(),
        content: Content::text(text),
    }
}

#[must_use]
pub fn assistant_message(text: impl Into<String>) -> PromptMessage {
    PromptMessage {
        role: "assistant".into(),
        content: Content::text(text),
    }
}

#[must_use]
pub fn prompt_messages(msgs: &[PromptMessage]) -> serde_json::Value {
    serde_json::json!({ "messages": msgs })
}

#[must_use]
pub fn prompt_result(description: impl Into<String>, msgs: &[PromptMessage]) -> serde_json::Value {
    serde_json::json!({
        "description": description.into(),
        "messages": msgs,
    })
}

// --- WAVE17 pure residual deepen (no product transport cutover) ---

/// Extract prompt description from a `prompts/get` result envelope.
#[must_use]
pub fn prompt_result_description(result: &serde_json::Value) -> Option<&str> {
    result.get("description").and_then(|v| v.as_str())
}

/// Count messages in a prompt result.
#[must_use]
pub fn prompt_result_message_count(result: &serde_json::Value) -> usize {
    result
        .get("messages")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0)
}

/// First message role in a prompt result (if any).
#[must_use]
pub fn prompt_result_first_role(result: &serde_json::Value) -> Option<&str> {
    result
        .get("messages")
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .and_then(|m| m.get("role"))
        .and_then(|v| v.as_str())
}

/// Normalize role strings (trim + lowercase); unknown → None.
/// Complements existing `is_valid_message_role` (exact match only).
#[must_use]
pub fn normalize_message_role(role: &str) -> Option<&'static str> {
    match role.trim().to_ascii_lowercase().as_str() {
        "user" => Some("user"),
        "assistant" => Some("assistant"),
        _ => None,
    }
}

/// Extract tool name list from a tools/list result.
#[must_use]
pub fn tool_names_from_list(result: &serde_json::Value) -> Vec<String> {
    result
        .get("tools")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|t| t.get("name").and_then(|n| n.as_str()).map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

/// Whether structuredContent is present on a tool result.
#[must_use]
pub fn tool_result_has_structured(result: &serde_json::Value) -> bool {
    result
        .get("structuredContent")
        .is_some_and(|v| !v.is_null())
}

/// Extract resource URIs from a resources/list result.
#[must_use]
pub fn resource_uris_from_list(result: &serde_json::Value) -> Vec<String> {
    result
        .get("resources")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|r| r.get("uri").and_then(|u| u.as_str()).map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

/// Extract log level from logging/setLevel params.
#[must_use]
pub fn logging_level_from_params(params: &serde_json::Value) -> Option<&str> {
    params.get("level").and_then(|v| v.as_str())
}

/// Whether a content array is non-empty tool payload.
#[must_use]
pub fn content_array_nonempty(result: &serde_json::Value) -> bool {
    result
        .get("content")
        .and_then(|v| v.as_array())
        .is_some_and(|a| !a.is_empty())
}

/// Join template params back into a URI (inverse of extract_params for simple templates).
#[must_use]
pub fn apply_template_params(
    template: &str,
    params: &std::collections::BTreeMap<String, String>,
) -> Option<String> {
    let mut out = String::new();
    let mut rest = template;
    while let Some(start) = rest.find('{') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        let end = after.find('}')?;
        let name = &after[..end];
        let value = params.get(name)?;
        if value.is_empty() || value.contains('/') {
            return None;
        }
        out.push_str(value);
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    Some(out)
}

// --- WAVE18 pure residual deepen (no product transport cutover) ---

/// Extract prompt names from a `prompts/list` result.
#[must_use]
pub fn prompt_names_from_list(result: &serde_json::Value) -> Vec<String> {
    result
        .get("prompts")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|p| p.get("name").and_then(|n| n.as_str()).map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

/// Extract resource names from a `resources/list` result.
#[must_use]
pub fn resource_names_from_list(result: &serde_json::Value) -> Vec<String> {
    result
        .get("resources")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|r| r.get("name").and_then(|n| n.as_str()).map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

/// Count tools in a tools/list result.
#[must_use]
pub fn tool_count_from_list(result: &serde_json::Value) -> usize {
    result
        .get("tools")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0)
}

/// Normalize protocol version string (trim); empty → None.
#[must_use]
pub fn normalize_protocol_version(version: &str) -> Option<&str> {
    let v = version.trim();
    if v.is_empty() {
        None
    } else {
        Some(v)
    }
}

/// True when method is `ping` (case-sensitive product method).
#[must_use]
pub fn is_ping_method(method: &str) -> bool {
    method == "ping"
}

/// Extract content `type` field from a content object.
#[must_use]
pub fn content_type_of(content: &serde_json::Value) -> Option<&str> {
    content.get("type").and_then(|v| v.as_str())
}

/// First content type from a tool result content array.
#[must_use]
pub fn tool_result_first_content_type(result: &serde_json::Value) -> Option<&str> {
    result
        .get("content")
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .and_then(content_type_of)
}

/// Whether tool result is marked as error (`isError: true`).
#[must_use]
pub fn tool_result_is_error(result: &serde_json::Value) -> bool {
    result
        .get("isError")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

/// Extract progress token from progress notification params.
#[must_use]
pub fn progress_token_from_params(params: &serde_json::Value) -> Option<serde_json::Value> {
    params.get("progressToken").cloned()
}

/// Extract progress fraction (0..1-ish) from progress params.
#[must_use]
pub fn progress_value_from_params(params: &serde_json::Value) -> Option<f64> {
    params.get("progress").and_then(|v| v.as_f64())
}

// --- WAVE19 pure residual ---

/// Extract `mimeType` from a protocol resource wire object.
#[must_use]
pub fn resource_mime_type(resource: &serde_json::Value) -> Option<&str> {
    resource
        .get("mimeType")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
}

/// Extract optional tool title from a Tool wire object / struct JSON.
#[must_use]
pub fn tool_title_of(tool: &serde_json::Value) -> Option<&str> {
    tool.get("title")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
}

/// Collect prompt argument names from a prompt wire object (order preserved).
#[must_use]
pub fn prompt_argument_names(prompt: &serde_json::Value) -> Vec<String> {
    prompt
        .get("arguments")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|a| a.get("name").and_then(|n| n.as_str()).map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

/// True when a Content wire object carries non-empty annotations.
#[must_use]
pub fn content_has_annotations(content: &serde_json::Value) -> bool {
    match content.get("annotations") {
        Some(a) if a.is_object() => !a.as_object().map(|o| o.is_empty()).unwrap_or(true),
        _ => false,
    }
}

/// Server name from initialize result envelope.
#[must_use]
pub fn initialize_server_name(result: &serde_json::Value) -> Option<&str> {
    result
        .get("serverInfo")
        .and_then(|s| s.get("name"))
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
}

/// Server version from initialize result envelope.
#[must_use]
pub fn initialize_server_version(result: &serde_json::Value) -> Option<&str> {
    result
        .get("serverInfo")
        .and_then(|s| s.get("version"))
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
}

/// Count of content blocks on a tool result (0 if missing/non-array).
#[must_use]
pub fn tool_result_content_count(result: &serde_json::Value) -> usize {
    result
        .get("content")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0)
}

/// True when method is logging-related (`logging/*` or `notifications/message`).
#[must_use]
pub fn is_logging_method(method: &str) -> bool {
    method == methods::LOGGING_SET_LEVEL
        || method == methods::LOG_MESSAGE
        || method.starts_with("logging/")
}

// --- WAVE20 pure residual ---

/// Extract optional `description` from a resource wire object.
#[must_use]
pub fn resource_description(resource: &serde_json::Value) -> Option<&str> {
    resource
        .get("description")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
}

/// Extract optional `description` from a tool wire object.
#[must_use]
pub fn tool_description(tool: &serde_json::Value) -> Option<&str> {
    tool.get("description")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
}

/// Extract `uriTemplate` from a resource template wire object.
#[must_use]
pub fn resource_template_uri_template(template: &serde_json::Value) -> Option<&str> {
    template
        .get("uriTemplate")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
}

/// Audience values from content annotations (`annotations.audience` string or array).
#[must_use]
pub fn content_annotation_audience(content: &serde_json::Value) -> Vec<String> {
    let Some(ann) = content.get("annotations") else {
        return Vec::new();
    };
    match ann.get("audience") {
        Some(serde_json::Value::String(s)) if !s.is_empty() => vec![s.clone()],
        Some(serde_json::Value::Array(arr)) => arr
            .iter()
            .filter_map(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect(),
        _ => Vec::new(),
    }
}

/// Priority from content annotations (`annotations.priority` number).
#[must_use]
pub fn content_annotation_priority(content: &serde_json::Value) -> Option<f64> {
    content
        .get("annotations")
        .and_then(|a| a.get("priority"))
        .and_then(|v| v.as_f64())
}

/// Optional `instructions` string from initialize result.
#[must_use]
pub fn initialize_instructions(result: &serde_json::Value) -> Option<&str> {
    result
        .get("instructions")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
}

/// True when method is cancellation-related (`notifications/cancelled`).
#[must_use]
pub fn is_cancellation_method(method: &str) -> bool {
    method == methods::CANCELLED_NOTIFICATION || method == "notifications/cancelled"
}

/// Collect root URIs from a roots/list result.
#[must_use]
pub fn root_uris_from_list(result: &serde_json::Value) -> Vec<String> {
    result
        .get("roots")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|r| r.get("uri").and_then(|u| u.as_str()).map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

/// Tool name from tools/call params.
#[must_use]
pub fn tool_name_from_call_params(params: &serde_json::Value) -> Option<&str> {
    params
        .get("name")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
}

// --- WAVE21 pure residual (handler dual-oracle kernels) ---

/// Unknown-tool error result (parity with `handler.ts` handleToolsCall missing tool).
#[must_use]
pub fn unknown_tool_error_result(name: &str) -> serde_json::Value {
    tool_error(format!("Unknown tool: {name}"))
}

/// Tool handler exception result (parity with `handler.ts` catch path).
#[must_use]
pub fn tool_handler_error_result(error: impl std::fmt::Display) -> serde_json::Value {
    tool_error(format!("Tool error: {error}"))
}

/// Unknown method error message string (parity with `handler.ts` default switch).
#[must_use]
pub fn unknown_method_error_message(method: &str) -> String {
    format!("Unknown method: {method}")
}

/// True when client capabilities advertise sampling (object present).
#[must_use]
pub fn client_has_sampling(capabilities: &serde_json::Value) -> bool {
    capabilities.get("sampling").is_some_and(|v| !v.is_null())
}

/// True when client capabilities advertise elicitation (object present).
#[must_use]
pub fn client_has_elicitation(capabilities: &serde_json::Value) -> bool {
    capabilities
        .get("elicitation")
        .is_some_and(|v| !v.is_null())
}

/// Count of resources in a resources/list result body.
#[must_use]
pub fn resource_count_from_list(result: &serde_json::Value) -> usize {
    list_result_len(result, "resources")
}

/// Count of prompts in a prompts/list result body.
#[must_use]
pub fn prompt_count_from_list(result: &serde_json::Value) -> usize {
    list_result_len(result, "prompts")
}

/// True when initialize result carries non-empty instructions.
#[must_use]
pub fn initialize_has_instructions(result: &serde_json::Value) -> bool {
    result
        .get("instructions")
        .and_then(|v| v.as_str())
        .is_some_and(|s| !s.is_empty())
}

/// Progress notification params wire object (parity with handler progress helper).
#[must_use]
pub fn progress_notification_params(
    progress_token: serde_json::Value,
    progress: f64,
    total: Option<f64>,
    message: Option<String>,
) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert("progressToken".into(), progress_token);
    map.insert("progress".into(), serde_json::json!(progress));
    if let Some(t) = total {
        map.insert("total".into(), serde_json::json!(t));
    }
    if let Some(m) = message {
        map.insert("message".into(), serde_json::Value::String(m));
    }
    serde_json::Value::Object(map)
}

/// Log notification params wire object (parity with handler log helper / notifications/message).
#[must_use]
pub fn log_notification_params(
    level: impl Into<String>,
    data: serde_json::Value,
    logger: Option<String>,
) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert("level".into(), serde_json::Value::String(level.into()));
    map.insert("data".into(), data);
    if let Some(l) = logger {
        map.insert("logger".into(), serde_json::Value::String(l));
    }
    serde_json::Value::Object(map)
}

/// Methods accepted by the TS `handleRequest` switch (server product handler surface).
#[must_use]
pub fn is_handler_request_method(method: &str) -> bool {
    matches!(
        method,
        methods::INITIALIZE
            | methods::PING
            | methods::TOOLS_LIST
            | methods::TOOLS_CALL
            | methods::RESOURCES_LIST
            | methods::RESOURCES_TEMPLATES_LIST
            | methods::RESOURCES_READ
            | methods::RESOURCES_SUBSCRIBE
            | methods::RESOURCES_UNSUBSCRIBE
            | methods::PROMPTS_LIST
            | methods::PROMPTS_GET
            | methods::LOGGING_SET_LEVEL
            | methods::COMPLETION_COMPLETE
    )
}

/// First content text from a tool error result when `isError` is true.
#[must_use]
pub fn tool_error_message(result: &serde_json::Value) -> Option<&str> {
    if !is_tool_error_result(result) {
        return None;
    }
    tool_result_first_text(result)
}

/// Empty success body used by ping / subscribe / unsubscribe handlers.
#[must_use]
pub fn empty_object_result() -> serde_json::Value {
    serde_json::json!({})
}

/// True when a tools/list result is empty (no tools array or empty array).
#[must_use]
pub fn tools_list_is_empty(result: &serde_json::Value) -> bool {
    tool_count_from_list(result) == 0
}

// --- WAVE22 pure residual (handler/list/call dual-oracle kernels) ---

/// Tool arguments object from tools/call params (missing/non-object → None).
#[must_use]
pub fn tool_arguments_from_call_params(params: &serde_json::Value) -> Option<&serde_json::Value> {
    params.get("arguments").filter(|v| v.is_object())
}

/// True when tools/call params carry a progressToken in `_meta`.
#[must_use]
pub fn tools_call_has_progress_token(params: &serde_json::Value) -> bool {
    progress_token_from_params(params).is_some()
        || params
            .get("_meta")
            .and_then(|m| m.get("progressToken"))
            .is_some()
}

/// URI from resources/read|subscribe|unsubscribe params.
#[must_use]
pub fn resource_uri_from_params(params: &serde_json::Value) -> Option<&str> {
    params
        .get("uri")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
}

/// Prompt name from prompts/get params.
#[must_use]
pub fn prompt_name_from_get_params(params: &serde_json::Value) -> Option<&str> {
    params
        .get("name")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
}

/// Count of resource templates in templates/list result.
#[must_use]
pub fn resource_template_count_from_list(result: &serde_json::Value) -> usize {
    list_result_len(result, "resourceTemplates")
}

/// True when resources/list is empty.
#[must_use]
pub fn resources_list_is_empty(result: &serde_json::Value) -> bool {
    resource_count_from_list(result) == 0
}

/// True when prompts/list is empty.
#[must_use]
pub fn prompts_list_is_empty(result: &serde_json::Value) -> bool {
    prompt_count_from_list(result) == 0
}

/// True when resource templates list is empty.
#[must_use]
pub fn resource_templates_list_is_empty(result: &serde_json::Value) -> bool {
    resource_template_count_from_list(result) == 0
}

/// Log level from logging/setLevel params.
#[must_use]
pub fn log_level_from_set_params(params: &serde_json::Value) -> Option<&str> {
    params.get("level").and_then(|v| v.as_str())
}

/// Client capability flags summary for initialize negotiation dual-oracle.
#[must_use]
pub fn client_capability_flags(capabilities: &serde_json::Value) -> (bool, bool, bool) {
    let roots = capabilities.get("roots").is_some_and(|v| !v.is_null());
    (
        roots,
        client_has_sampling(capabilities),
        client_has_elicitation(capabilities),
    )
}

/// True when a method is a server→client request (sampling/elicitation).
#[must_use]
pub fn is_server_to_client_request_method(method: &str) -> bool {
    matches!(
        method,
        methods::SAMPLING_CREATE_MESSAGE | methods::ELICITATION_CREATE
    )
}

/// Content text builder for a simple tool success result.
#[must_use]
pub fn tool_text_success_result(text: impl Into<String>) -> serde_json::Value {
    serde_json::json!({
        "content": [{ "type": "text", "text": text.into() }],
        "isError": false
    })
}

/// Cursor from list params (None when absent/empty).
#[must_use]
pub fn cursor_from_list_params(params: &serde_json::Value) -> Option<&str> {
    params
        .get("cursor")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

// ── WAVE23 pure residual: handler subscribe/completion/dispatch dual-oracle ──

/// Whether the tool context may expose a sampling client.
/// Dual-oracle of handler `clientCapabilities?.sampling && requestFn`.
#[must_use]
pub fn tool_context_sampling_available(
    client_has_sampling_cap: bool,
    request_fn_present: bool,
) -> bool {
    client_has_sampling_cap && request_fn_present
}

/// Whether the tool context may expose an elicit helper.
/// Dual-oracle of handler `clientCapabilities?.elicitation && requestFn`.
#[must_use]
pub fn tool_context_elicitation_available(
    client_has_elicitation_cap: bool,
    request_fn_present: bool,
) -> bool {
    client_has_elicitation_cap && request_fn_present
}

/// URI from resources/subscribe|unsubscribe params.
#[must_use]
pub fn uri_from_subscribe_params(params: &serde_json::Value) -> Option<&str> {
    params
        .get("uri")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

/// Pure subscription set add — returns whether the set grew.
/// Dual-oracle of `handleResourcesSubscribe` (`subscriptions.add`).
#[must_use]
pub fn subscription_add(existing: &[String], uri: &str) -> (Vec<String>, bool) {
    if existing.iter().any(|u| u == uri) {
        return (existing.to_vec(), false);
    }
    let mut next = existing.to_vec();
    next.push(uri.to_string());
    (next, true)
}

/// Pure subscription set remove — returns whether the set shrank.
/// Dual-oracle of `handleResourcesUnsubscribe` (`subscriptions.delete`).
#[must_use]
pub fn subscription_remove(existing: &[String], uri: &str) -> (Vec<String>, bool) {
    let before = existing.len();
    let next: Vec<String> = existing
        .iter()
        .filter(|u| u.as_str() != uri)
        .cloned()
        .collect();
    let removed = next.len() < before;
    (next, removed)
}

/// True when `uri` contains the completion argument value (resource ref filter).
/// Dual-oracle of `resource.uri.includes(params.argument.value)`.
#[must_use]
pub fn resource_uri_matches_completion(uri: &str, argument_value: &str) -> bool {
    uri.contains(argument_value)
}

/// Filter resource URIs for completion/complete ref/resource branch.
#[must_use]
pub fn completion_values_from_resource_uris(uris: &[&str], argument_value: &str) -> Vec<String> {
    uris.iter()
        .filter(|u| resource_uri_matches_completion(u, argument_value))
        .map(|u| (*u).to_string())
        .collect()
}

/// Basic completion result body — dual-oracle of handler
/// `{ completion: { values, hasMore: false } }` (no total when unset).
#[must_use]
pub fn basic_completion_result(values: &[String]) -> serde_json::Value {
    completion_complete_result(values, None, Some(false))
}

/// Parse completion ref type from params (`ref.type`).
#[must_use]
pub fn completion_ref_type(params: &serde_json::Value) -> Option<&str> {
    params
        .pointer("/ref/type")
        .and_then(|v| v.as_str())
        .filter(|t| is_valid_completion_ref_type(t))
}

/// Completion argument value from params.
#[must_use]
pub fn completion_argument_value(params: &serde_json::Value) -> Option<&str> {
    params.pointer("/argument/value").and_then(|v| v.as_str())
}

/// Prompt name from completion ref when `ref/prompt`.
#[must_use]
pub fn completion_ref_prompt_name(params: &serde_json::Value) -> Option<&str> {
    if completion_ref_type(params) != Some("ref/prompt") {
        return None;
    }
    params.pointer("/ref/name").and_then(|v| v.as_str())
}

/// Dispatch classification for an inbound JSON-RPC message kind label.
/// Dual-oracle of `dispatch` notification vs request branching (no I/O).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchMessageKind {
    Notification,
    Request,
    Other,
}

/// Classify message kind for handler dispatch.
#[must_use]
pub fn dispatch_message_kind(is_notification: bool, is_request: bool) -> DispatchMessageKind {
    if is_notification {
        DispatchMessageKind::Notification
    } else if is_request {
        DispatchMessageKind::Request
    } else {
        DispatchMessageKind::Other
    }
}

/// True when a notification method is `notifications/initialized`.
#[must_use]
pub fn is_initialized_notification_method(method: &str) -> bool {
    method == methods::INITIALIZED
}

/// Error message text for handler catch dual-oracle
/// (`error instanceof Error ? error.message : String(error)`).
#[must_use]
pub fn handler_error_message(is_error_instance: bool, message: &str, display: &str) -> String {
    if is_error_instance {
        message.to_string()
    } else {
        display.to_string()
    }
}

/// Progress token extraction for tool context (`params._meta?.progressToken`).
/// Dual-oracle of handler `params._meta?.progressToken` (not top-level progressToken).
#[must_use]
pub fn tool_context_progress_token(params: &serde_json::Value) -> Option<serde_json::Value> {
    params
        .pointer("/_meta/progressToken")
        .filter(|v| !v.is_null())
        .cloned()
}

/// Whether logging/setLevel should accept the level (valid LogLevel enum).
/// Dual-oracle of handler assignment (TS assigns without validate; pure residual
/// fail-closed helper for product path dens).
#[must_use]
pub fn logging_set_level_accepts(level: &str) -> bool {
    is_valid_log_level(level)
}

/// Dual-oracle of `throw new Error(\`Unknown prompt: ${params.name}\`)`.
#[must_use]
pub fn unknown_prompt_error_message(name: &str) -> String {
    format!("Unknown prompt: {name}")
}

/// Dual-oracle of unknown resource error message shape.
#[must_use]
pub fn unknown_resource_error_message(uri: &str) -> String {
    format!("Unknown resource: {uri}")
}

/// Dual-oracle of `params.arguments ?? {}` for prompts/get.
#[must_use]
pub fn prompt_arguments_from_get_params(params: &serde_json::Value) -> serde_json::Value {
    params
        .get("arguments")
        .filter(|v| v.is_object())
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}))
}

/// Dual-oracle of completion ref type branch (`ref/prompt`).
#[must_use]
pub fn completion_ref_is_prompt(params: &serde_json::Value) -> bool {
    completion_ref_type(params) == Some("ref/prompt")
}

/// Dual-oracle of completion ref type branch (`ref/resource`).
#[must_use]
pub fn completion_ref_is_resource(params: &serde_json::Value) -> bool {
    completion_ref_type(params) == Some("ref/resource")
}

/// Dual-oracle of handler notification switch: initialized vs ignore-other.
#[must_use]
pub fn notification_should_mark_initialized(method: &str) -> bool {
    is_initialized_notification_method(method)
}

/// Dual-oracle of `dispatch` returning `{ type: "none" }` for notification/other.
#[must_use]
pub fn dispatch_returns_none(is_notification: bool, is_request: bool) -> bool {
    !matches!(
        dispatch_message_kind(is_notification, is_request),
        DispatchMessageKind::Request
    )
}

/// Dual-oracle of success response path (request + handler ok).
#[must_use]
pub fn dispatch_success_result_present(is_request: bool, handler_ok: bool) -> bool {
    is_request && handler_ok
}

/// Dual-oracle of logging/setLevel body after level assign.
#[must_use]
pub fn logging_set_level_result() -> serde_json::Value {
    empty_object_result()
}

/// Dual-oracle of resources/subscribe|unsubscribe result body.
#[must_use]
pub fn subscription_mutation_result() -> serde_json::Value {
    empty_object_result()
}

/// Dual-oracle of tools/call `params.arguments ?? {}` coalesce.
#[must_use]
pub fn tool_arguments_or_empty(params: &serde_json::Value) -> serde_json::Value {
    tool_arguments_from_call_params(params)
        .filter(|v| v.is_object())
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}))
}

/// Dual-oracle of handleRequest switch membership for core methods.
#[must_use]
pub fn is_core_handler_method(method: &str) -> bool {
    matches!(
        method,
        methods::INITIALIZE
            | methods::PING
            | methods::TOOLS_LIST
            | methods::TOOLS_CALL
            | methods::RESOURCES_LIST
            | methods::RESOURCES_TEMPLATES_LIST
            | methods::RESOURCES_READ
            | methods::RESOURCES_SUBSCRIBE
            | methods::RESOURCES_UNSUBSCRIBE
            | methods::PROMPTS_LIST
            | methods::PROMPTS_GET
            | methods::LOGGING_SET_LEVEL
            | methods::COMPLETION_COMPLETE
    )
}

// ── WAVE25 pure residual dens: progress/roots/sampling kernels ──────────────
// Dual-oracle of notifications/helpers + roots list + sampling param gates.
// NO authority_rust / package_main_rust / prod authority invent.

/// Dual-oracle of progress notification params shape checks.
#[must_use]
pub fn progress_params_valid(progress: f64, total: Option<f64>) -> bool {
    if !progress.is_finite() || progress < 0.0 {
        return false;
    }
    if let Some(t) = total {
        if !t.is_finite() || t < 0.0 || progress > t {
            return false;
        }
    }
    true
}

/// Dual-oracle of progress fraction (None when total missing/zero).
#[must_use]
pub fn progress_fraction(progress: f64, total: Option<f64>) -> Option<f64> {
    let t = total.filter(|v| v.is_finite() && *v > 0.0)?;
    if !progress.is_finite() || progress < 0.0 {
        return None;
    }
    Some((progress / t).clamp(0.0, 1.0))
}

/// Dual-oracle of roots/list empty result body.
#[must_use]
pub fn roots_list_is_empty(result: &serde_json::Value) -> bool {
    result
        .get("roots")
        .and_then(|v| v.as_array())
        .map(|a| a.is_empty())
        .unwrap_or(true)
}

/// Count roots entries.
#[must_use]
pub fn roots_list_count(result: &serde_json::Value) -> usize {
    result
        .get("roots")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0)
}

/// Extract root URI at index.
#[must_use]
pub fn root_uri_at(result: &serde_json::Value, index: usize) -> Option<&str> {
    result
        .get("roots")
        .and_then(|v| v.as_array())
        .and_then(|a| a.get(index))
        .and_then(|r| r.get("uri"))
        .and_then(|u| u.as_str())
}

/// Sampling maxTokens gate (must be positive).
#[must_use]
pub fn sampling_max_tokens_valid(max_tokens: i64) -> bool {
    max_tokens > 0
}

/// Dual-oracle of sampling temperature clamp policy (None = omit).
#[must_use]
pub fn sampling_temperature_ok(temp: Option<f64>) -> bool {
    match temp {
        None => true,
        Some(t) => t.is_finite() && (0.0..=2.0).contains(&t),
    }
}

/// Whether sampling createMessage may run (client caps + request fn).
#[must_use]
pub fn sampling_client_available(client_declared: bool, request_fn_present: bool) -> bool {
    client_declared && request_fn_present
}

/// Whether elicitation create may run.
#[must_use]
pub fn elicitation_client_available(client_declared: bool, request_fn_present: bool) -> bool {
    client_declared && request_fn_present
}

// ── WAVE26 pure residual dens: cancellation / ping / logging level kernels ──

/// Dual-oracle: cancellation notification method constant residual.
pub const NOTIFICATION_CANCELLED: &str = "notifications/cancelled";

/// Dual-oracle: ping method residual.
pub const METHOD_PING: &str = "ping";

/// Canonical logging levels (MCP spec residual closed set).
pub const LOGGING_LEVELS: &[&str] = &[
    "debug",
    "info",
    "notice",
    "warning",
    "error",
    "critical",
    "alert",
    "emergency",
];

/// True when level is a canonical logging level (case-sensitive dual-oracle of wire).
#[must_use]
pub fn logging_level_valid(level: &str) -> bool {
    LOGGING_LEVELS.contains(&level)
}

/// Pure cancellation params envelope dual-oracle (`requestId` required residual).
#[must_use]
pub fn cancellation_params(
    request_id: serde_json::Value,
    reason: Option<&str>,
) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert("requestId".into(), request_id);
    if let Some(r) = reason.filter(|s| !s.is_empty()) {
        map.insert("reason".into(), serde_json::Value::String(r.to_string()));
    }
    serde_json::Value::Object(map)
}

/// Pure ping result is empty object dual-oracle (wave26 residual alias inventory).
#[must_use]
pub fn wave26_ping_result_empty() -> serde_json::Value {
    serde_json::json!({})
}

/// Dual-oracle: logging/setLevel params gate — level must be canonical.
#[must_use]
pub fn logging_set_level_params_valid(params: &serde_json::Value) -> bool {
    params
        .get("level")
        .and_then(|v| v.as_str())
        .is_some_and(logging_level_valid)
}

/// Dual-oracle: progress token presence on call params residual deepen.
#[must_use]
pub fn call_params_has_progress_token(params: &serde_json::Value) -> bool {
    progress_token_from_call_params(params).is_some()
        || params
            .get("_meta")
            .and_then(|m| progress_token_from_meta(m))
            .is_some()
}

/// Dual-oracle: empty roots list result residual.
#[must_use]
pub fn empty_roots_list_result() -> serde_json::Value {
    serde_json::json!({"roots": []})
}

// ── WAVE27 pure residual dens: resource templates / elicitation / empty result kernels ──

/// Dual-oracle resource templates list method residual.
pub const METHOD_RESOURCES_TEMPLATES_LIST: &str = "resources/templates/list";

/// Dual-oracle elicitation create method residual (client capability).
pub const METHOD_ELICITATION_CREATE: &str = "elicitation/create";

/// Dual-oracle: empty resource templates list result residual.
#[must_use]
pub fn empty_resource_templates_list_result() -> serde_json::Value {
    serde_json::json!({"resourceTemplates": []})
}

/// Dual-oracle: resource template uriTemplate presence gate.
#[must_use]
pub fn resource_template_uri_present(template: &serde_json::Value) -> bool {
    template
        .get("uriTemplate")
        .and_then(|v| v.as_str())
        .is_some_and(|s| !s.is_empty())
}

/// Dual-oracle: count resource templates in list result.
#[must_use]
pub fn resource_templates_count(result: &serde_json::Value) -> usize {
    result
        .get("resourceTemplates")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0)
}

/// Dual-oracle: elicitation params require message residual.
#[must_use]
pub fn elicitation_params_valid(params: &serde_json::Value) -> bool {
    params
        .get("message")
        .and_then(|v| v.as_str())
        .is_some_and(|s| !s.is_empty())
}

/// Dual-oracle empty object result residual alias inventory.
#[must_use]
pub fn wave27_empty_object_result() -> serde_json::Value {
    serde_json::json!({})
}

/// Dual-oracle: initialize result has serverInfo residual gate.
#[must_use]
pub fn initialize_result_has_server_info(result: &serde_json::Value) -> bool {
    result
        .get("serverInfo")
        .and_then(|v| v.as_object())
        .is_some_and(|o| o.contains_key("name"))
}

// ── WAVE28 pure residual dens: tools/list empty + prompts/list empty + schema gate ──

/// Dual-oracle tools list method residual.
pub const METHOD_TOOLS_LIST: &str = "tools/list";
/// Dual-oracle prompts list method residual.
pub const METHOD_PROMPTS_LIST: &str = "prompts/list";
/// Dual-oracle resources list method residual.
pub const METHOD_RESOURCES_LIST: &str = "resources/list";

/// Dual-oracle: empty tools list result residual.
#[must_use]
pub fn empty_tools_list_result() -> serde_json::Value {
    serde_json::json!({"tools": []})
}

/// Dual-oracle: empty prompts list result residual.
#[must_use]
pub fn empty_prompts_list_result() -> serde_json::Value {
    serde_json::json!({"prompts": []})
}

/// Dual-oracle: empty resources list result residual.
#[must_use]
pub fn empty_resources_list_result() -> serde_json::Value {
    serde_json::json!({"resources": []})
}

/// Dual-oracle: count tools in tools/list result.
#[must_use]
pub fn tools_count_from_list(result: &serde_json::Value) -> usize {
    result
        .get("tools")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0)
}

/// Dual-oracle: tool descriptor has required name residual.
#[must_use]
pub fn tool_descriptor_name_present(tool: &serde_json::Value) -> bool {
    tool.get("name")
        .and_then(|v| v.as_str())
        .is_some_and(|s| !s.is_empty())
}

/// Dual-oracle: prompt descriptor has required name residual.
#[must_use]
pub fn prompt_descriptor_name_present(prompt: &serde_json::Value) -> bool {
    prompt
        .get("name")
        .and_then(|v| v.as_str())
        .is_some_and(|s| !s.is_empty())
}

/// Dual-oracle: inputSchema object residual gate for tools.
#[must_use]
pub fn tool_input_schema_is_object(tool: &serde_json::Value) -> bool {
    tool.get("inputSchema")
        .and_then(|v| v.as_object())
        .is_some()
}

/// Dual-oracle empty array residual alias inventory.
#[must_use]
pub fn wave28_empty_array() -> serde_json::Value {
    serde_json::json!([])
}

// ── WAVE29 pure residual dens: tools/call + prompts/get + resources/read gate kernels ──

/// Dual-oracle tools/call method residual.
pub const METHOD_TOOLS_CALL: &str = "tools/call";
/// Dual-oracle prompts/get method residual.
pub const METHOD_PROMPTS_GET: &str = "prompts/get";
/// Dual-oracle resources/read method residual.
pub const METHOD_RESOURCES_READ: &str = "resources/read";

/// Dual-oracle: tools/call params require non-empty name residual.
#[must_use]
pub fn tools_call_params_valid(params: &serde_json::Value) -> bool {
    params
        .get("name")
        .and_then(|v| v.as_str())
        .is_some_and(|s| !s.is_empty())
}

/// Dual-oracle: prompts/get params require non-empty name residual.
#[must_use]
pub fn prompts_get_params_valid(params: &serde_json::Value) -> bool {
    params
        .get("name")
        .and_then(|v| v.as_str())
        .is_some_and(|s| !s.is_empty())
}

/// Dual-oracle: resources/read params require non-empty uri residual.
#[must_use]
pub fn resources_read_params_valid(params: &serde_json::Value) -> bool {
    params
        .get("uri")
        .and_then(|v| v.as_str())
        .is_some_and(|s| !s.is_empty())
}

/// Dual-oracle residual honesty: empty content array for tool success.
#[must_use]
pub fn empty_tool_content() -> serde_json::Value {
    serde_json::json!([])
}

/// Dual-oracle residual: wave29 empty object alias inventory.
#[must_use]
pub fn wave29_empty_object() -> serde_json::Value {
    serde_json::json!({})
}

// ── WAVE30 pure residual dens: notifications/* + roots/list gate kernels ──

/// Dual-oracle notifications/initialized method residual.
pub const METHOD_NOTIFICATIONS_INITIALIZED: &str = "notifications/initialized";
/// Dual-oracle notifications/cancelled method residual.
pub const METHOD_NOTIFICATIONS_CANCELLED: &str = "notifications/cancelled";
/// Dual-oracle notifications/progress method residual.
pub const METHOD_NOTIFICATIONS_PROGRESS: &str = "notifications/progress";
/// Dual-oracle roots/list method residual.
pub const METHOD_ROOTS_LIST: &str = "roots/list";

/// Dual-oracle residual: notification methods do not expect a result payload.
#[must_use]
pub fn is_notification_method(method: &str) -> bool {
    method.starts_with("notifications/")
}

/// Dual-oracle residual: roots/list params may be empty object or absent.
#[must_use]
pub fn roots_list_params_valid(params: Option<&serde_json::Value>) -> bool {
    match params {
        None => true,
        Some(v) if v.is_null() => true,
        Some(v) if v.is_object() => true,
        _ => false,
    }
}

/// Dual-oracle residual: progress notification requires progressToken.
#[must_use]
pub fn notifications_progress_params_valid(params: &serde_json::Value) -> bool {
    params.get("progressToken").is_some()
}

/// Dual-oracle residual: wave30 empty notification ack alias.
/// (empty roots list result lives as existing `empty_roots_list_result` dual-oracle.)
#[must_use]
pub fn wave30_notification_ack() -> serde_json::Value {
    serde_json::json!({})
}
