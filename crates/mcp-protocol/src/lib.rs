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
    map.insert(
        "prompts".into(),
        serde_json::Value::Array(prompts.to_vec()),
    );
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
    body.get("isError").and_then(|v| v.as_bool()).unwrap_or(false)
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
    Notification::Cancelled {
        request_id,
        reason,
    }
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
        Notification::Cancelled {
            request_id,
            reason,
        } => {
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
    map.insert("maxTokens".into(), serde_json::Value::Number(max_tokens.into()));
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
        map.insert(
            "roots".into(),
            serde_json::json!({ "listChanged": v }),
        );
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
        map.insert(
            "_meta".into(),
            serde_json::json!({ "progressToken": tok }),
        );
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
            map.insert("intelligencePriority".into(), serde_json::Value::Number(num));
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
pub const LIFECYCLE_METHODS: &[&str] = &[
    methods::INITIALIZE,
    methods::INITIALIZED,
    methods::PING,
];

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
    map.insert("maxTokens".into(), serde_json::Value::Number(max_tokens.into()));
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
pub const SERVER_TO_CLIENT_METHODS: &[&str] =
    &[methods::SAMPLING_CREATE_MESSAGE, methods::ELICITATION_CREATE];

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
    params
        .get("_meta")
        .and_then(|m| m.get("progressToken"))
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
pub fn completion_argument(
    name: impl Into<String>,
    value: impl Into<String>,
) -> serde_json::Value {
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
    if !scheme
        .chars()
        .enumerate()
        .all(|(i, c)| {
            if i == 0 {
                c.is_ascii_alphabetic()
            } else {
                c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.'
            }
        })
    {
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
            let next_lit = segments.get(i + 1).and_then(|(ip, t)| if !ip { Some(t.as_str()) } else { None });
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
                if !key.is_empty()
                    && key
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '_')
                {
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
    resource.get("mimeType").and_then(|v| v.as_str()).filter(|s| !s.is_empty())
}

/// Extract optional tool title from a Tool wire object / struct JSON.
#[must_use]
pub fn tool_title_of(tool: &serde_json::Value) -> Option<&str> {
    tool.get("title").and_then(|v| v.as_str()).filter(|s| !s.is_empty())
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
    capabilities
        .get("sampling")
        .is_some_and(|v| !v.is_null())
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
    let roots = capabilities
        .get("roots")
        .is_some_and(|v| !v.is_null());
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
    let next: Vec<String> = existing.iter().filter(|u| u.as_str() != uri).cloned().collect();
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
pub fn completion_values_from_resource_uris(
    uris: &[&str],
    argument_value: &str,
) -> Vec<String> {
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
    "debug", "info", "notice", "warning", "error", "critical", "alert", "emergency",
];

/// True when level is a canonical logging level (case-sensitive dual-oracle of wire).
#[must_use]
pub fn logging_level_valid(level: &str) -> bool {
    LOGGING_LEVELS.contains(&level)
}

/// Pure cancellation params envelope dual-oracle (`requestId` required residual).
#[must_use]
pub fn cancellation_params(request_id: serde_json::Value, reason: Option<&str>) -> serde_json::Value {
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



// ── WAVE39 pure residual dens: elicitation + resource templates method residual ──
// Dual-oracle residual of elicitation/create + resources/templates/list pure halves.
// Transport/handler I/O residual retained. dens ≠ flip. package_main_rust not claimed.

/// Dual-oracle residual: elicitation create method.
pub const WAVE39_METHOD_ELICITATION_CREATE: &str = "elicitation/create";
/// Dual-oracle residual: resources templates list method.
pub const WAVE39_METHOD_RESOURCES_TEMPLATES_LIST: &str = "resources/templates/list";

/// Dual-oracle residual: wave39 method catalog.
pub const WAVE39_METHODS: &[&str] = &[
    WAVE39_METHOD_ELICITATION_CREATE,
    WAVE39_METHOD_RESOURCES_TEMPLATES_LIST,
];

/// Dual-oracle residual: is wave39 densed method.
#[must_use]
pub fn is_wave39_method(method: &str) -> bool {
    WAVE39_METHODS.contains(&method)
}

/// Dual-oracle residual: elicitation create params require non-empty message string pure half.
#[must_use]
pub fn wave39_elicitation_create_params_valid(params: &serde_json::Value) -> bool {
    params
        .get("message")
        .and_then(|v| v.as_str())
        .is_some_and(|s| !s.trim().is_empty())
}

/// Dual-oracle residual: resources templates list accepts empty/object params.
#[must_use]
pub fn wave39_resources_templates_list_params_ok(params: &serde_json::Value) -> bool {
    params.is_object() || params.is_null()
}

/// Dual-oracle residual: empty object alias inventory.
#[must_use]
pub fn wave39_empty_object() -> serde_json::Value {
    serde_json::json!({})
}

/// Dual-oracle residual: method group for wave39 methods.
#[must_use]
pub fn wave39_method_group(method: &str) -> Option<&'static str> {
    match method {
        WAVE39_METHOD_ELICITATION_CREATE => Some("elicitation"),
        WAVE39_METHOD_RESOURCES_TEMPLATES_LIST => Some("resources"),
        _ => None,
    }
}

// ── WAVE40 pure residual dens: logging/setLevel + completion/complete method dual-oracle residual ──
// Dual-oracle residual of METHOD_LOGGING_SET_LEVEL / METHOD_COMPLETION_COMPLETE pure halves.
// Transport/handler I/O residual retained. dens ≠ flip. package_main_rust not claimed.

/// Dual-oracle residual: WAVE40 logging method.
pub const WAVE40_METHOD_LOGGING_SET_LEVEL: &str = "logging/setLevel";
/// Dual-oracle residual: WAVE40 completion method.
pub const WAVE40_METHOD_COMPLETION_COMPLETE: &str = "completion/complete";

/// Dual-oracle residual: WAVE40 methods catalog.
pub const WAVE40_METHODS: &[&str] = &[
    WAVE40_METHOD_LOGGING_SET_LEVEL,
    WAVE40_METHOD_COMPLETION_COMPLETE,
];

/// Dual-oracle residual: known WAVE40 method.
#[must_use]
pub fn is_wave40_method(method: &str) -> bool {
    WAVE40_METHODS.contains(&method)
}

/// Dual-oracle residual: empty object shell.
#[must_use]
pub fn wave40_empty_object() -> serde_json::Value {
    serde_json::json!({})
}

/// Dual-oracle residual: method group for WAVE40 methods.
#[must_use]
pub fn wave40_method_group(method: &str) -> Option<&'static str> {
    match method {
        WAVE40_METHOD_LOGGING_SET_LEVEL => Some("logging"),
        WAVE40_METHOD_COMPLETION_COMPLETE => Some("completion"),
        _ => None,
    }
}

/// Dual-oracle residual: dual-oracle with existing METHOD_* constants.
#[must_use]
pub fn wave40_methods_match_ssot() -> bool {
    WAVE40_METHOD_LOGGING_SET_LEVEL == METHOD_LOGGING_SET_LEVEL
        && WAVE40_METHOD_COMPLETION_COMPLETE == METHOD_COMPLETION_COMPLETE
}

/// Dual-oracle residual: logging setLevel params require string level field.
#[must_use]
pub fn wave40_logging_set_level_params_valid(params: &serde_json::Value) -> bool {
    params
        .get("level")
        .and_then(|v| v.as_str())
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false)
}

/// Dual-oracle residual: completion complete params require ref object.
#[must_use]
pub fn wave40_completion_complete_params_valid(params: &serde_json::Value) -> bool {
    params.get("ref").map(|v| v.is_object()).unwrap_or(false)
}


#[cfg(test)]

// ── WAVE31 pure residual dens: resources/subscribe + logging/setLevel + completion/complete gate kernels ──

/// Dual-oracle residual method names.
pub const METHOD_RESOURCES_SUBSCRIBE: &str = "resources/subscribe";
pub const METHOD_RESOURCES_UNSUBSCRIBE: &str = "resources/unsubscribe";
pub const METHOD_LOGGING_SET_LEVEL: &str = "logging/setLevel";
pub const METHOD_COMPLETION_COMPLETE: &str = "completion/complete";

/// Dual-oracle residual: resources subscribe params require non-empty uri.
#[must_use]
pub fn resources_subscribe_params_valid(params: &serde_json::Value) -> bool {
    params
        .get("uri")
        .and_then(|v| v.as_str())
        .map(|s| !s.is_empty())
        .unwrap_or(false)
}

/// Dual-oracle residual: resources unsubscribe params require non-empty uri.
#[must_use]
pub fn resources_unsubscribe_params_valid(params: &serde_json::Value) -> bool {
    resources_subscribe_params_valid(params)
}

/// Dual-oracle residual: logging/setLevel params require known level.
#[must_use]
pub fn logging_set_level_params_gate(params: &serde_json::Value) -> bool {
    params
        .get("level")
        .and_then(|v| v.as_str())
        .map(logging_level_valid)
        .unwrap_or(false)
}

/// Dual-oracle residual: completion/complete requires ref object.
#[must_use]
pub fn completion_complete_params_valid(params: &serde_json::Value) -> bool {
    params
        .get("ref")
        .map(|r| r.is_object())
        .unwrap_or(false)
}

/// Dual-oracle residual: empty completion values list.
#[must_use]
pub fn empty_completion_values() -> serde_json::Value {
    serde_json::json!([])
}

/// Dual-oracle residual: wave31 empty object alias inventory.
#[must_use]
pub fn wave31_empty_object() -> serde_json::Value {
    serde_json::json!({})
}

/// Dual-oracle residual: is subscribe/unsubscribe method.
#[must_use]
pub fn is_resource_subscription_method(method: &str) -> bool {
    method == METHOD_RESOURCES_SUBSCRIBE || method == METHOD_RESOURCES_UNSUBSCRIBE
}

// ── WAVE32 pure residual dens: logging message + resource updated + list_changed residual kernels ──

/// Dual-oracle residual notification method names densed this wave.
pub const WAVE32_METHOD_NOTIFICATIONS_MESSAGE: &str = "notifications/message";
pub const WAVE32_METHOD_NOTIFICATIONS_RESOURCES_UPDATED: &str = "notifications/resources/updated";
pub const WAVE32_METHOD_NOTIFICATIONS_TOOLS_LIST_CHANGED: &str = "notifications/tools/list_changed";
pub const WAVE32_METHOD_NOTIFICATIONS_PROMPTS_LIST_CHANGED: &str =
    "notifications/prompts/list_changed";

/// Dual-oracle residual: notifications/message requires non-empty level + data presence.
#[must_use]
pub fn wave32_notifications_message_params_valid(params: &serde_json::Value) -> bool {
    let level_ok = params
        .get("level")
        .and_then(|v| v.as_str())
        .map(|s| !s.is_empty())
        .unwrap_or(false);
    let data_ok = params.get("data").is_some();
    level_ok && data_ok
}

/// Dual-oracle residual: resources/updated requires non-empty uri.
#[must_use]
pub fn wave32_resources_updated_params_valid(params: &serde_json::Value) -> bool {
    params
        .get("uri")
        .and_then(|v| v.as_str())
        .map(|s| !s.is_empty())
        .unwrap_or(false)
}

/// Dual-oracle residual: list_changed notifications accept empty params object/null.
#[must_use]
pub fn wave32_list_changed_params_ok(params: &serde_json::Value) -> bool {
    params.is_object() || params.is_null()
}

/// Dual-oracle residual: wave32 empty object alias inventory.
#[must_use]
pub fn wave32_empty_object() -> serde_json::Value {
    serde_json::json!({})
}

/// Dual-oracle residual: is wave32 densed notification method family.
#[must_use]
pub fn is_wave32_list_changed_or_message_method(method: &str) -> bool {
    matches!(
        method,
        "notifications/message"
            | "notifications/resources/updated"
            | "notifications/tools/list_changed"
            | "notifications/prompts/list_changed"
    )
}


// ── WAVE33 pure residual dens: sampling/createMessage + roots/list_changed residual kernels ──
// Dual-oracle residual of MCP method catalog halves not densed in WAVE32.
// Transport/handler I/O remains residual. NO authority_rust / package_main_rust invent.

/// Dual-oracle residual: sampling createMessage method.
pub const WAVE33_METHOD_SAMPLING_CREATE_MESSAGE: &str = "sampling/createMessage";
/// Dual-oracle residual: notifications/roots/list_changed method.
pub const WAVE33_METHOD_NOTIFICATIONS_ROOTS_LIST_CHANGED: &str =
    "notifications/roots/list_changed";
/// Dual-oracle residual: notifications/resources/list_changed method.
pub const WAVE33_METHOD_NOTIFICATIONS_RESOURCES_LIST_CHANGED: &str =
    "notifications/resources/list_changed";

/// Dual-oracle residual: sampling createMessage params gate (messages array required).
#[must_use]
pub fn wave33_sampling_create_message_params_valid(params: &serde_json::Value) -> bool {
    params
        .get("messages")
        .and_then(|m| m.as_array())
        .map(|a| !a.is_empty())
        .unwrap_or(false)
}

/// Dual-oracle residual: list_changed family accepts empty object / null / missing.
#[must_use]
pub fn wave33_list_changed_params_ok(params: &serde_json::Value) -> bool {
    params.is_null() || params.is_object()
}

/// Dual-oracle residual: wave33 empty object alias.
#[must_use]
pub fn wave33_empty_object() -> serde_json::Value {
    serde_json::json!({})
}

/// Dual-oracle residual: is wave33 densed method family.
#[must_use]
pub fn is_wave33_sampling_or_list_changed_method(method: &str) -> bool {
    matches!(
        method,
        "sampling/createMessage"
            | "notifications/roots/list_changed"
            | "notifications/resources/list_changed"
    )
}




// ── WAVE34 pure residual dens: notifications/progress + tools/list residual kernels ──
// Dual-oracle residual of MCP progress notification + tools/list method halves.
// Transport/handler I/O remains residual. NO authority_rust / package_main_rust invent.

/// Dual-oracle residual: notifications/progress method.
pub const WAVE34_METHOD_NOTIFICATIONS_PROGRESS: &str = "notifications/progress";
/// Dual-oracle residual: tools/list method.
pub const WAVE34_METHOD_TOOLS_LIST: &str = "tools/list";
/// Dual-oracle residual: tools/call method alias inventory.
pub const WAVE34_METHOD_TOOLS_CALL: &str = "tools/call";

/// Dual-oracle residual: progress params require progressToken + numeric progress.
#[must_use]
pub fn wave34_progress_params_valid(params: &serde_json::Value) -> bool {
    let token_ok = params.get("progressToken").is_some();
    let progress_ok = params
        .get("progress")
        .and_then(|v| v.as_f64())
        .map(|p| p.is_finite() && (0.0..=100.0).contains(&p))
        .unwrap_or(false);
    token_ok && progress_ok
}

/// Dual-oracle residual: tools/list accepts empty object / null params.
#[must_use]
pub fn wave34_tools_list_params_ok(params: &serde_json::Value) -> bool {
    params.is_null() || params.is_object()
}

/// Dual-oracle residual: wave34 empty object alias.
#[must_use]
pub fn wave34_empty_object() -> serde_json::Value {
    serde_json::json!({})
}

/// Dual-oracle residual: is wave34 densed method family.
#[must_use]
pub fn is_wave34_progress_or_tools_method(method: &str) -> bool {
    matches!(
        method,
        "notifications/progress" | "tools/list" | "tools/call"
    )
}

// ── WAVE35 pure residual dens: Method catalog completeness dual-oracle residual ──
// Dual-oracle residual of `src/protocol/mcp.ts` Method object keys/values pure halves.
// Transport/handler I/O remains residual. NO authority_rust / package_main_rust invent.

/// Dual-oracle residual: Method catalog wire values (product Method object order).
pub const WAVE35_METHOD_CATALOG: &[&str] = &[
    "initialize",
    "notifications/initialized",
    "ping",
    "resources/list",
    "resources/templates/list",
    "resources/read",
    "resources/subscribe",
    "resources/unsubscribe",
    "notifications/resources/updated",
    "notifications/resources/list_changed",
    "prompts/list",
    "prompts/get",
    "notifications/prompts/list_changed",
    "tools/list",
    "tools/call",
    "notifications/tools/list_changed",
    "logging/setLevel",
    "notifications/message",
    "completion/complete",
    "sampling/createMessage",
    "elicitation/create",
    "notifications/progress",
    "notifications/cancelled",
    "roots/list",
    "notifications/roots/list_changed",
];

/// Dual-oracle residual: Method object size.
pub const WAVE35_METHOD_COUNT: usize = 25;

/// Dual-oracle residual: known Method membership.
#[must_use]
pub fn is_wave35_method_catalog_member(method: &str) -> bool {
    WAVE35_METHOD_CATALOG.contains(&method)
}

/// Dual-oracle residual: catalog length dual-oracle of Object.keys(Method).length.
#[must_use]
pub fn wave35_method_catalog_count() -> usize {
    WAVE35_METHOD_CATALOG.len()
}

/// Dual-oracle residual: lifecycle methods pure residual subset.
#[must_use]
pub fn is_wave35_lifecycle_method(method: &str) -> bool {
    matches!(
        method,
        "initialize" | "notifications/initialized" | "ping"
    )
}

/// Dual-oracle residual: server→client request methods (sampling + elicitation).
#[must_use]
pub fn is_wave35_server_to_client_request(method: &str) -> bool {
    matches!(method, "sampling/createMessage" | "elicitation/create")
}

/// Dual-oracle residual: list_changed notification family.
#[must_use]
pub fn is_wave35_list_changed_notification(method: &str) -> bool {
    matches!(
        method,
        "notifications/resources/list_changed"
            | "notifications/prompts/list_changed"
            | "notifications/tools/list_changed"
            | "notifications/roots/list_changed"
    )
}

/// Dual-oracle residual: empty object alias.
#[must_use]
pub fn wave35_empty_object() -> serde_json::Value {
    serde_json::json!({})
}


// ── WAVE36 pure residual dens: Content type catalog dual-oracle dens ──
// Dual-oracle residual of `src/protocol/mcp.ts` Content union type tags.
// Transport/handler I/O remains residual. dens ≠ flip; package_main_rust not claimed.

/// Dual-oracle residual: Content type tags (stable order).
pub const WAVE36_CONTENT_TYPES: &[&str] = &["text", "image", "audio", "resource"];

/// Dual-oracle residual: content type count.
pub const WAVE36_CONTENT_TYPE_COUNT: usize = 4;

/// Dual-oracle residual: audience roles for content annotations.
pub const WAVE36_CONTENT_AUDIENCE: &[&str] = &["user", "assistant"];

/// Dual-oracle residual: known content type tag.
#[must_use]
pub fn is_wave36_content_type(ty: &str) -> bool {
    WAVE36_CONTENT_TYPES.contains(&ty)
}

/// Dual-oracle residual: content is media (image|audio).
#[must_use]
pub fn is_wave36_media_content_type(ty: &str) -> bool {
    matches!(ty, "image" | "audio")
}

/// Dual-oracle residual: content is resource embed.
#[must_use]
pub fn is_wave36_resource_content_type(ty: &str) -> bool {
    ty == "resource"
}

/// Dual-oracle residual: known audience role.
#[must_use]
pub fn is_wave36_content_audience(role: &str) -> bool {
    WAVE36_CONTENT_AUDIENCE.contains(&role)
}

/// Dual-oracle residual: priority annotation in [0, 1] if present.
#[must_use]
pub fn wave36_priority_valid(priority: Option<f64>) -> bool {
    match priority {
        None => true,
        Some(p) if p.is_finite() => (0.0..=1.0).contains(&p),
        Some(_) => false,
    }
}

#[must_use]
pub fn wave36_empty_object() -> serde_json::Value {
    serde_json::json!({})
}

// ── WAVE37 pure residual dens: Protocol version catalog dual-oracle dens ──
// Dual-oracle of `src/protocol/mcp.ts` LATEST_PROTOCOL_VERSION /
// SUPPORTED_PROTOCOL_VERSIONS pure residual catalog.
// Transport/handler I/O residual retained. dens ≠ flip. package_main_rust not claimed.

/// Dual-oracle residual: latest protocol version wire.
pub const WAVE37_LATEST_PROTOCOL_VERSION: &str = "2025-03-26";
/// Dual-oracle residual: supported protocol versions catalog.
pub const WAVE37_SUPPORTED_PROTOCOL_VERSIONS: &[&str] = &["2025-03-26", "2024-11-05"];
/// Dual-oracle residual: supported catalog count.
pub const WAVE37_SUPPORTED_PROTOCOL_VERSION_COUNT: usize = 2;

/// Dual-oracle residual: known supported protocol version.
#[must_use]
pub fn is_wave37_supported_protocol_version(v: &str) -> bool {
    WAVE37_SUPPORTED_PROTOCOL_VERSIONS.contains(&v)
}

/// Dual-oracle residual: version is latest.
#[must_use]
pub fn is_wave37_latest_protocol_version(v: &str) -> bool {
    v == WAVE37_LATEST_PROTOCOL_VERSION
}

/// Dual-oracle residual: negotiate — pick client if supported else latest.
#[must_use]
pub fn wave37_negotiate_protocol_version(client: Option<&str>) -> &'static str {
    match client {
        Some(v) if is_wave37_supported_protocol_version(v) => {
            // return the static catalog entry
            WAVE37_SUPPORTED_PROTOCOL_VERSIONS
                .iter()
                .copied()
                .find(|c| *c == v)
                .unwrap_or(WAVE37_LATEST_PROTOCOL_VERSION)
        }
        _ => WAVE37_LATEST_PROTOCOL_VERSION,
    }
}

/// Dual-oracle residual: empty object helper (parity with prior dens waves).
#[must_use]
pub fn wave37_empty_object() -> serde_json::Value {
    serde_json::json!({})
}

// ── WAVE38 pure residual dens: method group membership dual-oracle dens ──
// Dual-oracle residual of protocol method group catalogs pure halves
// (tools/prompts/resources/lifecycle membership helpers).
// Transport/handler I/O residual retained. dens ≠ flip. package_main_rust not claimed.

/// Dual-oracle residual: method group labels catalog.
pub const WAVE38_METHOD_GROUPS: &[&str] = &[
    "lifecycle",
    "tools",
    "prompts",
    "resources",
    "notifications",
    "logging",
    "completion",
    "sampling",
];

/// Dual-oracle residual: known method group.
#[must_use]
pub fn is_wave38_method_group(group: &str) -> bool {
    WAVE38_METHOD_GROUPS.contains(&group)
}

/// Dual-oracle residual: classify method into group (pure prefix half).
#[must_use]
pub fn wave38_method_group(method: &str) -> Option<&'static str> {
    if method == "initialize" || method == "ping" || method == "shutdown" {
        Some("lifecycle")
    } else if method.starts_with("tools/") {
        Some("tools")
    } else if method.starts_with("prompts/") {
        Some("prompts")
    } else if method.starts_with("resources/") {
        Some("resources")
    } else if method.starts_with("notifications/") {
        Some("notifications")
    } else if method.starts_with("logging/") {
        Some("logging")
    } else if method.starts_with("completion/") {
        Some("completion")
    } else if method.starts_with("sampling/") {
        Some("sampling")
    } else {
        None
    }
}

/// Dual-oracle residual: tools list vs call membership.
#[must_use]
pub fn is_wave38_tools_method(method: &str) -> bool {
    wave38_method_group(method) == Some("tools")
}

/// Dual-oracle residual: notifications family.
#[must_use]
pub fn is_wave38_notification_method(method: &str) -> bool {
    wave38_method_group(method) == Some("notifications")
}

/// Dual-oracle residual: empty object helper (parity with prior dens waves).
#[must_use]
pub fn wave38_empty_object() -> serde_json::Value {
    serde_json::json!({})
}






mod tests {
    use super::*;

    fn array_len(v: &serde_json::Value, key: &str) -> usize {
        v.get(key)
            .and_then(|x| x.as_array())
            .map(|a| a.len())
            .unwrap_or(usize::MAX)
    }

    #[test]
    fn supported_versions() {
        assert!(is_supported_protocol_version("2025-03-26"));
        assert!(is_supported_protocol_version("2024-11-05"));
        assert!(!is_supported_protocol_version("1.0.0"));
    }

    #[test]
    fn negotiate_prefers_client_when_known() {
        assert_eq!(
            negotiate_protocol_version(Some("2024-11-05")),
            "2024-11-05"
        );
        assert_eq!(
            negotiate_protocol_version(Some("nope")),
            LATEST_PROTOCOL_VERSION
        );
        assert_eq!(negotiate_protocol_version(None), LATEST_PROTOCOL_VERSION);
    }

    #[test]
    fn text_content_roundtrip() {
        let c = Content::text("hello");
        assert!(c.is_text());
        let v = match serde_json::to_value(&c) {
            Ok(v) => v,
            Err(e) => panic!("serialize: {e}"),
        };
        assert_eq!(v["type"], "text");
        assert_eq!(v["text"], "hello");
    }

    #[test]
    fn tools_list_with_cursor() {
        let tools = [Tool {
            name: "ping".into(),
            title: None,
            description: Some("p".into()),
            input_schema: serde_json::json!({"type": "object"}),
            output_schema: None,
            annotations: None,
        }];
        let body = tools_list_result(&tools, Some("abc".into()));
        assert_eq!(body["tools"][0]["name"], "ping");
        assert_eq!(body["nextCursor"], "abc");
    }

    #[test]
    fn content_constructors_and_tool_error() {
        let t = Content::text("hi");
        assert!(t.is_text());
        let img = Content::image("AAAA", "image/png");
        assert!(matches!(img, Content::Image { .. }));
        let err = tool_error("boom");
        assert_eq!(err["isError"], true);
        assert_eq!(err["content"][0]["text"], "boom");
        let jt = json_text_content(&serde_json::json!({"a": 1}));
        assert!(jt.is_text());
    }

    #[test]
    fn notification_helpers() {
        let n = tools_list_changed();
        assert!(matches!(n, Notification::ToolsListChanged {}));
        let p = progress_notification(serde_json::json!(1), 0.5, Some(1.0), None);
        assert!(matches!(p, Notification::Progress { .. }));
        let c = cancelled(serde_json::json!("id-1"), Some("stop".into()));
        assert!(matches!(c, Notification::Cancelled { .. }));
    }

    #[test]
    fn template_match_and_extract() {
        // {param} matches a single path segment (TS [^/]+)
        assert!(matches_template("file:///{path}", "file:///tmp"));
        assert!(!matches_template("file:///{path}", "file:///tmp/a"));
        let p = match extract_params("file:///{bucket}/{key}", "file:///b1/obj") {
            Some(p) => p,
            None => panic!("expected params"),
        };
        assert_eq!(p.get("bucket").map(String::as_str), Some("b1"));
        assert_eq!(p.get("key").map(String::as_str), Some("obj"));
        assert!(extract_params("file:///{bucket}/{key}", "file:///b1/obj/extra").is_none());
        assert!(matches_template("static", "static"));
        assert!(!matches_template("static", "static2"));
        let p2 = match extract_params("/users/{id}/files/{name}", "/users/42/files/a.txt") {
            Some(p) => p,
            None => panic!("expected params"),
        };
        assert_eq!(p2.get("id").map(String::as_str), Some("42"));
        assert_eq!(p2.get("name").map(String::as_str), Some("a.txt"));
    }

    #[test]
    fn interpolate_placeholders() {
        let mut args = std::collections::BTreeMap::new();
        args.insert("name".into(), "Ada".into());
        assert_eq!(interpolate("Hello {{name}}!", &args), "Hello Ada!");
        assert_eq!(interpolate("Hi {{missing}}", &args), "Hi {{missing}}");
        assert_eq!(interpolate("nope", &args), "nope");
    }

    #[test]
    fn prompt_message_helpers() {
        let u = user_message("hi");
        assert_eq!(u.role, "user");
        let a = assistant_message("yo");
        assert_eq!(a.role, "assistant");
        let body = prompt_result("desc", &[u, a]);
        assert_eq!(body["description"], "desc");
        assert_eq!(array_len(&body, "messages"), 2);
        assert_eq!(methods::TOOLS_CALL, "tools/call");
        assert_eq!(methods::INITIALIZE, "initialize");
    }

    #[test]
    fn wave5_log_and_embedded() {
        let n = log_notification("info", serde_json::json!({"ok": true}), Some("app".into()));
        assert!(matches!(n, Notification::Log { .. }));
        let c = embedded_resource_content("file:///x", Some("text/plain".into()), Some("hi".into()));
        assert!(matches!(c, Content::Resource { .. }));
        if let Content::Resource { resource, .. } = c {
            assert_eq!(resource.uri, "file:///x");
            assert_eq!(resource.text.as_deref(), Some("hi"));
        }
    }
    #[test]
    fn wave6_resource_and_multi_content() {
        let rt = resource_text("file:///a", "hello", None);
        assert_eq!(rt.uri, "file:///a");
        assert_eq!(rt.text.as_deref(), Some("hello"));
        assert_eq!(rt.mime_type.as_deref(), Some("text/plain"));
        let rr = resources_read_result(&[rt]);
        assert_eq!(array_len(&rr, "contents"), 1);
        let blob = resource_blob("file:///b", "AAAA", "image/png");
        assert_eq!(blob["contents"][0]["blob"], "AAAA");
        let contents = [
            Content::text("t"),
            Content::image("AA", "image/png"),
            Content::audio("BB", "audio/wav"),
        ];
        let body = tool_content_result(&contents, false);
        assert_eq!(body["isError"], false);
        assert_eq!(array_len(&body, "content"), 3);
    }

    #[test]
    fn wave7_media_prompt_json() {
        // FLEET-ENTERPRISE-WAVE7
        let img = image_content("AAAA", "image/png");
        assert!(matches!(img, Content::Image { .. }));
        let aud = audio_content("BBBB", "audio/wav");
        assert!(matches!(aud, Content::Audio { .. }));
        let msg = prompt_message("user", Content::text("hi"));
        assert_eq!(msg.role, "user");
        let pretty = json_text_pretty(&serde_json::json!({"a": 1}));
        match pretty {
            Content::Text { text, .. } => assert!(text.contains("\"a\"")),
            _ => panic!("expected text"),
        }
    }

    #[test]
    fn wave8_list_envelopes_and_error_flag() {
        // FLEET-ENTERPRISE-WAVE8
        let rr = resources_list_result(
            &[serde_json::json!({"uri": "file:///a", "name": "a"})],
            Some("c1".into()),
        );
        assert_eq!(array_len(&rr, "resources"), 1);
        assert_eq!(rr["nextCursor"], "c1");
        let pr = prompts_list_result(&[], None);
        assert_eq!(array_len(&pr, "prompts"), 0);
        assert!(pr.get("nextCursor").is_none());
        let empty = empty_tool_result();
        assert_eq!(empty["isError"], false);
        assert_eq!(array_len(&empty, "content"), 0);
        assert!(!is_tool_error_result(&empty));
        assert!(is_tool_error_result(&tool_error("boom")));
    }

    #[test]
    fn bulk_template_multi_segment_and_reject_extra() {
        assert!(!matches_template(
            "/users/{id}/posts/{pid}",
            "/users/1/posts/2/extra"
        ));
        let p = match extract_params("/users/{id}/posts/{pid}", "/users/1/posts/2") {
            Some(p) => p,
            None => panic!("expected params"),
        };
        assert_eq!(p.get("id").map(String::as_str), Some("1"));
        assert_eq!(p.get("pid").map(String::as_str), Some("2"));
        assert!(extract_params("/users/{id}", "/users/a/b").is_none());
        assert!(extract_params("/users/{id}", "/users/").is_none());
    }

    #[test]
    fn bulk_interpolate_missing_and_valid() {
        let mut args = std::collections::BTreeMap::new();
        args.insert("name".into(), "Ada".into());
        assert_eq!(interpolate("hi {{name}}", &args), "hi Ada");
        assert_eq!(interpolate("hi {{missing}}", &args), "hi {{missing}}");
    }

    #[test]
    fn bulk_notifications_cancelled_and_progress() {
        let n = cancelled(serde_json::json!(1), Some("stop".into()));
        assert!(matches!(n, Notification::Cancelled { .. }));
        let p = progress_notification(
            serde_json::json!("t1"),
            1.0,
            Some(10.0),
            Some("working".into()),
        );
        assert!(matches!(p, Notification::Progress { .. }));
        assert!(!is_tool_error_result(&empty_tool_result()));
        assert!(is_tool_error_result(&tool_text_result("x", true)));
        assert!(!is_tool_error_result(&tool_text_result("x", false)));
    }

    #[test]
    fn bulk_list_results_without_cursor_omit_next() {
        let tools = tools_list_result(&[], None);
        let next = tools.get("nextCursor");
        assert!(next.is_none() || next.is_some_and(|v| v.is_null()));
        let tools2 = tools_list_result(&[], Some("c".into()));
        assert_eq!(tools2.get("nextCursor").and_then(|v| v.as_str()), Some("c"));
    }

    /// SHA-bound pure residual golden: protocol version + tool error envelope.
    #[test]
    fn pure_residual_protocol_golden_fixture() {
        let raw = include_str!("../fixtures/protocol_golden.json");
        let doc: serde_json::Value = match serde_json::from_str(raw) {
            Ok(v) => v,
            Err(e) => panic!("protocol_golden.json: {e}"),
        };
        assert_eq!(doc["schema"], "mcp-protocol-golden/v1");
        assert_eq!(doc["latestProtocolVersion"], LATEST_PROTOCOL_VERSION);
        let supported = match doc["supportedProtocolVersions"].as_array() {
            Some(a) => a,
            None => panic!("supportedProtocolVersions"),
        };
        for v in supported {
            let s = match v.as_str() {
                Some(s) => s,
                None => panic!("version not string"),
            };
            assert!(is_supported_protocol_version(s));
        }
        for case in match doc["negotiate"].as_array() {
            Some(a) => a,
            None => panic!("negotiate"),
        } {
            let client = case.get("client").and_then(|v| v.as_str());
            let expected = match case["expected"].as_str() {
                Some(s) => s,
                None => panic!("expected"),
            };
            assert_eq!(negotiate_protocol_version(client), expected);
        }
        let err = tool_error("fixture-boom");
        assert!(is_tool_error_result(&err));
        assert_eq!(err["content"][0]["text"], "fixture-boom");
        assert!(!is_tool_error_result(&empty_tool_result()));

        // WAVE10: notification → JSON-RPC method mapping golden
        if let Some(cases) = doc.get("notificationToJsonRpc").and_then(|v| v.as_array()) {
            for case in cases {
                let name = case["name"].as_str().unwrap_or("?");
                let ntype = match case["notificationType"].as_str() {
                    Some(s) => s,
                    None => panic!("notificationType for {name}"),
                };
                let n = match ntype {
                    "tools/list_changed" => tools_list_changed(),
                    "resources/list_changed" => resources_list_changed(),
                    "prompts/list_changed" => prompts_list_changed(),
                    "resource/updated" => {
                        let uri = case["uri"].as_str().unwrap_or("file:///x");
                        resource_updated(uri)
                    }
                    "progress" => progress_notification(
                        case.get("progressToken")
                            .cloned()
                            .unwrap_or(serde_json::json!(1)),
                        case["progress"].as_f64().unwrap_or(0.0),
                        case.get("total").and_then(|v| v.as_f64()),
                        case.get("message")
                            .and_then(|v| v.as_str())
                            .map(str::to_string),
                    ),
                    "log" => log_notification(
                        case["level"].as_str().unwrap_or("info"),
                        case.get("data").cloned().unwrap_or(serde_json::json!({})),
                        case.get("logger")
                            .and_then(|v| v.as_str())
                            .map(str::to_string),
                    ),
                    "cancelled" => cancelled(
                        case.get("requestId")
                            .cloned()
                            .unwrap_or(serde_json::json!(1)),
                        case.get("reason")
                            .and_then(|v| v.as_str())
                            .map(str::to_string),
                    ),
                    other => panic!("unknown notificationType {other} in {name}"),
                };
                let wire = notification_to_jsonrpc(&n);
                let expected_method = match case["expectedMethod"].as_str() {
                    Some(s) => s,
                    None => panic!("expectedMethod for {name}"),
                };
                assert_eq!(wire.method, expected_method, "case {name}");
                let expect_params = case["expectParams"].as_bool().unwrap_or(false);
                assert_eq!(wire.params.is_some(), expect_params, "case {name} params");
            }
        }

        // WAVE10: normalize_tool_result golden
        if let Some(cases) = doc.get("normalizeToolResult").and_then(|v| v.as_array()) {
            for case in cases {
                let name = case["name"].as_str().unwrap_or("?");
                let input = match case.get("input") {
                    Some(v) => v,
                    None => panic!("input for {name}"),
                };
                let out = normalize_tool_result(input);
                let expected_len = case["expectedContentLen"].as_u64().unwrap_or(0) as usize;
                assert_eq!(array_len(&out, "content"), expected_len, "case {name}");
                if let Some(sc) = case.get("expectedStructuredContent") {
                    assert_eq!(
                        out.get("structuredContent"),
                        Some(sc),
                        "case {name} structuredContent"
                    );
                }
            }
        }

        // WAVE10: log levels
        if let Some(levels) = doc.get("logLevels").and_then(|v| v.as_array()) {
            for v in levels {
                let s = match v.as_str() {
                    Some(s) => s,
                    None => panic!("log level not string"),
                };
                assert!(is_valid_log_level(s), "level {s}");
            }
        }
        assert!(!is_valid_log_level("trace"));

        // WAVE11: MCP method catalog golden
        if let Some(methods_list) = doc.get("mcpMethods").and_then(|v| v.as_array()) {
            assert_eq!(methods_list.len(), methods::ALL.len());
            for v in methods_list {
                let s = match v.as_str() {
                    Some(s) => s,
                    None => panic!("method not string"),
                };
                assert!(is_mcp_method(s), "method {s}");
            }
        }
        assert!(!is_mcp_method("not/a/method"));

        // WAVE11: protocol descriptors golden
        if let Some(cases) = doc.get("protocolDescriptors").and_then(|v| v.as_array()) {
            for case in cases {
                let kind = case["kind"].as_str().unwrap_or("?");
                match kind {
                    "resource" => {
                        let r = protocol_resource(
                            case["name"].as_str().unwrap_or(""),
                            case["uri"].as_str().unwrap_or(""),
                            case.get("description")
                                .and_then(|v| v.as_str())
                                .map(str::to_string),
                            case.get("mimeType")
                                .and_then(|v| v.as_str())
                                .map(str::to_string),
                        );
                        assert_eq!(r.name, case["name"].as_str().unwrap_or(""));
                        assert_eq!(r.uri, case["uri"].as_str().unwrap_or(""));
                    }
                    "template" => {
                        let t = protocol_template(
                            case["name"].as_str().unwrap_or(""),
                            case["uriTemplate"].as_str().unwrap_or(""),
                            case.get("description")
                                .and_then(|v| v.as_str())
                                .map(str::to_string),
                            case.get("mimeType")
                                .and_then(|v| v.as_str())
                                .map(str::to_string),
                        );
                        assert_eq!(
                            t.uri_template,
                            case["uriTemplate"].as_str().unwrap_or("")
                        );
                        if let Some(ok) = case.get("matchUri").and_then(|v| v.as_str()) {
                            assert!(matches_template(&t.uri_template, ok));
                        }
                        if let Some(bad) = case.get("rejectUri").and_then(|v| v.as_str()) {
                            assert!(!matches_template(&t.uri_template, bad));
                        }
                    }
                    "prompt" => {
                        let args = case
                            .get("arguments")
                            .and_then(|v| v.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .map(|a| PromptArgument {
                                        name: a["name"].as_str().unwrap_or("").into(),
                                        description: a
                                            .get("description")
                                            .and_then(|v| v.as_str())
                                            .map(str::to_string),
                                        required: a.get("required").and_then(|v| v.as_bool()),
                                    })
                                    .collect::<Vec<_>>()
                            });
                        let p = protocol_prompt(
                            case["name"].as_str().unwrap_or(""),
                            case.get("description")
                                .and_then(|v| v.as_str())
                                .map(str::to_string),
                            args,
                        );
                        assert_eq!(p.name, case["name"].as_str().unwrap_or(""));
                    }
                    other => panic!("unknown protocolDescriptors kind {other}"),
                }
            }
        }

        // WAVE11: resource contents wire golden
        if let Some(cases) = doc.get("resourceContents").and_then(|v| v.as_array()) {
            for case in cases {
                let name = case["name"].as_str().unwrap_or("?");
                if case.get("text").is_some() {
                    let body = resource_text_result(
                        case["uri"].as_str().unwrap_or(""),
                        case["text"].as_str().unwrap_or(""),
                        None,
                    );
                    assert_eq!(
                        body["contents"][0]["type"].as_str(),
                        case.get("expectedType").and_then(|v| v.as_str()),
                        "case {name}"
                    );
                    if let Some(mime) = case.get("expectedMime").and_then(|v| v.as_str()) {
                        assert_eq!(body["contents"][0]["mimeType"], mime, "case {name}");
                    }
                } else if case.get("blob").is_some() {
                    let body = resource_blob(
                        case["uri"].as_str().unwrap_or(""),
                        case["blob"].as_str().unwrap_or(""),
                        case["mimeType"].as_str().unwrap_or("application/octet-stream"),
                    );
                    assert_eq!(
                        body["contents"][0]["type"].as_str(),
                        case.get("expectedType").and_then(|v| v.as_str()),
                        "case {name}"
                    );
                    assert_eq!(
                        body["contents"][0]["blob"].as_str(),
                        case.get("blob").and_then(|v| v.as_str()),
                        "case {name}"
                    );
                }
            }
        }

        // WAVE11: elicitation actions golden
        if let Some(actions) = doc.get("elicitationActions").and_then(|v| v.as_array()) {
            for v in actions {
                let s = match v.as_str() {
                    Some(s) => s,
                    None => panic!("elicitation action not string"),
                };
                assert!(is_valid_elicitation_action(s), "action {s}");
            }
        }

        // WAVE11: sampling envelope golden
        if let Some(sc) = doc.get("samplingCreate") {
            let max_tokens = sc["maxTokens"].as_u64().unwrap_or(0);
            let params = sampling_create_params(
                &[serde_json::json!({"role": "user", "content": {"type": "text", "text": "hi"}})],
                max_tokens,
                None,
                None,
            );
            assert_eq!(params["maxTokens"], max_tokens);
            let msg_count = sc["messageCount"].as_u64().unwrap_or(0) as usize;
            assert_eq!(array_len(&params, "messages"), msg_count);
            let result = sampling_create_result(
                "assistant",
                Content::text("ok"),
                sc["model"].as_str().unwrap_or("m"),
                sc.get("stopReason")
                    .and_then(|v| v.as_str())
                    .map(str::to_string),
            );
            assert_eq!(
                result["stopReason"].as_str(),
                sc.get("stopReason").and_then(|v| v.as_str())
            );
        }

        // WAVE12: request params + method kind goldens
        if let Some(cases) = doc.get("requestParams").and_then(|v| v.as_array()) {
            for case in cases {
                let name = case["name"].as_str().unwrap_or("?");
                let kind = case["kind"].as_str().unwrap_or("?");
                match kind {
                    "initialize" => {
                        let p = initialize_params(
                            case["protocolVersion"].as_str().unwrap_or(""),
                            case["clientName"].as_str().unwrap_or(""),
                            case["clientVersion"].as_str().unwrap_or(""),
                            case.get("capabilities")
                                .cloned()
                                .unwrap_or_else(|| serde_json::json!({})),
                        );
                        assert_eq!(
                            p["clientInfo"]["name"].as_str(),
                            case["clientName"].as_str(),
                            "case {name}"
                        );
                    }
                    "tools_call" => {
                        let p = tools_call_params(
                            case["toolName"].as_str().unwrap_or(""),
                            case.get("arguments").cloned(),
                        );
                        assert_eq!(
                            p["name"].as_str(),
                            case["toolName"].as_str(),
                            "case {name}"
                        );
                    }
                    "resources_read" => {
                        let p = resources_read_params(case["uri"].as_str().unwrap_or(""));
                        assert_eq!(p["uri"].as_str(), case["uri"].as_str(), "case {name}");
                    }
                    "list" => {
                        let cursor = case
                            .get("cursor")
                            .and_then(|v| if v.is_null() { None } else { v.as_str() })
                            .map(str::to_string);
                        let p = list_params(cursor);
                        if let Some(c) = case.get("cursor").and_then(|v| v.as_str()) {
                            assert_eq!(p["cursor"], c, "case {name}");
                        } else {
                            assert!(
                                p.as_object().is_some_and(|m| m.is_empty()),
                                "case {name}"
                            );
                        }
                    }
                    "completion" => {
                        let p = completion_complete_params(
                            case["refType"].as_str().unwrap_or(""),
                            case.get("refName")
                                .and_then(|v| v.as_str())
                                .map(str::to_string),
                            case.get("refUri")
                                .and_then(|v| v.as_str())
                                .map(str::to_string),
                            case["argumentName"].as_str().unwrap_or(""),
                            case["argumentValue"].as_str().unwrap_or(""),
                        );
                        assert_eq!(
                            p["ref"]["type"].as_str(),
                            case["refType"].as_str(),
                            "case {name}"
                        );
                    }
                    other => panic!("unknown requestParams kind {other} in {name}"),
                }
            }
        }
        if let Some(cases) = doc.get("methodKinds").and_then(|v| v.as_array()) {
            for case in cases {
                let method = case["method"].as_str().unwrap_or("");
                let kind = case["kind"].as_str().unwrap_or("");
                match kind {
                    "request" => {
                        assert!(is_mcp_request_method(method), "{method}");
                        assert!(!is_mcp_notification_method(method), "{method}");
                    }
                    "notification" => {
                        assert!(is_mcp_notification_method(method), "{method}");
                        assert!(!is_mcp_request_method(method), "{method}");
                    }
                    other => panic!("unknown methodKinds kind {other}"),
                }
            }
        }

        // WAVE13: method domains golden
        if let Some(cases) = doc.get("methodDomains").and_then(|v| v.as_array()) {
            for case in cases {
                let method = case["method"].as_str().unwrap_or("");
                let domain = case["domain"].as_str().unwrap_or("");
                assert_eq!(method_domain(method), domain, "method {method}");
            }
        }

        // WAVE13: includeContext / stopReason catalogs
        if let Some(vals) = doc.get("includeContexts").and_then(|v| v.as_array()) {
            for v in vals {
                let s = v.as_str().unwrap_or("");
                assert!(is_valid_include_context(s), "includeContext {s}");
            }
        }
        if let Some(vals) = doc.get("canonicalStopReasons").and_then(|v| v.as_array()) {
            for v in vals {
                let s = v.as_str().unwrap_or("");
                assert!(is_canonical_stop_reason(s), "stopReason {s}");
            }
        }

        // WAVE13: cancelled + tools_call with progress meta
        if let Some(cases) = doc.get("requestParamsWave13").and_then(|v| v.as_array()) {
            for case in cases {
                let name = case["name"].as_str().unwrap_or("?");
                let kind = case["kind"].as_str().unwrap_or("?");
                match kind {
                    "cancelled" => {
                        let p = cancelled_params(
                            case.get("requestId").cloned().unwrap_or(serde_json::json!(1)),
                            case.get("reason").and_then(|v| v.as_str()).map(str::to_string),
                        );
                        assert_eq!(
                            p.get("requestId"),
                            case.get("requestId"),
                            "case {name}"
                        );
                    }
                    "tools_call_progress" => {
                        let p = tools_call_params_with_progress(
                            case["toolName"].as_str().unwrap_or(""),
                            case.get("arguments").cloned(),
                            case.get("progressToken").cloned(),
                        );
                        assert_eq!(
                            p["_meta"]["progressToken"],
                            case.get("progressToken").cloned().unwrap_or(serde_json::Value::Null),
                            "case {name}"
                        );
                    }
                    other => panic!("unknown requestParamsWave13 kind {other}"),
                }
            }
        }

        // WAVE14: catalogs + server→client + list_changed + progressToken extract
        if let Some(vals) = doc.get("messageRoles").and_then(|v| v.as_array()) {
            for v in vals {
                let s = v.as_str().unwrap_or("");
                assert!(is_valid_message_role(s), "role {s}");
            }
        }
        if let Some(vals) = doc.get("contentTypes").and_then(|v| v.as_array()) {
            for v in vals {
                let s = v.as_str().unwrap_or("");
                assert!(is_valid_content_type(s), "contentType {s}");
            }
        }
        if let Some(vals) = doc.get("audienceValues").and_then(|v| v.as_array()) {
            for v in vals {
                let s = v.as_str().unwrap_or("");
                assert!(is_valid_audience(s), "audience {s}");
            }
        }
        if let Some(vals) = doc.get("serverToClientMethods").and_then(|v| v.as_array()) {
            for v in vals {
                let s = v.as_str().unwrap_or("");
                assert!(is_server_to_client_method(s), "s2c {s}");
            }
        }
        if let Some(vals) = doc.get("completionRefTypes").and_then(|v| v.as_array()) {
            for v in vals {
                let s = v.as_str().unwrap_or("");
                assert!(is_valid_completion_ref_type(s), "ref {s}");
            }
        }
        if let Some(vals) = doc.get("elicitationPropertyTypes").and_then(|v| v.as_array()) {
            for v in vals {
                let s = v.as_str().unwrap_or("");
                assert!(is_valid_elicitation_property_type(s), "prop {s}");
            }
        }
        if let Some(cases) = doc.get("listChangedMethods").and_then(|v| v.as_array()) {
            for case in cases {
                let method = case["method"].as_str().unwrap_or("");
                let expected = case["isListChanged"].as_bool().unwrap_or(false);
                assert_eq!(
                    is_list_changed_notification(method),
                    expected,
                    "listChanged {method}"
                );
            }
        }
        if let Some(cases) = doc.get("logLevelAtLeast").and_then(|v| v.as_array()) {
            for case in cases {
                let level = case["level"].as_str().unwrap_or("");
                let minimum = case["minimum"].as_str().unwrap_or("");
                let expected = case["expected"].as_bool().unwrap_or(false);
                assert_eq!(
                    log_level_at_least(level, minimum),
                    expected,
                    "log {level}>={minimum}"
                );
            }
        }
        if let Some(cases) = doc.get("progressTokenExtract").and_then(|v| v.as_array()) {
            for case in cases {
                let name = case["name"].as_str().unwrap_or("?");
                let params = case
                    .get("params")
                    .cloned()
                    .unwrap_or_else(|| serde_json::json!({}));
                let got = progress_token_from_call_params(&params);
                if case.get("expected").map(|v| v.is_null()).unwrap_or(true)
                    && case.get("expectedToken").is_none()
                {
                    assert!(got.is_none(), "case {name}");
                } else if let Some(tok) = case.get("expectedToken") {
                    assert_eq!(got, Some(tok), "case {name}");
                }
            }
        }
        if let Some(case) = doc.get("rootsListChangedWire") {
            let wire = notification_to_jsonrpc(&roots_list_changed());
            assert_eq!(
                wire.method.as_str(),
                case["expectedMethod"].as_str().unwrap_or(""),
                "roots list_changed method"
            );
            assert!(wire.params.is_none());
        }
        // WAVE15: method direction + list cursor + priority + completion_argument
        if let Some(cases) = doc.get("methodDirections").and_then(|v| v.as_array()) {
            for case in cases {
                let method = case["method"].as_str().unwrap_or("");
                let expected = case["direction"].as_str().unwrap_or("");
                assert_eq!(
                    method_direction(method),
                    expected,
                    "direction {method}"
                );
            }
        }
        if let Some(vals) = doc
            .get("clientToServerRequestMethods")
            .and_then(|v| v.as_array())
        {
            for v in vals {
                let s = v.as_str().unwrap_or("");
                assert!(is_client_to_server_request_method(s), "c2s {s}");
                assert_eq!(method_direction(s), "client_to_server", "c2s dir {s}");
            }
        }
        if let Some(vals) = doc.get("notificationMethods").and_then(|v| v.as_array()) {
            for v in vals {
                let s = v.as_str().unwrap_or("");
                assert!(NOTIFICATION_METHODS.contains(&s), "notif catalog {s}");
                assert_eq!(method_direction(s), "notification", "notif dir {s}");
            }
        }
        if let Some(cases) = doc.get("listNextCursor").and_then(|v| v.as_array()) {
            for case in cases {
                let name = case["name"].as_str().unwrap_or("?");
                let result = case
                    .get("result")
                    .cloned()
                    .unwrap_or_else(|| serde_json::json!({}));
                let has = case["hasNext"].as_bool().unwrap_or(false);
                assert_eq!(list_has_next_cursor(&result), has, "list cursor {name}");
                if let Some(c) = case.get("cursor").and_then(|v| v.as_str()) {
                    assert_eq!(list_next_cursor(&result), Some(c), "list next {name}");
                } else {
                    assert!(list_next_cursor(&result).is_none(), "list next none {name}");
                }
            }
        }
        if let Some(cases) = doc.get("priorityClamp").and_then(|v| v.as_array()) {
            for case in cases {
                let name = case["name"].as_str().unwrap_or("?");
                let raw = case["raw"].as_f64().unwrap_or(0.0);
                let valid = case["valid"].as_bool().unwrap_or(false);
                let clamped = case["clamped"].as_f64().unwrap_or(0.0);
                assert_eq!(is_valid_priority(raw), valid, "priority valid {name}");
                assert!((clamp_priority(raw) - clamped).abs() < 1e-9, "clamp {name}");
            }
        }
        if let Some(case) = doc.get("completionArgument") {
            let got = completion_argument(
                case["name"].as_str().unwrap_or(""),
                case["value"].as_str().unwrap_or(""),
            );
            assert_eq!(got["name"], case["name"]);
            assert_eq!(got["value"], case["value"]);
        }
        if let Some(cases) = doc.get("toolResultFirstText").and_then(|v| v.as_array()) {
            for case in cases {
                let name = case["name"].as_str().unwrap_or("?");
                let result = case
                    .get("result")
                    .cloned()
                    .unwrap_or_else(|| serde_json::json!({}));
                let got = tool_result_first_text(&result);
                if let Some(t) = case.get("expected").and_then(|v| v.as_str()) {
                    assert_eq!(got, Some(t), "first text {name}");
                } else {
                    assert!(got.is_none(), "first text none {name}");
                }
            }
        }
        if let Some(cases) = doc.get("fileUris").and_then(|v| v.as_array()) {
            for case in cases {
                let uri = case["uri"].as_str().unwrap_or("");
                let expected = case["isFile"].as_bool().unwrap_or(false);
                assert_eq!(is_file_uri(uri), expected, "file uri {uri}");
            }
        }
        // WAVE16: domain catalogs + content/uri/list extractors
        if let Some(vals) = doc.get("resourcesMethods").and_then(|v| v.as_array()) {
            for v in vals {
                let s = v.as_str().unwrap_or("");
                assert!(is_resources_method(s), "resources {s}");
                assert!(RESOURCES_METHODS.contains(&s), "resources catalog {s}");
            }
        }
        if let Some(vals) = doc.get("toolsMethods").and_then(|v| v.as_array()) {
            for v in vals {
                let s = v.as_str().unwrap_or("");
                assert!(is_tools_method(s), "tools {s}");
            }
        }
        if let Some(vals) = doc.get("promptsMethods").and_then(|v| v.as_array()) {
            for v in vals {
                let s = v.as_str().unwrap_or("");
                assert!(is_prompts_method(s), "prompts {s}");
            }
        }
        if let Some(cases) = doc.get("uriSchemes").and_then(|v| v.as_array()) {
            for case in cases {
                let uri = case["uri"].as_str().unwrap_or("");
                let expected = case.get("scheme").and_then(|v| v.as_str());
                assert_eq!(uri_scheme(uri), expected, "scheme {uri}");
                let http = case["isHttpOrHttps"].as_bool().unwrap_or(false);
                assert_eq!(is_http_or_https_uri(uri), http, "http(s) {uri}");
            }
        }
        if let Some(cases) = doc.get("contentTypeStr").and_then(|v| v.as_array()) {
            for case in cases {
                let kind = case["kind"].as_str().unwrap_or("");
                let expected = case["type"].as_str().unwrap_or("");
                let c = match kind {
                    "text" => Content::text("x"),
                    "image" => Content::image("AA", "image/png"),
                    "audio" => Content::audio("BB", "audio/wav"),
                    "resource" => embedded_resource_content("file:///x", None, Some("t".into())),
                    other => panic!("unknown content kind {other}"),
                };
                assert_eq!(content_type_str(&c), expected, "content type {kind}");
                let has_text = case["hasTextBody"].as_bool().unwrap_or(false);
                assert_eq!(content_text_body(&c).is_some(), has_text, "text body {kind}");
            }
        }
        if let Some(cases) = doc.get("toolResultAllText").and_then(|v| v.as_array()) {
            for case in cases {
                let name = case["name"].as_str().unwrap_or("?");
                let result = case
                    .get("result")
                    .cloned()
                    .unwrap_or_else(|| serde_json::json!({}));
                let len = case["contentLen"].as_u64().unwrap_or(0) as usize;
                assert_eq!(tool_result_content_len(&result), len, "len {name}");
                let got = tool_result_all_text(&result);
                if let Some(t) = case.get("expectedText").and_then(|v| v.as_str()) {
                    assert_eq!(got.as_deref(), Some(t), "all text {name}");
                } else {
                    assert!(got.is_none(), "all text none {name}");
                }
            }
        }
        if let Some(cases) = doc.get("paramsExtract").and_then(|v| v.as_array()) {
            for case in cases {
                let name = case["name"].as_str().unwrap_or("?");
                let params = case
                    .get("params")
                    .cloned()
                    .unwrap_or_else(|| serde_json::json!({}));
                if let Some(tn) = case.get("toolName").and_then(|v| v.as_str()) {
                    assert_eq!(tools_call_name(&params), Some(tn), "tool name {name}");
                } else {
                    assert!(tools_call_name(&params).is_none(), "tool name none {name}");
                }
                if let Some(u) = case.get("uri").and_then(|v| v.as_str()) {
                    assert_eq!(params_uri(&params), Some(u), "uri {name}");
                } else {
                    assert!(params_uri(&params).is_none(), "uri none {name}");
                }
            }
        }
        if let Some(cases) = doc.get("listResultLen").and_then(|v| v.as_array()) {
            for case in cases {
                let name = case["name"].as_str().unwrap_or("?");
                let result = case
                    .get("result")
                    .cloned()
                    .unwrap_or_else(|| serde_json::json!({}));
                let key = case["key"].as_str().unwrap_or("tools");
                let len = case["len"].as_u64().unwrap_or(0) as usize;
                assert_eq!(list_result_len(&result, key), len, "list len {name}");
            }
        }
        if let Some(case) = doc.get("initializeExtract") {
            let init = initialize_result(
                case["protocolVersion"].as_str().unwrap_or(""),
                case["serverName"].as_str().unwrap_or(""),
                case["serverVersion"].as_str().unwrap_or(""),
                empty_server_capabilities(),
                None,
            );
            assert_eq!(
                initialize_result_protocol_version(&init),
                case["protocolVersion"].as_str()
            );
            assert_eq!(
                initialize_result_server_name(&init),
                case["serverName"].as_str()
            );
            let meta = meta_with_progress_token(serde_json::json!("tok"));
            assert_eq!(
                progress_token_from_meta(&meta),
                Some(&serde_json::json!("tok"))
            );
        }
    }

    #[test]
    fn wave10_notification_to_jsonrpc_methods() {
        let wire = notification_to_jsonrpc(&tools_list_changed());
        assert_eq!(wire.method, methods::TOOLS_LIST_CHANGED);
        assert!(wire.params.is_none());

        let wire = notification_to_jsonrpc(&resource_updated("file:///a"));
        assert_eq!(wire.method, methods::RESOURCES_UPDATED);
        assert_eq!(
            wire.params.as_ref().and_then(|p| p.get("uri")).and_then(|u| u.as_str()),
            Some("file:///a")
        );

        let wire = notification_to_jsonrpc(&progress_notification(
            serde_json::json!("tok"),
            3.0,
            Some(10.0),
            Some("working".into()),
        ));
        assert_eq!(wire.method, methods::PROGRESS_NOTIFICATION);
        let p = match wire.params.as_ref() {
            Some(p) => p,
            None => panic!("expected params"),
        };
        assert_eq!(p["progressToken"], "tok");
        assert_eq!(p["progress"], 3.0);
        assert_eq!(p["total"], 10.0);
        assert_eq!(p["message"], "working");

        let wire = notification_to_jsonrpc(&log_notification(
            "warning",
            serde_json::json!({"x": 1}),
            Some("svc".into()),
        ));
        assert_eq!(wire.method, methods::LOG_MESSAGE);
        assert_eq!(
            wire.params.as_ref().and_then(|p| p.get("level")).and_then(|l| l.as_str()),
            Some("warning")
        );

        let wire = notification_to_jsonrpc(&cancelled(serde_json::json!(42), Some("bye".into())));
        assert_eq!(wire.method, methods::CANCELLED_NOTIFICATION);
        assert_eq!(
            wire.params.as_ref().and_then(|p| p.get("requestId")),
            Some(&serde_json::json!(42))
        );
    }

    #[test]
    fn wave10_normalize_tool_result_shapes() {
        let full = serde_json::json!({
            "content": [{"type": "text", "text": "a"}],
            "isError": false
        });
        let n = normalize_tool_result(&full);
        assert_eq!(array_len(&n, "content"), 1);
        assert_eq!(n["isError"], false);

        let arr = serde_json::json!([
            {"type": "text", "text": "a"},
            {"type": "text", "text": "b"}
        ]);
        let n = normalize_tool_result(&arr);
        assert_eq!(array_len(&n, "content"), 2);

        let single = serde_json::json!({"type": "text", "text": "solo"});
        let n = normalize_tool_result(&single);
        assert_eq!(array_len(&n, "content"), 1);
        assert_eq!(n["content"][0]["text"], "solo");

        // structuredContent on full result must be preserved
        let with_sc = serde_json::json!({
            "content": [{"type": "text", "text": "x"}],
            "structuredContent": {"k": 1}
        });
        let n = normalize_tool_result(&with_sc);
        assert_eq!(n["structuredContent"]["k"], 1);
    }

    #[test]
    fn wave10_initialize_and_list_envelopes() {
        let init = initialize_result(
            LATEST_PROTOCOL_VERSION,
            "test-server",
            "0.1.0",
            empty_server_capabilities(),
            Some("hi".into()),
        );
        assert_eq!(init["protocolVersion"], LATEST_PROTOCOL_VERSION);
        assert_eq!(init["serverInfo"]["name"], "test-server");
        assert_eq!(init["instructions"], "hi");

        let tpl = resource_templates_list_result(
            &[serde_json::json!({"uriTemplate": "file:///{p}", "name": "f"})],
            Some("c".into()),
        );
        assert_eq!(array_len(&tpl, "resourceTemplates"), 1);
        assert_eq!(tpl["nextCursor"], "c");

        let roots = roots_list_result(&[serde_json::json!({"uri": "file:///"})]);
        assert_eq!(array_len(&roots, "roots"), 1);

        let comp = completion_complete_result(&["a".into(), "b".into()], Some(2), Some(false));
        assert_eq!(comp["completion"]["values"][0], "a");
        assert_eq!(comp["completion"]["total"], 2);
        assert_eq!(comp["completion"]["hasMore"], false);

        assert!(is_valid_log_level("error"));
        assert!(!is_valid_log_level("verbose"));

        let tool = protocol_tool(
            "ping",
            Some("p".into()),
            serde_json::json!({"type": "object"}),
            None,
        );
        assert_eq!(tool.name, "ping");
        let body = tools_list_result(&[tool], None);
        assert_eq!(body["tools"][0]["name"], "ping");
    }

    #[test]
    fn wave11_protocol_descriptors_and_templates() {
        let r = protocol_resource(
            "readme",
            "file:///README.md",
            Some("docs".into()),
            Some("text/markdown".into()),
        );
        assert_eq!(r.name, "readme");
        assert_eq!(r.uri, "file:///README.md");
        let list = resources_list_result(
            &[serde_json::to_value(&r).unwrap_or_else(|_| serde_json::json!({}))],
            None,
        );
        assert_eq!(list["resources"][0]["name"], "readme");

        let t = protocol_template(
            "file",
            "file:///{path}",
            Some("f".into()),
            Some("text/plain".into()),
        );
        assert_eq!(t.uri_template, "file:///{path}");
        assert!(matches_template(&t.uri_template, "file:///a.txt"));
        assert!(!matches_template(&t.uri_template, "file:///a/b"));

        let p = protocol_prompt(
            "greet",
            Some("say hi".into()),
            Some(vec![PromptArgument {
                name: "who".into(),
                description: Some("name".into()),
                required: Some(true),
            }]),
        );
        assert_eq!(p.name, "greet");
        assert_eq!(
            p.arguments
                .as_ref()
                .and_then(|a| a.first())
                .map(|a| a.name.as_str()),
            Some("who")
        );

        let root = protocol_root("file:///", Some("workspace".into()));
        assert_eq!(root.uri, "file:///");
        assert_eq!(root.name.as_deref(), Some("workspace"));
    }

    #[test]
    fn wave11_resource_contents_wire_shape() {
        let item = resource_text("file:///a", "hello", None);
        assert_eq!(item.kind.as_deref(), Some("resource"));
        assert_eq!(item.mime_type.as_deref(), Some("text/plain"));
        let full = resource_text_result("file:///a", "hello", None);
        assert_eq!(full["contents"][0]["type"], "resource");
        assert_eq!(full["contents"][0]["text"], "hello");

        let blob = resource_blob("file:///b", "AAAA", "image/png");
        assert_eq!(blob["contents"][0]["type"], "resource");
        assert_eq!(blob["contents"][0]["blob"], "AAAA");
        assert_eq!(blob["contents"][0]["mimeType"], "image/png");

        let multi = resource_contents(&[
            resource_text("file:///1", "one", None),
            EmbeddedResource::blob_item("file:///2", "BB", "application/octet-stream"),
        ]);
        assert_eq!(array_len(&multi, "contents"), 2);
    }

    #[test]
    fn wave11_sampling_elicitation_envelopes() {
        let params = sampling_create_params(
            &[serde_json::json!({"role": "user", "content": {"type": "text", "text": "hi"}})],
            128,
            Some("sys".into()),
            Some(0.2),
        );
        assert_eq!(params["maxTokens"], 128);
        assert_eq!(params["systemPrompt"], "sys");
        assert_eq!(params["temperature"], 0.2);
        assert_eq!(array_len(&params, "messages"), 1);

        let result = sampling_create_result(
            "assistant",
            Content::text("yo"),
            "test-model",
            Some("endTurn".into()),
        );
        assert_eq!(result["role"], "assistant");
        assert_eq!(result["model"], "test-model");
        assert_eq!(result["stopReason"], "endTurn");
        assert_eq!(result["content"]["type"], "text");

        let ep = elicitation_create_params(
            "Name?",
            serde_json::json!({
                "type": "object",
                "properties": { "name": { "type": "string" } }
            }),
        );
        assert_eq!(ep["message"], "Name?");
        assert_eq!(ep["requestedSchema"]["type"], "object");

        let er = elicitation_create_result("accept", Some(serde_json::json!({"name": "Ada"})));
        assert_eq!(er["action"], "accept");
        assert_eq!(er["content"]["name"], "Ada");
        assert!(is_valid_elicitation_action("accept"));
        assert!(is_valid_elicitation_action("decline"));
        assert!(is_valid_elicitation_action("cancel"));
        assert!(!is_valid_elicitation_action("maybe"));
    }

    #[test]
    fn wave11_method_catalog_and_capabilities() {
        assert!(is_mcp_method(methods::TOOLS_CALL));
        assert!(is_mcp_method(methods::SAMPLING_CREATE_MESSAGE));
        assert!(is_mcp_method(methods::ELICITATION_CREATE));
        assert!(is_mcp_method(methods::RESOURCES_TEMPLATES_LIST));
        assert!(!is_mcp_method("tools/unknown"));
        assert_eq!(methods::ALL.len(), 25);

        let caps = server_capabilities(
            Some(true),
            Some(true),
            Some(true),
            Some(false),
            true,
            true,
        );
        assert_eq!(caps["tools"]["listChanged"], true);
        assert_eq!(caps["resources"]["subscribe"], true);
        assert_eq!(caps["resources"]["listChanged"], true);
        assert_eq!(caps["prompts"]["listChanged"], false);
        assert!(caps.get("logging").is_some());
        assert!(caps.get("completions").is_some());

        assert_eq!(wave26_ping_result_empty(), serde_json::json!({}));

        let with_sc = tool_result_with_structured(
            &[Content::text("ok")],
            false,
            Some(serde_json::json!({"score": 1})),
        );
        assert_eq!(with_sc["structuredContent"]["score"], 1);
        let n = normalize_tool_result(&with_sc);
        assert_eq!(n["structuredContent"]["score"], 1);
    }

    #[test]
    fn wave12_request_param_envelopes_and_method_kind() {
        assert!(is_mcp_request_method(methods::TOOLS_CALL));
        assert!(is_mcp_request_method(methods::INITIALIZE));
        assert!(is_mcp_request_method(methods::PING));
        assert!(!is_mcp_request_method(methods::TOOLS_LIST_CHANGED));
        assert!(is_mcp_notification_method(methods::TOOLS_LIST_CHANGED));
        assert!(is_mcp_notification_method(methods::INITIALIZED));
        assert!(!is_mcp_notification_method(methods::TOOLS_CALL));
        assert!(!is_mcp_request_method("not/a/method"));
        assert!(!is_mcp_notification_method("not/a/method"));

        let caps = client_capabilities(Some(true), true, true);
        assert_eq!(caps["roots"]["listChanged"], true);
        assert!(caps.get("sampling").is_some());
        assert!(caps.get("elicitation").is_some());

        let init = initialize_params(
            LATEST_PROTOCOL_VERSION,
            "test-client",
            "1.0.0",
            caps,
        );
        assert_eq!(init["protocolVersion"], LATEST_PROTOCOL_VERSION);
        assert_eq!(init["clientInfo"]["name"], "test-client");
        assert_eq!(init["clientInfo"]["version"], "1.0.0");

        let info = implementation_info("svc", "0.2.0");
        assert_eq!(info["name"], "svc");
        assert_eq!(info["version"], "0.2.0");

        let lp = list_params(Some("c1".into()));
        assert_eq!(lp["cursor"], "c1");
        let lp_empty = list_params(None);
        assert!(lp_empty.as_object().is_some_and(|m| m.is_empty()));

        let call = tools_call_params("ping", Some(serde_json::json!({"x": 1})));
        assert_eq!(call["name"], "ping");
        assert_eq!(call["arguments"]["x"], 1);

        let read = resources_read_params("file:///a");
        assert_eq!(read["uri"], "file:///a");
        assert_eq!(
            resources_subscribe_params("file:///a")["uri"],
            "file:///a"
        );
        assert_eq!(
            resources_unsubscribe_params("file:///a")["uri"],
            "file:///a"
        );

        let mut args = std::collections::BTreeMap::new();
        args.insert("who".into(), "Ada".into());
        let get = prompts_get_params("greet", Some(args));
        assert_eq!(get["name"], "greet");
        assert_eq!(get["arguments"]["who"], "Ada");

        let logp = logging_set_level_params("warning");
        assert_eq!(logp["level"], "warning");

        let comp = completion_complete_params(
            "ref/prompt",
            Some("greet".into()),
            None,
            "who",
            "A",
        );
        assert_eq!(comp["ref"]["type"], "ref/prompt");
        assert_eq!(comp["ref"]["name"], "greet");
        assert_eq!(comp["argument"]["name"], "who");
        assert_eq!(comp["argument"]["value"], "A");

        let pp = progress_params(serde_json::json!("t"), 0.5, Some(1.0), Some("half".into()));
        assert_eq!(pp["progressToken"], "t");
        assert_eq!(pp["progress"], 0.5);
        assert_eq!(pp["total"], 1.0);
        assert_eq!(pp["message"], "half");

        let sm = sampling_message("user", text_content("hi"));
        assert_eq!(sm["role"], "user");
        assert_eq!(sm["content"]["type"], "text");
        assert_eq!(sm["content"]["text"], "hi");

        let t = text_content("x");
        assert!(t.is_text());
        assert!(!t.is_image());
        let img = image_content("AA", "image/png");
        assert!(img.is_image());
        assert!(!img.is_audio());
        let aud = audio_content("BB", "audio/wav");
        assert!(aud.is_audio());
        let emb = embedded_resource_content("file:///x", None, Some("z".into()));
        assert!(emb.is_resource());
    }


    #[test]
    fn wave13_cancelled_meta_domains_and_sampling_prefs() {
        let c = cancelled_params(serde_json::json!(9), Some("user abort".into()));
        assert_eq!(c["requestId"], 9);
        assert_eq!(c["reason"], "user abort");
        let c2 = cancelled_params(serde_json::json!("r1"), None);
        assert!(c2.get("reason").is_none());

        let p = tools_call_params_with_progress(
            "ping",
            Some(serde_json::json!({"n": 1})),
            Some(serde_json::json!("tok-1")),
        );
        assert_eq!(p["name"], "ping");
        assert_eq!(p["_meta"]["progressToken"], "tok-1");

        let hint = model_hint(Some("gpt".into()));
        assert_eq!(hint["name"], "gpt");
        let prefs = model_preferences(Some(0.2), Some(0.5), Some(0.8), Some(&[hint]));
        assert_eq!(prefs["costPriority"], 0.2);
        assert_eq!(prefs["speedPriority"], 0.5);
        assert_eq!(prefs["intelligencePriority"], 0.8);
        assert_eq!(prefs["hints"][0]["name"], "gpt");

        assert!(is_valid_include_context("none"));
        assert!(is_valid_include_context("thisServer"));
        assert!(is_valid_include_context("allServers"));
        assert!(!is_valid_include_context("other"));

        assert!(is_canonical_stop_reason("endTurn"));
        assert!(is_canonical_stop_reason("stopSequence"));
        assert!(is_canonical_stop_reason("maxTokens"));
        assert!(!is_canonical_stop_reason("custom"));

        let ann = content_annotations(Some(vec!["user".into()]), Some(0.9));
        assert_eq!(ann.priority, Some(0.9));
        let t = Content::text("x").with_annotations(ann);
        assert!(t.is_text());

        let le = log_entry("info", Some(serde_json::json!({"ok": true})), Some("svc".into()));
        assert_eq!(le["level"], "info");
        assert_eq!(le["logger"], "svc");

        assert!(is_lifecycle_method(methods::INITIALIZE));
        assert!(is_lifecycle_method(methods::PING));
        assert!(!is_lifecycle_method(methods::TOOLS_CALL));

        assert_eq!(method_domain(methods::INITIALIZE), "lifecycle");
        assert_eq!(method_domain(methods::TOOLS_CALL), "tools");
        assert_eq!(method_domain(methods::TOOLS_LIST_CHANGED), "tools");
        assert_eq!(method_domain(methods::RESOURCES_READ), "resources");
        assert_eq!(method_domain(methods::PROMPTS_GET), "prompts");
        assert_eq!(method_domain(methods::LOG_MESSAGE), "logging");
        assert_eq!(method_domain(methods::COMPLETION_COMPLETE), "completion");
        assert_eq!(method_domain(methods::SAMPLING_CREATE_MESSAGE), "sampling");
        assert_eq!(method_domain(methods::ELICITATION_CREATE), "elicitation");
        assert_eq!(method_domain(methods::PROGRESS_NOTIFICATION), "progress");
        assert_eq!(method_domain(methods::CANCELLED_NOTIFICATION), "cancellation");
        assert_eq!(method_domain(methods::ROOTS_LIST), "roots");
        assert_eq!(method_domain("not/real"), "unknown");

        let ext = sampling_create_params_ext(
            &[serde_json::json!({"role": "user", "content": {"type": "text", "text": "hi"}})],
            32,
            Some("sys".into()),
            Some(0.7),
            Some("thisServer".into()),
            Some(model_preferences(None, Some(1.0), None, None)),
        );
        assert_eq!(ext["maxTokens"], 32);
        assert_eq!(ext["includeContext"], "thisServer");
        assert_eq!(ext["modelPreferences"]["speedPriority"], 1.0);
        assert_eq!(ext["systemPrompt"], "sys");

        let rc = resource_content(EmbeddedResource::text_item("file:///a", "body", None));
        assert!(rc.is_resource());
        assert_eq!(empty_client_capabilities(), serde_json::json!({}));
    }

    #[test]
    fn wave14_catalogs_extractors_and_roots_list_changed() {
        assert!(is_valid_message_role("user"));
        assert!(is_valid_message_role("assistant"));
        assert!(!is_valid_message_role("system"));

        assert!(is_valid_content_type("text"));
        assert!(is_valid_content_type("image"));
        assert!(is_valid_content_type("audio"));
        assert!(is_valid_content_type("resource"));
        assert!(!is_valid_content_type("video"));

        assert!(is_valid_audience("user"));
        assert!(is_valid_audience("assistant"));
        assert!(!is_valid_audience("system"));

        assert!(is_server_to_client_method(methods::SAMPLING_CREATE_MESSAGE));
        assert!(is_server_to_client_method(methods::ELICITATION_CREATE));
        assert!(!is_server_to_client_method(methods::TOOLS_CALL));
        assert!(!is_server_to_client_method(methods::INITIALIZE));

        assert!(is_valid_completion_ref_type("ref/prompt"));
        assert!(is_valid_completion_ref_type("ref/resource"));
        assert!(!is_valid_completion_ref_type("ref/tool"));

        assert!(is_valid_elicitation_property_type("string"));
        assert!(is_valid_elicitation_property_type("integer"));
        assert!(!is_valid_elicitation_property_type("object"));

        let ann = tool_annotations(
            Some("Read file".into()),
            Some(true),
            Some(false),
            Some(true),
            Some(false),
        );
        assert_eq!(ann.title.as_deref(), Some("Read file"));
        assert_eq!(ann.read_only_hint, Some(true));
        assert_eq!(ann.destructive_hint, Some(false));

        let arg = prompt_argument("who", Some("person".into()), Some(true));
        assert_eq!(arg.name, "who");
        assert_eq!(arg.required, Some(true));

        let r = completion_ref("ref/prompt", Some("greet".into()), None);
        assert_eq!(r["type"], "ref/prompt");
        assert_eq!(r["name"], "greet");
        let r2 = completion_ref("ref/resource", None, Some("file:///{p}".into()));
        assert_eq!(r2["type"], "ref/resource");
        assert_eq!(r2["uri"], "file:///{p}");

        let call = tools_call_params_with_progress(
            "ping",
            None,
            Some(serde_json::json!("tok-wave14")),
        );
        assert_eq!(
            progress_token_from_call_params(&call),
            Some(&serde_json::json!("tok-wave14"))
        );
        let no_meta = tools_call_params("ping", None);
        assert!(progress_token_from_call_params(&no_meta).is_none());

        assert_eq!(log_level_index("debug"), Some(0));
        assert_eq!(log_level_index("emergency"), Some(7));
        assert!(log_level_at_least("error", "warning"));
        assert!(log_level_at_least("error", "error"));
        assert!(!log_level_at_least("info", "error"));
        assert!(!log_level_at_least("nope", "info"));

        assert!(is_list_changed_notification(methods::TOOLS_LIST_CHANGED));
        assert!(is_list_changed_notification(methods::ROOTS_LIST_CHANGED));
        assert!(!is_list_changed_notification(methods::RESOURCES_UPDATED));

        let wire = notification_to_jsonrpc(&roots_list_changed());
        assert_eq!(wire.method, methods::ROOTS_LIST_CHANGED);
        assert!(wire.params.is_none());
    }

    #[test]
    fn wave15_direction_list_cursor_priority_and_extractors() {
        assert!(is_client_to_server_request_method(methods::TOOLS_CALL));
        assert!(is_client_to_server_request_method(methods::INITIALIZE));
        assert!(!is_client_to_server_request_method(
            methods::SAMPLING_CREATE_MESSAGE
        ));
        assert!(!is_client_to_server_request_method(
            methods::TOOLS_LIST_CHANGED
        ));

        assert_eq!(method_direction(methods::TOOLS_CALL), "client_to_server");
        assert_eq!(
            method_direction(methods::SAMPLING_CREATE_MESSAGE),
            "server_to_client"
        );
        assert_eq!(
            method_direction(methods::TOOLS_LIST_CHANGED),
            "notification"
        );
        assert_eq!(method_direction("nope/method"), "unknown");

        assert_eq!(NOTIFICATION_METHODS.len(), 9);
        for m in NOTIFICATION_METHODS {
            assert!(is_mcp_notification_method(m), "{m}");
        }

        let meta = serde_json::json!({"progressToken": "m-tok"});
        assert_eq!(
            progress_token_from_meta(&meta),
            Some(&serde_json::json!("m-tok"))
        );
        assert!(progress_token_from_meta(&serde_json::json!({})).is_none());

        let with_cursor = serde_json::json!({"tools": [], "nextCursor": "abc"});
        assert_eq!(list_next_cursor(&with_cursor), Some("abc"));
        assert!(list_has_next_cursor(&with_cursor));
        let no_cursor = serde_json::json!({"tools": []});
        assert!(list_next_cursor(&no_cursor).is_none());
        assert!(!list_has_next_cursor(&no_cursor));
        let empty_cursor = serde_json::json!({"nextCursor": ""});
        assert!(!list_has_next_cursor(&empty_cursor));

        assert!(is_valid_priority(0.0));
        assert!(is_valid_priority(0.5));
        assert!(is_valid_priority(1.0));
        assert!(!is_valid_priority(-0.1));
        assert!(!is_valid_priority(1.1));
        assert!(!is_valid_priority(f64::NAN));
        assert_eq!(clamp_priority(-1.0), 0.0);
        assert_eq!(clamp_priority(2.0), 1.0);
        assert_eq!(clamp_priority(0.3), 0.3);
        assert_eq!(clamp_priority(f64::INFINITY), 0.0);

        let arg = completion_argument("prefix", "hel");
        assert_eq!(arg["name"], "prefix");
        assert_eq!(arg["value"], "hel");

        let ok = tool_text_result("hello", false);
        assert_eq!(tool_result_first_text(&ok), Some("hello"));
        let img = serde_json::json!({
            "content": [{"type": "image", "data": "AA", "mimeType": "image/png"}],
            "isError": false
        });
        assert!(tool_result_first_text(&img).is_none());
        assert!(tool_result_first_text(&serde_json::json!({})).is_none());

        assert!(is_file_uri("file:///tmp"));
        assert!(is_file_uri("file://localhost/tmp"));
        assert!(!is_file_uri("https://example.com"));
        assert!(!is_file_uri(""));
    }

    #[test]
    fn wave16_domain_catalogs_content_uri_and_extractors() {
        assert!(is_resources_method(methods::RESOURCES_READ));
        assert!(is_resources_method(methods::RESOURCES_LIST_CHANGED));
        assert!(!is_resources_method(methods::TOOLS_CALL));
        assert!(is_tools_method(methods::TOOLS_CALL));
        assert!(is_tools_method(methods::TOOLS_LIST_CHANGED));
        assert!(!is_tools_method(methods::PROMPTS_GET));
        assert!(is_prompts_method(methods::PROMPTS_LIST));
        assert!(is_prompts_method(methods::PROMPTS_LIST_CHANGED));
        assert!(!is_prompts_method(methods::PING));
        assert_eq!(RESOURCES_METHODS.len(), 7);
        assert_eq!(TOOLS_METHODS.len(), 3);
        assert_eq!(PROMPTS_METHODS.len(), 3);
        for m in RESOURCES_METHODS {
            assert_eq!(method_domain(m), "resources", "{m}");
        }
        for m in TOOLS_METHODS {
            assert_eq!(method_domain(m), "tools", "{m}");
        }
        for m in PROMPTS_METHODS {
            assert_eq!(method_domain(m), "prompts", "{m}");
        }

        let text = Content::text("hello");
        assert_eq!(content_type_str(&text), "text");
        assert_eq!(content_text_body(&text), Some("hello"));
        let img = Content::image("AA", "image/png");
        assert_eq!(content_type_str(&img), "image");
        assert!(content_text_body(&img).is_none());
        let aud = Content::audio("BB", "audio/wav");
        assert_eq!(content_type_str(&aud), "audio");
        let res = embedded_resource_content("file:///x", None, Some("body".into()));
        assert_eq!(content_type_str(&res), "resource");

        let multi = serde_json::json!({
            "content": [
                {"type": "text", "text": "a"},
                {"type": "image", "data": "AA", "mimeType": "image/png"},
                {"type": "text", "text": "b"}
            ],
            "isError": false
        });
        assert_eq!(tool_result_content_len(&multi), 3);
        assert_eq!(tool_result_all_text(&multi).as_deref(), Some("ab"));
        assert_eq!(
            tool_result_first_text(&multi),
            Some("a")
        );
        assert_eq!(tool_result_content_len(&serde_json::json!({})), 0);
        assert!(tool_result_all_text(&serde_json::json!({"content": []})).is_none());

        let call = tools_call_params("ping", Some(serde_json::json!({})));
        assert_eq!(tools_call_name(&call), Some("ping"));
        assert!(tools_call_name(&serde_json::json!({})).is_none());

        let read = resources_read_params("file:///tmp/a");
        assert_eq!(params_uri(&read), Some("file:///tmp/a"));
        assert!(params_uri(&serde_json::json!({})).is_none());

        let list = tools_list_result(
            &[Tool {
                name: "a".into(),
                title: None,
                description: None,
                input_schema: serde_json::json!({"type": "object"}),
                output_schema: None,
                annotations: None,
            }],
            Some("c1".into()),
        );
        assert_eq!(list_result_len(&list, "tools"), 1);
        assert!(list_result_array(&list, "tools").is_some());
        assert_eq!(list_result_len(&list, "resources"), 0);

        assert_eq!(uri_scheme("file:///tmp"), Some("file"));
        assert_eq!(uri_scheme("https://example.com/x"), Some("https"));
        assert_eq!(uri_scheme("http://localhost"), Some("http"));
        assert!(uri_scheme("noscheme").is_none());
        assert!(uri_scheme(":bad").is_none());
        assert!(is_http_or_https_uri("https://example.com"));
        assert!(is_http_or_https_uri("http://example.com"));
        assert!(!is_http_or_https_uri("file:///tmp"));
        assert!(!is_http_or_https_uri("ftp://x"));

        let meta = meta_with_progress_token(serde_json::json!(42));
        assert_eq!(meta["progressToken"], 42);
        assert_eq!(
            progress_token_from_meta(&meta),
            Some(&serde_json::json!(42))
        );

        let init = initialize_result(
            LATEST_PROTOCOL_VERSION,
            "demo",
            "1.0.0",
            empty_server_capabilities(),
            None,
        );
        assert_eq!(
            initialize_result_protocol_version(&init),
            Some(LATEST_PROTOCOL_VERSION)
        );
        assert_eq!(initialize_result_server_name(&init), Some("demo"));
        assert!(initialize_result_server_name(&serde_json::json!({})).is_none());
    }

    #[test]
    fn wave17_prompt_list_template_and_role_extractors() {
        let msgs = [user_message("hi"), assistant_message("yo")];
        let pr = prompt_result("desc", &msgs);
        assert_eq!(prompt_result_description(&pr), Some("desc"));
        assert_eq!(prompt_result_message_count(&pr), 2);
        assert_eq!(prompt_result_first_role(&pr), Some("user"));
        assert!(is_valid_message_role("user"));
        assert!(is_valid_message_role("assistant"));
        assert!(!is_valid_message_role("system"));
        assert_eq!(normalize_message_role(" User "), Some("user"));
        assert_eq!(normalize_message_role("ASSISTANT"), Some("assistant"));
        assert!(normalize_message_role("system").is_none());

        let tools = tools_list_result(
            &[Tool {
                name: "ping".into(),
                title: None,
                description: None,
                input_schema: serde_json::json!({"type": "object"}),
                output_schema: None,
                annotations: None,
            }],
            None,
        );
        assert_eq!(tool_names_from_list(&tools), vec!["ping".to_string()]);
        assert!(tool_names_from_list(&serde_json::json!({})).is_empty());

        let structured = serde_json::json!({
            "content": [{"type":"text","text":"x"}],
            "structuredContent": {"ok": true},
            "isError": false
        });
        assert!(tool_result_has_structured(&structured));
        assert!(!tool_result_has_structured(&serde_json::json!({"content":[]})));
        assert!(content_array_nonempty(&structured));
        assert!(!content_array_nonempty(&serde_json::json!({"content":[]})));

        let res = protocol_resource("a", "file:///a", None, None);
        let res_wire = match serde_json::to_value(&res) {
            Ok(v) => v,
            Err(e) => panic!("resource wire: {e}"),
        };
        let resources = resources_list_result(
            &[res_wire],
            None,
        );
        assert_eq!(
            resource_uris_from_list(&resources),
            vec!["file:///a".to_string()]
        );

        let log_params = logging_set_level_params("info");
        assert_eq!(logging_level_from_params(&log_params), Some("info"));
        assert!(logging_level_from_params(&serde_json::json!({})).is_none());

        let mut params = std::collections::BTreeMap::new();
        params.insert("name".into(), "demo".into());
        assert_eq!(
            apply_template_params("file:///{name}/x", &params).as_deref(),
            Some("file:///demo/x")
        );
        assert!(apply_template_params(
            "file:///{name}/x",
            &std::collections::BTreeMap::new()
        )
        .is_none());
        let mut bad = std::collections::BTreeMap::new();
        bad.insert("name".into(), "a/b".into());
        assert!(apply_template_params("file:///{name}/x", &bad).is_none());
    }

    #[test]
    fn wave18_list_progress_and_content_extractors() {
        let prompts = serde_json::json!({
            "prompts": [{"name": "p1"}, {"name": "p2"}],
            "nextCursor": "c1"
        });
        assert_eq!(
            prompt_names_from_list(&prompts),
            vec!["p1".to_string(), "p2".to_string()]
        );
        assert!(list_has_next_cursor(&prompts));
        assert!(!list_has_next_cursor(&serde_json::json!({"nextCursor": ""})));
        assert!(!list_has_next_cursor(&serde_json::json!({})));

        let resources = serde_json::json!({
            "resources": [
                {"name": "r1", "uri": "file:///a"},
                {"name": "r2", "uri": "file:///b"}
            ]
        });
        assert_eq!(
            resource_names_from_list(&resources),
            vec!["r1".to_string(), "r2".to_string()]
        );

        let tools = tools_list_result(
            &[Tool {
                name: "ping".into(),
                title: None,
                description: None,
                input_schema: serde_json::json!({"type": "object"}),
                output_schema: None,
                annotations: None,
            }],
            Some("n".into()),
        );
        assert_eq!(tool_count_from_list(&tools), 1);
        assert!(list_has_next_cursor(&tools));

        assert_eq!(normalize_protocol_version("  2025-03-26  "), Some("2025-03-26"));
        assert!(normalize_protocol_version("   ").is_none());
        assert!(is_ping_method("ping"));
        assert!(!is_ping_method("Ping"));

        let ok = serde_json::json!({
            "content": [{"type": "text", "text": "hi"}],
            "isError": false
        });
        assert_eq!(tool_result_first_content_type(&ok), Some("text"));
        assert!(!tool_result_is_error(&ok));
        let err = tool_error("boom");
        assert!(tool_result_is_error(&err));
        assert_eq!(content_type_of(&serde_json::json!({"type": "image"})), Some("image"));

        let prog = serde_json::json!({
            "progressToken": "t1",
            "progress": 0.5,
            "total": 1.0
        });
        assert_eq!(
            progress_token_from_params(&prog),
            Some(serde_json::json!("t1"))
        );
        assert_eq!(progress_value_from_params(&prog), Some(0.5));
        assert!(progress_value_from_params(&serde_json::json!({})).is_none());
    }

    #[test]
    fn wave19_resource_tool_prompt_initialize_extractors() {
        let res = serde_json::json!({"uri":"file:///a","name":"a","mimeType":"text/plain"});
        assert_eq!(resource_mime_type(&res), Some("text/plain"));
        assert!(resource_mime_type(&serde_json::json!({"uri":"x"})).is_none());

        let tool = serde_json::json!({"name":"ping","title":"Ping tool","inputSchema":{"type":"object"}});
        assert_eq!(tool_title_of(&tool), Some("Ping tool"));
        assert!(tool_title_of(&serde_json::json!({"name":"x"})).is_none());

        let prompt = serde_json::json!({
            "name":"greet",
            "arguments":[{"name":"who","required":true},{"name":"tone"}]
        });
        assert_eq!(prompt_argument_names(&prompt), vec!["who".to_string(), "tone".to_string()]);
        assert!(prompt_argument_names(&serde_json::json!({"name":"x"})).is_empty());

        let with_ann = serde_json::json!({"type":"text","text":"hi","annotations":{"priority":0.5}});
        assert!(content_has_annotations(&with_ann));
        assert!(!content_has_annotations(&serde_json::json!({"type":"text","text":"hi"})));
        assert!(!content_has_annotations(&serde_json::json!({"type":"text","text":"hi","annotations":{}})));

        let init = serde_json::json!({
            "protocolVersion":"2025-03-26",
            "serverInfo":{"name":"demo","version":"1.2.3"},
            "capabilities":{}
        });
        assert_eq!(initialize_server_name(&init), Some("demo"));
        assert_eq!(initialize_server_version(&init), Some("1.2.3"));

        let tr = serde_json::json!({"content":[{"type":"text","text":"a"},{"type":"text","text":"b"}]});
        assert_eq!(tool_result_content_count(&tr), 2);
        assert_eq!(tool_result_content_count(&serde_json::json!({})), 0);

        assert!(is_logging_method(methods::LOGGING_SET_LEVEL));
        assert!(is_logging_method(methods::LOG_MESSAGE));
        assert!(!is_logging_method(methods::TOOLS_CALL));
    }

    #[test]
    fn wave20_descriptions_annotations_roots_and_call_params() {
        let res = serde_json::json!({"uri":"file:///a","name":"a","description":"Alpha"});
        assert_eq!(resource_description(&res), Some("Alpha"));
        assert!(resource_description(&serde_json::json!({"uri":"x"})).is_none());

        let tool = serde_json::json!({"name":"ping","description":"Ping tool"});
        assert_eq!(tool_description(&tool), Some("Ping tool"));

        let tmpl = serde_json::json!({"uriTemplate":"file:///{bucket}/{key}","name":"obj"});
        assert_eq!(
            resource_template_uri_template(&tmpl),
            Some("file:///{bucket}/{key}")
        );

        let c = serde_json::json!({
            "type":"text","text":"hi",
            "annotations":{"audience":["user","assistant"],"priority":0.75}
        });
        assert_eq!(
            content_annotation_audience(&c),
            vec!["user".to_string(), "assistant".to_string()]
        );
        assert_eq!(content_annotation_priority(&c), Some(0.75));
        assert!(content_annotation_audience(&serde_json::json!({"type":"text"})).is_empty());

        let init = serde_json::json!({
            "protocolVersion":"2025-03-26",
            "serverInfo":{"name":"demo","version":"1"},
            "instructions":"Use tools carefully"
        });
        assert_eq!(initialize_instructions(&init), Some("Use tools carefully"));

        assert!(is_cancellation_method(methods::CANCELLED_NOTIFICATION));
        assert!(!is_cancellation_method(methods::PING));

        let roots = serde_json::json!({"roots":[{"uri":"file:///a"},{"uri":"file:///b"}]});
        assert_eq!(
            root_uris_from_list(&roots),
            vec!["file:///a".to_string(), "file:///b".to_string()]
        );

        let call = serde_json::json!({"name":"echo","arguments":{"x":1}});
        assert_eq!(tool_name_from_call_params(&call), Some("echo"));
        assert!(tool_name_from_call_params(&serde_json::json!({})).is_none());
    }

    #[test]
    fn wave21_handler_pure_kernels() {
        let unk = unknown_tool_error_result("missing");
        assert_eq!(unk["isError"], true);
        assert_eq!(unk["content"][0]["text"], "Unknown tool: missing");
        assert_eq!(tool_error_message(&unk), Some("Unknown tool: missing"));

        let herr = tool_handler_error_result("boom");
        assert_eq!(herr["isError"], true);
        assert_eq!(herr["content"][0]["text"], "Tool error: boom");

        assert_eq!(
            unknown_method_error_message("nope/x"),
            "Unknown method: nope/x"
        );

        let caps_both = serde_json::json!({"sampling": {}, "elicitation": {}});
        assert!(client_has_sampling(&caps_both));
        assert!(client_has_elicitation(&caps_both));
        assert!(!client_has_sampling(&serde_json::json!({})));
        assert!(!client_has_elicitation(&serde_json::json!({"sampling": null})));

        let tools = tools_list_result(
            &[Tool {
                name: "a".into(),
                title: None,
                description: None,
                input_schema: serde_json::json!({"type": "object"}),
                output_schema: None,
                annotations: None,
            }],
            None,
        );
        assert_eq!(tool_count_from_list(&tools), 1);
        assert!(!tools_list_is_empty(&tools));
        assert!(tools_list_is_empty(&serde_json::json!({"tools": []})));

        let res_list = serde_json::json!({"resources": [{"uri": "file:///a"}, {"uri": "file:///b"}]});
        assert_eq!(resource_count_from_list(&res_list), 2);
        let prompts = serde_json::json!({"prompts": [{"name": "p"}]});
        assert_eq!(prompt_count_from_list(&prompts), 1);

        let init = serde_json::json!({
            "protocolVersion": "2025-03-26",
            "serverInfo": {"name": "s", "version": "1"},
            "capabilities": {},
            "instructions": "use carefully"
        });
        assert!(initialize_has_instructions(&init));
        assert!(!initialize_has_instructions(&serde_json::json!({
            "protocolVersion": "2025-03-26",
            "serverInfo": {"name": "s", "version": "1"},
            "capabilities": {}
        })));

        let progress = progress_notification_params(
            serde_json::json!("tok"),
            0.5,
            Some(1.0),
            Some("halfway".into()),
        );
        assert_eq!(progress["progressToken"], "tok");
        assert_eq!(progress["progress"], 0.5);
        assert_eq!(progress["total"], 1.0);
        assert_eq!(progress["message"], "halfway");

        let logp = log_notification_params("info", serde_json::json!({"ok": true}), Some("app".into()));
        assert_eq!(logp["level"], "info");
        assert_eq!(logp["logger"], "app");
        assert_eq!(logp["data"]["ok"], true);

        assert!(is_handler_request_method(methods::INITIALIZE));
        assert!(is_handler_request_method(methods::TOOLS_CALL));
        assert!(is_handler_request_method(methods::COMPLETION_COMPLETE));
        assert!(!is_handler_request_method(methods::SAMPLING_CREATE_MESSAGE));
        assert!(!is_handler_request_method(methods::INITIALIZED));

        assert_eq!(empty_object_result(), serde_json::json!({}));
        assert_eq!(ping_result(), empty_object_result());
    }


    #[test]
    fn wave22_call_list_capability_helpers() {
        let call = serde_json::json!({
            "name": "echo",
            "arguments": {"x": 1},
            "_meta": {"progressToken": "t1"}
        });
        assert_eq!(tool_name_from_call_params(&call), Some("echo"));
        assert_eq!(
            tool_arguments_from_call_params(&call).and_then(|a| a.get("x").and_then(|v| v.as_i64())),
            Some(1)
        );
        assert!(tools_call_has_progress_token(&call));
        assert!(!tools_call_has_progress_token(&serde_json::json!({"name": "x"})));

        assert_eq!(
            resource_uri_from_params(&serde_json::json!({"uri": "file:///a"})),
            Some("file:///a")
        );
        assert_eq!(
            prompt_name_from_get_params(&serde_json::json!({"name": "greet"})),
            Some("greet")
        );

        let empty_res = serde_json::json!({"resources": []});
        assert!(resources_list_is_empty(&empty_res));
        assert!(prompts_list_is_empty(&serde_json::json!({"prompts": []})));
        assert!(resource_templates_list_is_empty(
            &serde_json::json!({"resourceTemplates": []})
        ));
        assert_eq!(
            resource_template_count_from_list(&serde_json::json!({
                "resourceTemplates": [{"uriTemplate": "f://{id}"}]
            })),
            1
        );

        assert_eq!(
            log_level_from_set_params(&serde_json::json!({"level": "debug"})),
            Some("debug")
        );

        let caps = serde_json::json!({
            "roots": {"listChanged": true},
            "sampling": {},
            "elicitation": {}
        });
        assert_eq!(client_capability_flags(&caps), (true, true, true));
        assert_eq!(
            client_capability_flags(&serde_json::json!({})),
            (false, false, false)
        );

        assert!(is_server_to_client_request_method(methods::SAMPLING_CREATE_MESSAGE));
        assert!(is_server_to_client_request_method(methods::ELICITATION_CREATE));
        assert!(!is_server_to_client_request_method(methods::TOOLS_CALL));

        let ok = tool_text_success_result("hi");
        assert_eq!(ok["isError"], false);
        assert_eq!(ok["content"][0]["text"], "hi");

        assert_eq!(
            cursor_from_list_params(&serde_json::json!({"cursor": "  abc  "})),
            Some("abc")
        );
        assert!(cursor_from_list_params(&serde_json::json!({"cursor": "   "})).is_none());
        assert!(cursor_from_list_params(&serde_json::json!({})).is_none());
    }

    #[test]
    fn wave23_handler_subscribe_completion_dispatch() {
        assert!(tool_context_sampling_available(true, true));
        assert!(!tool_context_sampling_available(true, false));
        assert!(!tool_context_elicitation_available(false, true));
        assert!(tool_context_elicitation_available(true, true));

        assert_eq!(
            uri_from_subscribe_params(&serde_json::json!({"uri": " file:///a "})),
            Some("file:///a")
        );
        assert!(uri_from_subscribe_params(&serde_json::json!({"uri": "  "})).is_none());

        let (next, added) = subscription_add(&["file:///a".into()], "file:///b");
        assert!(added);
        assert_eq!(next.len(), 2);
        let (same, added2) = subscription_add(&next, "file:///a");
        assert!(!added2);
        assert_eq!(same.len(), 2);
        let (after, removed) = subscription_remove(&next, "file:///a");
        assert!(removed);
        assert_eq!(after, vec!["file:///b".to_string()]);
        let (_, removed2) = subscription_remove(&after, "file:///missing");
        assert!(!removed2);

        let uris = ["file:///docs/a.md", "file:///src/b.ts", "mem://c"];
        let vals = completion_values_from_resource_uris(&uris, "docs");
        assert_eq!(vals, vec!["file:///docs/a.md".to_string()]);
        assert!(resource_uri_matches_completion("file:///x", "x"));
        assert!(!resource_uri_matches_completion("file:///x", "y"));

        let body = basic_completion_result(&["a".into(), "b".into()]);
        assert_eq!(body["completion"]["values"][0], "a");
        assert_eq!(body["completion"]["hasMore"], false);

        let params = serde_json::json!({
            "ref": {"type": "ref/prompt", "name": "greet"},
            "argument": {"name": "who", "value": "Ad"}
        });
        assert_eq!(completion_ref_type(&params), Some("ref/prompt"));
        assert_eq!(completion_argument_value(&params), Some("Ad"));
        assert_eq!(completion_ref_prompt_name(&params), Some("greet"));
        assert!(completion_ref_prompt_name(&serde_json::json!({
            "ref": {"type": "ref/resource", "uri": "f://{id}"}
        }))
        .is_none());

        assert_eq!(
            dispatch_message_kind(true, false),
            DispatchMessageKind::Notification
        );
        assert_eq!(
            dispatch_message_kind(false, true),
            DispatchMessageKind::Request
        );
        assert_eq!(
            dispatch_message_kind(false, false),
            DispatchMessageKind::Other
        );
        assert!(is_initialized_notification_method(methods::INITIALIZED));
        assert!(!is_initialized_notification_method(methods::TOOLS_CALL));

        assert_eq!(
            handler_error_message(true, "boom", "Error: boom"),
            "boom"
        );
        assert_eq!(
            handler_error_message(false, "boom", "stringified"),
            "stringified"
        );

        let call = serde_json::json!({"_meta": {"progressToken": 7}});
        assert_eq!(
            tool_context_progress_token(&call),
            Some(serde_json::json!(7))
        );
        assert!(logging_set_level_accepts("info"));
        assert!(!logging_set_level_accepts("verbose"));
    }

    #[test]
    fn wave24_handler_prompt_args_dispatch_routing() {
        assert_eq!(
            unknown_prompt_error_message("greet"),
            "Unknown prompt: greet"
        );
        assert_eq!(
            unknown_resource_error_message("file:///a"),
            "Unknown resource: file:///a"
        );
        assert_eq!(
            prompt_arguments_from_get_params(&serde_json::json!({
                "name": "g",
                "arguments": {"x": 1}
            })),
            serde_json::json!({"x": 1})
        );
        assert_eq!(
            prompt_arguments_from_get_params(&serde_json::json!({"name": "g"})),
            serde_json::json!({})
        );
        let prompt_params = serde_json::json!({
            "ref": {"type": "ref/prompt", "name": "greet"},
            "argument": {"name": "who", "value": "Ad"}
        });
        let resource_params = serde_json::json!({
            "ref": {"type": "ref/resource", "uri": "file:///{id}"},
            "argument": {"name": "id", "value": "a"}
        });
        assert!(completion_ref_is_prompt(&prompt_params));
        assert!(!completion_ref_is_resource(&prompt_params));
        assert!(completion_ref_is_resource(&resource_params));
        assert!(!completion_ref_is_prompt(&resource_params));
        assert!(notification_should_mark_initialized(methods::INITIALIZED));
        assert!(!notification_should_mark_initialized(methods::TOOLS_CALL));
        assert!(dispatch_returns_none(true, false));
        assert!(dispatch_returns_none(false, false));
        assert!(!dispatch_returns_none(false, true));
        assert!(dispatch_success_result_present(true, true));
        assert!(!dispatch_success_result_present(true, false));
        assert!(!dispatch_success_result_present(false, true));
        assert_eq!(logging_set_level_result(), serde_json::json!({}));
        assert_eq!(subscription_mutation_result(), empty_object_result());
        assert_eq!(
            tool_arguments_or_empty(&serde_json::json!({
                "name": "t",
                "arguments": {"a": true}
            })),
            serde_json::json!({"a": true})
        );
        assert_eq!(
            tool_arguments_or_empty(&serde_json::json!({"name": "t"})),
            serde_json::json!({})
        );
        assert!(is_core_handler_method(methods::INITIALIZE));
        assert!(is_core_handler_method(methods::COMPLETION_COMPLETE));
        assert!(!is_core_handler_method("no/such"));
    }

    #[test]
    fn wave25_progress_roots_sampling_residual() {
        assert!(progress_params_valid(0.0, None));
        assert!(progress_params_valid(5.0, Some(10.0)));
        assert!(!progress_params_valid(-1.0, None));
        assert!(!progress_params_valid(11.0, Some(10.0)));
        assert_eq!(progress_fraction(5.0, Some(10.0)), Some(0.5));
        assert_eq!(progress_fraction(5.0, None), None);

        let empty = serde_json::json!({"roots": []});
        assert!(roots_list_is_empty(&empty));
        assert_eq!(roots_list_count(&empty), 0);
        let roots = serde_json::json!({"roots": [{"uri": "file:///a"}, {"uri": "file:///b"}]});
        assert!(!roots_list_is_empty(&roots));
        assert_eq!(roots_list_count(&roots), 2);
        assert_eq!(root_uri_at(&roots, 0), Some("file:///a"));

        assert!(sampling_max_tokens_valid(1));
        assert!(!sampling_max_tokens_valid(0));
        assert!(sampling_temperature_ok(None));
        assert!(sampling_temperature_ok(Some(0.7)));
        assert!(!sampling_temperature_ok(Some(3.0)));

        let params = progress_notification_params(
            serde_json::json!("tok"),
            3.0,
            Some(10.0),
            Some("half".into()),
        );
        assert_eq!(params["progressToken"], "tok");
        assert_eq!(params["progress"], 3.0);
        assert_eq!(params["total"], 10.0);
        assert_eq!(params["message"], "half");

        assert!(sampling_client_available(true, true));
        assert!(!sampling_client_available(true, false));
        assert!(elicitation_client_available(true, true));
        assert!(!elicitation_client_available(false, true));
    }

    #[test]
    fn wave26_cancellation_ping_logging_residual() {
        assert_eq!(NOTIFICATION_CANCELLED, "notifications/cancelled");
        assert_eq!(METHOD_PING, "ping");
        assert_eq!(LOGGING_LEVELS.len(), 8);
        assert!(logging_level_valid("warning"));
        assert!(!logging_level_valid("WARN"));
        let p = cancellation_params(serde_json::json!(1), Some("user"));
        assert_eq!(p["requestId"], 1);
        assert_eq!(p["reason"], "user");
        let p2 = cancellation_params(serde_json::json!("a"), None);
        assert!(p2.get("reason").is_none());
        assert_eq!(wave26_ping_result_empty(), serde_json::json!({}));
        assert!(logging_set_level_params_valid(&serde_json::json!({"level": "info"})));
        assert!(!logging_set_level_params_valid(&serde_json::json!({"level": "verbose"})));
        assert_eq!(empty_roots_list_result()["roots"], serde_json::json!([]));
        let with_tok = serde_json::json!({"name": "t", "_meta": {"progressToken": "p1"}});
        assert!(call_params_has_progress_token(&with_tok));
        assert!(!call_params_has_progress_token(&serde_json::json!({"name": "t"})));
    }

    #[test]
    fn wave27_resource_templates_elicitation_residual() {
        assert_eq!(METHOD_RESOURCES_TEMPLATES_LIST, "resources/templates/list");
        assert_eq!(METHOD_ELICITATION_CREATE, "elicitation/create");
        let empty = empty_resource_templates_list_result();
        assert_eq!(resource_templates_count(&empty), 0);
        assert_eq!(empty["resourceTemplates"], serde_json::json!([]));
        let tpl = serde_json::json!({"uriTemplate": "file:///{path}", "name": "f"});
        assert!(resource_template_uri_present(&tpl));
        assert!(!resource_template_uri_present(&serde_json::json!({"name": "x"})));
        let listed = serde_json::json!({"resourceTemplates": [tpl.clone(), tpl]});
        assert_eq!(resource_templates_count(&listed), 2);
        assert!(elicitation_params_valid(&serde_json::json!({"message": "ok"})));
        assert!(!elicitation_params_valid(&serde_json::json!({"message": ""})));
        assert!(!elicitation_params_valid(&serde_json::json!({})));
        assert_eq!(wave27_empty_object_result(), serde_json::json!({}));
        assert!(initialize_result_has_server_info(&serde_json::json!({
            "serverInfo": {"name": "demo"}
        })));
        assert!(!initialize_result_has_server_info(&serde_json::json!({})));
    }

    #[test]
    fn wave28_tools_prompts_resources_list_residual() {
        assert_eq!(METHOD_TOOLS_LIST, "tools/list");
        assert_eq!(METHOD_PROMPTS_LIST, "prompts/list");
        assert_eq!(METHOD_RESOURCES_LIST, "resources/list");
        let tools = empty_tools_list_result();
        assert_eq!(tools_count_from_list(&tools), 0);
        assert_eq!(tools["tools"], serde_json::json!([]));
        let prompts = empty_prompts_list_result();
        assert_eq!(prompts["prompts"], serde_json::json!([]));
        let resources = empty_resources_list_result();
        assert_eq!(resources["resources"], serde_json::json!([]));
        let tool = serde_json::json!({"name": "echo", "inputSchema": {"type": "object"}});
        assert!(tool_descriptor_name_present(&tool));
        assert!(tool_input_schema_is_object(&tool));
        assert!(!tool_descriptor_name_present(&serde_json::json!({"name": ""})));
        assert!(prompt_descriptor_name_present(&serde_json::json!({"name": "p"})));
        assert!(!prompt_descriptor_name_present(&serde_json::json!({})));
        assert_eq!(wave28_empty_array(), serde_json::json!([]));
        let listed = serde_json::json!({"tools": [tool.clone(), tool]});
        assert_eq!(tools_count_from_list(&listed), 2);
    }

    #[test]
    fn wave29_tools_call_prompts_get_resources_read_residual() {
        assert_eq!(METHOD_TOOLS_CALL, "tools/call");
        assert_eq!(METHOD_PROMPTS_GET, "prompts/get");
        assert_eq!(METHOD_RESOURCES_READ, "resources/read");
        assert!(tools_call_params_valid(&serde_json::json!({"name": "echo"})));
        assert!(!tools_call_params_valid(&serde_json::json!({"name": ""})));
        assert!(!tools_call_params_valid(&serde_json::json!({})));
        assert!(prompts_get_params_valid(&serde_json::json!({"name": "p"})));
        assert!(!prompts_get_params_valid(&serde_json::json!({"name": ""})));
        assert!(resources_read_params_valid(&serde_json::json!({"uri": "file:///a"})));
        assert!(!resources_read_params_valid(&serde_json::json!({"uri": ""})));
        assert!(tool_result_is_error(&serde_json::json!({"isError": true})));
        assert!(!tool_result_is_error(&serde_json::json!({"isError": false})));
        assert!(!tool_result_is_error(&serde_json::json!({})));
        assert_eq!(empty_tool_content(), serde_json::json!([]));
        assert_eq!(wave29_empty_object(), serde_json::json!({}));
    }

    #[test]
    fn wave30_notifications_roots_residual() {
        assert_eq!(METHOD_NOTIFICATIONS_INITIALIZED, "notifications/initialized");
        assert_eq!(METHOD_NOTIFICATIONS_CANCELLED, "notifications/cancelled");
        assert_eq!(METHOD_NOTIFICATIONS_PROGRESS, "notifications/progress");
        assert_eq!(METHOD_ROOTS_LIST, "roots/list");
        assert!(is_notification_method("notifications/initialized"));
        assert!(!is_notification_method("tools/list"));
        assert!(roots_list_params_valid(None));
        assert!(roots_list_params_valid(Some(&serde_json::json!({}))));
        assert!(!roots_list_params_valid(Some(&serde_json::json!([]))));
        assert!(notifications_progress_params_valid(&serde_json::json!({"progressToken": "t1"})));
        assert!(!notifications_progress_params_valid(&serde_json::json!({})));
        assert_eq!(empty_roots_list_result()["roots"], serde_json::json!([]));
        assert_eq!(wave30_notification_ack(), serde_json::json!({}));
    }

    #[test]
    fn wave31_subscribe_logging_completion_residual() {
        assert_eq!(METHOD_RESOURCES_SUBSCRIBE, "resources/subscribe");
        assert_eq!(METHOD_RESOURCES_UNSUBSCRIBE, "resources/unsubscribe");
        assert_eq!(METHOD_LOGGING_SET_LEVEL, "logging/setLevel");
        assert_eq!(METHOD_COMPLETION_COMPLETE, "completion/complete");
        assert!(resources_subscribe_params_valid(&serde_json::json!({"uri": "file:///a"})));
        assert!(!resources_subscribe_params_valid(&serde_json::json!({"uri": ""})));
        assert!(!resources_subscribe_params_valid(&serde_json::json!({})));
        assert!(resources_unsubscribe_params_valid(&serde_json::json!({"uri": "file:///b"})));
        assert!(logging_set_level_params_gate(&serde_json::json!({"level": "info"})));
        assert!(!logging_set_level_params_gate(&serde_json::json!({"level": "verbose"})));
        assert!(completion_complete_params_valid(&serde_json::json!({"ref": {"type": "ref/prompt", "name": "p"}})));
        assert!(!completion_complete_params_valid(&serde_json::json!({"ref": "x"})));
        assert!(!completion_complete_params_valid(&serde_json::json!({})));
        assert_eq!(empty_completion_values(), serde_json::json!([]));
        assert_eq!(wave31_empty_object(), serde_json::json!({}));
        assert!(is_resource_subscription_method("resources/subscribe"));
        assert!(is_resource_subscription_method("resources/unsubscribe"));
        assert!(!is_resource_subscription_method("resources/list"));
    }

    #[test]
    fn wave32_message_resources_list_changed_residual() {
        assert_eq!(WAVE32_METHOD_NOTIFICATIONS_MESSAGE, "notifications/message");
        assert_eq!(
            WAVE32_METHOD_NOTIFICATIONS_RESOURCES_UPDATED,
            "notifications/resources/updated"
        );
        assert_eq!(
            WAVE32_METHOD_NOTIFICATIONS_TOOLS_LIST_CHANGED,
            "notifications/tools/list_changed"
        );
        assert_eq!(
            WAVE32_METHOD_NOTIFICATIONS_PROMPTS_LIST_CHANGED,
            "notifications/prompts/list_changed"
        );
        assert!(wave32_notifications_message_params_valid(
            &serde_json::json!({"level":"info","data":{"m":"x"}})
        ));
        assert!(!wave32_notifications_message_params_valid(
            &serde_json::json!({"level":"info"})
        ));
        assert!(!wave32_notifications_message_params_valid(
            &serde_json::json!({"level":"","data":{}})
        ));
        assert!(wave32_resources_updated_params_valid(
            &serde_json::json!({"uri":"file:///a"})
        ));
        assert!(!wave32_resources_updated_params_valid(
            &serde_json::json!({"uri":""})
        ));
        assert!(wave32_list_changed_params_ok(&serde_json::json!({})));
        assert!(wave32_list_changed_params_ok(&serde_json::Value::Null));
        assert!(!wave32_list_changed_params_ok(&serde_json::json!([])));
        assert_eq!(wave32_empty_object(), serde_json::json!({}));
        assert!(is_wave32_list_changed_or_message_method("notifications/message"));
        assert!(is_wave32_list_changed_or_message_method(
            "notifications/tools/list_changed"
        ));
        assert!(!is_wave32_list_changed_or_message_method("ping"));
    }

    #[test]
    fn wave33_sampling_roots_list_changed_residual() {
        assert_eq!(
            WAVE33_METHOD_SAMPLING_CREATE_MESSAGE,
            "sampling/createMessage"
        );
        assert_eq!(
            WAVE33_METHOD_NOTIFICATIONS_ROOTS_LIST_CHANGED,
            "notifications/roots/list_changed"
        );
        assert_eq!(
            WAVE33_METHOD_NOTIFICATIONS_RESOURCES_LIST_CHANGED,
            "notifications/resources/list_changed"
        );
        assert!(wave33_sampling_create_message_params_valid(&serde_json::json!({
            "messages": [{"role": "user", "content": {"type": "text", "text": "hi"}}]
        })));
        assert!(!wave33_sampling_create_message_params_valid(&serde_json::json!({
            "messages": []
        })));
        assert!(!wave33_sampling_create_message_params_valid(&serde_json::json!({})));
        assert!(wave33_list_changed_params_ok(&serde_json::json!({})));
        assert!(wave33_list_changed_params_ok(&serde_json::Value::Null));
        assert!(!wave33_list_changed_params_ok(&serde_json::json!([])));
        assert_eq!(wave33_empty_object(), serde_json::json!({}));
        assert!(is_wave33_sampling_or_list_changed_method("sampling/createMessage"));
        assert!(is_wave33_sampling_or_list_changed_method(
            "notifications/roots/list_changed"
        ));
        assert!(!is_wave33_sampling_or_list_changed_method("ping"));
    }






    #[test]
    fn wave34_progress_tools_list_residual() {
        assert_eq!(
            WAVE34_METHOD_NOTIFICATIONS_PROGRESS,
            "notifications/progress"
        );
        assert_eq!(WAVE34_METHOD_TOOLS_LIST, "tools/list");
        assert_eq!(WAVE34_METHOD_TOOLS_CALL, "tools/call");
        assert!(wave34_progress_params_valid(&serde_json::json!({
            "progressToken": "t1",
            "progress": 50.0
        })));
        assert!(wave34_progress_params_valid(&serde_json::json!({
            "progressToken": 1,
            "progress": 0.0
        })));
        assert!(!wave34_progress_params_valid(&serde_json::json!({
            "progress": 50.0
        })));
        assert!(!wave34_progress_params_valid(&serde_json::json!({
            "progressToken": "t1",
            "progress": 101.0
        })));
        assert!(wave34_tools_list_params_ok(&serde_json::json!({})));
        assert!(wave34_tools_list_params_ok(&serde_json::Value::Null));
        assert!(!wave34_tools_list_params_ok(&serde_json::json!([])));
        assert_eq!(wave34_empty_object(), serde_json::json!({}));
        assert!(is_wave34_progress_or_tools_method("notifications/progress"));
        assert!(is_wave34_progress_or_tools_method("tools/list"));
        assert!(is_wave34_progress_or_tools_method("tools/call"));
        assert!(!is_wave34_progress_or_tools_method("ping"));
    }

    #[test]
    fn wave35_method_catalog_residual() {
        assert_eq!(wave35_method_catalog_count(), WAVE35_METHOD_COUNT);
        assert_eq!(WAVE35_METHOD_COUNT, 25);
        assert_eq!(WAVE35_METHOD_CATALOG[0], "initialize");
        assert_eq!(
            WAVE35_METHOD_CATALOG[WAVE35_METHOD_COUNT - 1],
            "notifications/roots/list_changed"
        );
        assert!(is_wave35_method_catalog_member("tools/call"));
        assert!(is_wave35_method_catalog_member("elicitation/create"));
        assert!(!is_wave35_method_catalog_member("tools/list_changed"));
        assert!(is_wave35_lifecycle_method("initialize"));
        assert!(is_wave35_lifecycle_method("ping"));
        assert!(!is_wave35_lifecycle_method("tools/list"));
        assert!(is_wave35_server_to_client_request("sampling/createMessage"));
        assert!(is_wave35_server_to_client_request("elicitation/create"));
        assert!(!is_wave35_server_to_client_request("tools/call"));
        assert!(is_wave35_list_changed_notification(
            "notifications/tools/list_changed"
        ));
        assert!(is_wave35_list_changed_notification(
            "notifications/roots/list_changed"
        ));
        assert!(!is_wave35_list_changed_notification(
            "notifications/resources/updated"
        ));
        assert_eq!(wave35_empty_object(), serde_json::json!({}));
        for m in WAVE35_METHOD_CATALOG {
            assert!(is_wave35_method_catalog_member(m));
        }
    }

    #[test]
    fn wave36_content_type_catalog_residual() {
        assert_eq!(WAVE36_CONTENT_TYPE_COUNT, 4);
        assert_eq!(WAVE36_CONTENT_TYPES.len(), WAVE36_CONTENT_TYPE_COUNT);
        assert_eq!(WAVE36_CONTENT_TYPES[0], "text");
        assert_eq!(WAVE36_CONTENT_TYPES[3], "resource");
        assert!(is_wave36_content_type("text"));
        assert!(is_wave36_content_type("image"));
        assert!(is_wave36_content_type("audio"));
        assert!(is_wave36_content_type("resource"));
        assert!(!is_wave36_content_type("video"));
        assert!(is_wave36_media_content_type("image"));
        assert!(is_wave36_media_content_type("audio"));
        assert!(!is_wave36_media_content_type("text"));
        assert!(is_wave36_resource_content_type("resource"));
        assert!(!is_wave36_resource_content_type("text"));
        assert!(is_wave36_content_audience("user"));
        assert!(is_wave36_content_audience("assistant"));
        assert!(!is_wave36_content_audience("system"));
        assert!(wave36_priority_valid(None));
        assert!(wave36_priority_valid(Some(0.0)));
        assert!(wave36_priority_valid(Some(1.0)));
        assert!(!wave36_priority_valid(Some(1.1)));
        assert!(!wave36_priority_valid(Some(f64::NAN)));
        assert_eq!(wave36_empty_object(), serde_json::json!({}));
        for t in WAVE36_CONTENT_TYPES {
            assert!(is_wave36_content_type(t));
        }
    }

    #[test]
    fn wave37_protocol_version_catalog_residual() {
        assert_eq!(WAVE37_LATEST_PROTOCOL_VERSION, "2025-03-26");
        assert_eq!(WAVE37_SUPPORTED_PROTOCOL_VERSION_COUNT, 2);
        assert_eq!(
            WAVE37_SUPPORTED_PROTOCOL_VERSIONS.len(),
            WAVE37_SUPPORTED_PROTOCOL_VERSION_COUNT
        );
        assert_eq!(WAVE37_SUPPORTED_PROTOCOL_VERSIONS[0], "2025-03-26");
        assert_eq!(WAVE37_SUPPORTED_PROTOCOL_VERSIONS[1], "2024-11-05");
        assert!(is_wave37_supported_protocol_version("2025-03-26"));
        assert!(is_wave37_supported_protocol_version("2024-11-05"));
        assert!(!is_wave37_supported_protocol_version("2024-01-01"));
        assert!(is_wave37_latest_protocol_version("2025-03-26"));
        assert!(!is_wave37_latest_protocol_version("2024-11-05"));
        assert_eq!(
            wave37_negotiate_protocol_version(Some("2024-11-05")),
            "2024-11-05"
        );
        assert_eq!(
            wave37_negotiate_protocol_version(Some("nope")),
            "2025-03-26"
        );
        assert_eq!(wave37_negotiate_protocol_version(None), "2025-03-26");
        assert_eq!(wave37_empty_object(), serde_json::json!({}));
        // dual-oracle with existing SSOT constants
        assert_eq!(WAVE37_LATEST_PROTOCOL_VERSION, LATEST_PROTOCOL_VERSION);
        assert_eq!(
            WAVE37_SUPPORTED_PROTOCOL_VERSIONS,
            SUPPORTED_PROTOCOL_VERSIONS
        );
    }

    #[test]
    fn wave38_method_group_membership_residual() {
        assert_eq!(WAVE38_METHOD_GROUPS.len(), 8);
        assert!(is_wave38_method_group("tools"));
        assert!(is_wave38_method_group("lifecycle"));
        assert!(!is_wave38_method_group("admin"));
        assert_eq!(wave38_method_group("initialize"), Some("lifecycle"));
        assert_eq!(wave38_method_group("ping"), Some("lifecycle"));
        assert_eq!(wave38_method_group("tools/list"), Some("tools"));
        assert_eq!(wave38_method_group("tools/call"), Some("tools"));
        assert_eq!(wave38_method_group("prompts/get"), Some("prompts"));
        assert_eq!(wave38_method_group("resources/read"), Some("resources"));
        assert_eq!(
            wave38_method_group("notifications/message"),
            Some("notifications")
        );
        assert_eq!(wave38_method_group("logging/setLevel"), Some("logging"));
        assert_eq!(
            wave38_method_group("completion/complete"),
            Some("completion")
        );
        assert_eq!(
            wave38_method_group("sampling/createMessage"),
            Some("sampling")
        );
        assert_eq!(wave38_method_group("unknown/method"), None);
        assert!(is_wave38_tools_method("tools/call"));
        assert!(!is_wave38_tools_method("prompts/list"));
        assert!(is_wave38_notification_method("notifications/initialized"));
        assert_eq!(wave38_empty_object(), serde_json::json!({}));
        // dual-oracle: existing method constants map into groups
        assert_eq!(wave38_method_group(METHOD_TOOLS_LIST), Some("tools"));
        assert_eq!(wave38_method_group(METHOD_PING), Some("lifecycle"));
    }

    #[test]
    fn wave39_elicitation_templates_method_residual() {
        assert_eq!(WAVE39_METHOD_ELICITATION_CREATE, "elicitation/create");
        assert_eq!(
            WAVE39_METHOD_RESOURCES_TEMPLATES_LIST,
            "resources/templates/list"
        );
        assert_eq!(WAVE39_METHODS.len(), 2);
        assert!(is_wave39_method("elicitation/create"));
        assert!(is_wave39_method("resources/templates/list"));
        assert!(!is_wave39_method("tools/list"));
        assert!(wave39_elicitation_create_params_valid(&serde_json::json!({
            "message": "Confirm?"
        })));
        assert!(!wave39_elicitation_create_params_valid(&serde_json::json!({
            "message": "  "
        })));
        assert!(!wave39_elicitation_create_params_valid(&serde_json::json!({})));
        assert!(wave39_resources_templates_list_params_ok(&serde_json::json!({})));
        assert!(wave39_resources_templates_list_params_ok(&serde_json::Value::Null));
        assert!(!wave39_resources_templates_list_params_ok(&serde_json::json!([])));
        assert_eq!(wave39_empty_object(), serde_json::json!({}));
        assert_eq!(
            wave39_method_group(WAVE39_METHOD_ELICITATION_CREATE),
            Some("elicitation")
        );
        assert_eq!(
            wave39_method_group(WAVE39_METHOD_RESOURCES_TEMPLATES_LIST),
            Some("resources")
        );
        // dual-oracle with existing method constants
        assert_eq!(WAVE39_METHOD_ELICITATION_CREATE, METHOD_ELICITATION_CREATE);
        assert_eq!(
            WAVE39_METHOD_RESOURCES_TEMPLATES_LIST,
            METHOD_RESOURCES_TEMPLATES_LIST
        );
    }

    #[test]
    fn wave40_logging_completion_method_residual() {
        assert_eq!(WAVE40_METHOD_LOGGING_SET_LEVEL, "logging/setLevel");
        assert_eq!(WAVE40_METHOD_COMPLETION_COMPLETE, "completion/complete");
        assert_eq!(WAVE40_METHODS.len(), 2);
        assert!(is_wave40_method("logging/setLevel"));
        assert!(is_wave40_method("completion/complete"));
        assert!(!is_wave40_method("tools/list"));
        assert!(wave40_methods_match_ssot());
        assert_eq!(wave40_method_group(WAVE40_METHOD_LOGGING_SET_LEVEL), Some("logging"));
        assert_eq!(wave40_method_group(WAVE40_METHOD_COMPLETION_COMPLETE), Some("completion"));
        assert_eq!(wave40_method_group("nope"), None);
        assert!(wave40_logging_set_level_params_valid(&serde_json::json!({"level": "info"})));
        assert!(!wave40_logging_set_level_params_valid(&serde_json::json!({"level": "  "})));
        assert!(!wave40_logging_set_level_params_valid(&serde_json::json!({})));
        assert!(wave40_completion_complete_params_valid(&serde_json::json!({"ref": {}})));
        assert!(!wave40_completion_complete_params_valid(&serde_json::json!({"ref": "x"})));
        assert!(!wave40_completion_complete_params_valid(&serde_json::json!({})));
        assert_eq!(wave40_empty_object(), serde_json::json!({}));
        assert_eq!(WAVE40_METHOD_LOGGING_SET_LEVEL, METHOD_LOGGING_SET_LEVEL);
        assert_eq!(WAVE40_METHOD_COMPLETION_COMPLETE, METHOD_COMPLETION_COMPLETE);
    }







}

// ── WAVE41 pure residual dens: resources subscribe/unsubscribe method dual-oracle residual ──
// Dual-oracle residual of METHOD_RESOURCES_SUBSCRIBE / METHOD_RESOURCES_UNSUBSCRIBE pure halves.
// Transport/handler I/O residual retained. dens ≠ flip. package_main_rust not claimed.

/// Dual-oracle residual: WAVE41 subscribe method.
pub const WAVE41_METHOD_RESOURCES_SUBSCRIBE: &str = "resources/subscribe";
/// Dual-oracle residual: WAVE41 unsubscribe method.
pub const WAVE41_METHOD_RESOURCES_UNSUBSCRIBE: &str = "resources/unsubscribe";

/// Dual-oracle residual: WAVE41 methods catalog.
pub const WAVE41_METHODS: &[&str] = &[
    WAVE41_METHOD_RESOURCES_SUBSCRIBE,
    WAVE41_METHOD_RESOURCES_UNSUBSCRIBE,
];

/// Dual-oracle residual: known WAVE41 method.
#[must_use]
pub fn is_wave41_method(method: &str) -> bool {
    WAVE41_METHODS.contains(&method)
}

/// Dual-oracle residual: method group for WAVE41 methods.
#[must_use]
pub fn wave41_method_group(method: &str) -> Option<&'static str> {
    match method {
        WAVE41_METHOD_RESOURCES_SUBSCRIBE | WAVE41_METHOD_RESOURCES_UNSUBSCRIBE => Some("resources"),
        _ => None,
    }
}

/// Dual-oracle residual: dual-oracle with existing METHOD_* constants.
#[must_use]
pub fn wave41_methods_match_ssot() -> bool {
    WAVE41_METHOD_RESOURCES_SUBSCRIBE == METHOD_RESOURCES_SUBSCRIBE
        && WAVE41_METHOD_RESOURCES_UNSUBSCRIBE == METHOD_RESOURCES_UNSUBSCRIBE
}

/// Dual-oracle residual: subscribe/unsubscribe params require uri string.
#[must_use]
pub fn wave41_resource_sub_params_valid(params: &serde_json::Value) -> bool {
    params
        .get("uri")
        .and_then(|v| v.as_str())
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false)
}

/// Dual-oracle residual: WAVE41 method count is 2.
#[must_use]
pub fn wave41_method_count() -> usize {
    WAVE41_METHODS.len()
}

#[cfg(test)]
mod wave41_tests {
    use super::*;

    #[test]
    fn wave41_resources_subscribe_unsubscribe_dual_oracle() {
        assert_eq!(WAVE41_METHOD_RESOURCES_SUBSCRIBE, "resources/subscribe");
        assert_eq!(WAVE41_METHOD_RESOURCES_UNSUBSCRIBE, "resources/unsubscribe");
        assert_eq!(wave41_method_count(), 2);
        assert!(is_wave41_method("resources/subscribe"));
        assert!(is_wave41_method("resources/unsubscribe"));
        assert!(!is_wave41_method("resources/list"));
        assert_eq!(
            wave41_method_group(WAVE41_METHOD_RESOURCES_SUBSCRIBE),
            Some("resources")
        );
        assert_eq!(
            wave41_method_group(WAVE41_METHOD_RESOURCES_UNSUBSCRIBE),
            Some("resources")
        );
        assert!(wave41_methods_match_ssot());
        assert!(wave41_resource_sub_params_valid(&serde_json::json!({"uri": "file:///a"})));
        assert!(!wave41_resource_sub_params_valid(&serde_json::json!({"uri": ""})));
        assert!(!wave41_resource_sub_params_valid(&serde_json::json!({})));
        assert_eq!(WAVE41_METHOD_RESOURCES_SUBSCRIBE, METHOD_RESOURCES_SUBSCRIBE);
        assert_eq!(WAVE41_METHOD_RESOURCES_UNSUBSCRIBE, METHOD_RESOURCES_UNSUBSCRIBE);
    }
}
