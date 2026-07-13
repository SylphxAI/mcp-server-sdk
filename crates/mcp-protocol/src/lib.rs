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
            | Self::Audio { annotations: a, .. } => *a = Some(annotations),
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

}
