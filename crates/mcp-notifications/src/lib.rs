//! Notification product kernels (helpers + emitter wire conversion).
//!
//! Parity with `src/notifications/{helpers,emitter,types}.ts`.
//! Pure notification construction and toJsonRpc conversion. Transport send is
//! injected. WAVE4: rust_impl product surface.

use mcp_protocol::{
    cancelled, log_notification, notification_to_jsonrpc, progress_notification,
    prompts_list_changed, resource_updated, resources_list_changed, tools_list_changed,
    JsonRpcNotificationWire, Notification,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Create a progress notification (parity: helpers.progress).
#[must_use]
pub fn progress(
    progress_token: Value,
    current: f64,
    total: Option<f64>,
    message: Option<String>,
) -> Notification {
    progress_notification(progress_token, current, total, message)
}

/// Create a log notification (parity: helpers.log).
#[must_use]
pub fn log(level: impl Into<String>, data: Value, logger: Option<String>) -> Notification {
    log_notification(level, data, logger)
}

/// resources/list_changed.
#[must_use]
pub fn resources_list_changed_n() -> Notification {
    resources_list_changed()
}

/// tools/list_changed.
#[must_use]
pub fn tools_list_changed_n() -> Notification {
    tools_list_changed()
}

/// prompts/list_changed.
#[must_use]
pub fn prompts_list_changed_n() -> Notification {
    prompts_list_changed()
}

/// resource/updated.
#[must_use]
pub fn resource_updated_n(uri: impl Into<String>) -> Notification {
    resource_updated(uri)
}

/// cancelled.
#[must_use]
pub fn cancelled_n(request_id: Value, reason: Option<String>) -> Notification {
    cancelled(request_id, reason)
}

/// Convert notification to JSON-RPC method + params (parity: emitter toJsonRpc).
#[must_use]
pub fn to_jsonrpc(n: &Notification) -> JsonRpcNotificationWire {
    notification_to_jsonrpc(n)
}

/// Emitted wire form for a notification (method + optional params).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmittedNotification {
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl From<JsonRpcNotificationWire> for EmittedNotification {
    fn from(w: JsonRpcNotificationWire) -> Self {
        Self {
            method: w.method,
            params: w.params,
        }
    }
}

/// Pure emitter: records emissions instead of sending on a transport.
#[derive(Debug, Clone, Default)]
pub struct RecordingEmitter {
    pub emissions: Vec<EmittedNotification>,
}

impl RecordingEmitter {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn emit(&mut self, notification: &Notification) {
        let wire = to_jsonrpc(notification);
        self.emissions.push(wire.into());
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.emissions.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.emissions.is_empty()
    }

    pub fn clear(&mut self) {
        self.emissions.clear();
    }
}

/// No-op emit (parity: noopEmitter) — pure function that discards.
pub fn noop_emit(_notification: &Notification) {}

/// Apply emit using a callback (parity: createEmitter(send)).
pub fn emit_with<F>(notification: &Notification, mut send: F)
where
    F: FnMut(&str, Option<&Value>),
{
    let wire = to_jsonrpc(notification);
    send(&wire.method, wire.params.as_ref());
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn helpers_and_to_jsonrpc() {
        let p = progress(json!("tok"), 50.0, Some(100.0), Some("half".into()));
        let w = to_jsonrpc(&p);
        assert!(w.method.contains("progress"));
        assert_eq!(
            w.params
                .as_ref()
                .and_then(|p| p.get("progress"))
                .and_then(|v| v.as_f64()),
            Some(50.0)
        );

        let l = log("info", json!({"ok":true}), Some("test".into()));
        let w = to_jsonrpc(&l);
        assert!(w.method.contains("message") || w.method.contains("log"));

        for n in [
            resources_list_changed_n(),
            tools_list_changed_n(),
            prompts_list_changed_n(),
            resource_updated_n("u://x"),
            cancelled_n(json!(1), Some("bye".into())),
        ] {
            let w = to_jsonrpc(&n);
            assert!(!w.method.is_empty());
        }
    }

    #[test]
    fn recording_emitter() {
        let mut e = RecordingEmitter::new();
        e.emit(&tools_list_changed_n());
        e.emit(&progress(json!(1), 1.0, None, None));
        assert_eq!(e.len(), 2);
        assert!(!e.is_empty());
        e.clear();
        assert!(e.is_empty());
    }

    #[test]
    fn emit_with_callback() {
        let mut got: Vec<String> = Vec::new();
        emit_with(&resources_list_changed_n(), |method, _| {
            got.push(method.to_string());
        });
        assert_eq!(got.len(), 1);
        noop_emit(&tools_list_changed_n());
    }

    #[test]
    fn wave4_product_notifications_surface() {
        let n = progress(json!(null), 0.0, None, None);
        let e: EmittedNotification = to_jsonrpc(&n).into();
        assert!(!e.method.is_empty());
    }
}
