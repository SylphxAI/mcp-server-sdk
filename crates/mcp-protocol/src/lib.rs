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

/// Embedded resource payload (parity with MCP ResourceContents text form).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmbeddedResource {
    pub uri: String,
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
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
    pub annotations: Option<ToolAnnotations>,
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

/// Embedded resource content (parity with builders `embedded`).
#[must_use]
pub fn embedded_resource_content(
    uri: impl Into<String>,
    mime_type: Option<String>,
    text: Option<String>,
) -> Content {
    Content::Resource {
        resource: EmbeddedResource {
            uri: uri.into(),
            mime_type,
            text,
        },
        annotations: None,
    }
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


#[cfg(test)]
mod tests {
    use super::*;

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
        let v = serde_json::to_value(&c).unwrap();
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
        let p = extract_params("file:///{bucket}/{key}", "file:///b1/obj").unwrap();
        assert_eq!(p.get("bucket").map(String::as_str), Some("b1"));
        assert_eq!(p.get("key").map(String::as_str), Some("obj"));
        assert!(extract_params("file:///{bucket}/{key}", "file:///b1/obj/extra").is_none());
        assert!(matches_template("static", "static"));
        assert!(!matches_template("static", "static2"));
        let p2 = extract_params("/users/{id}/files/{name}", "/users/42/files/a.txt").unwrap();
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
        assert_eq!(body["messages"].as_array().unwrap().len(), 2);
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
}
