//! Sampling and elicitation client product kernels.
//!
//! Parity with:
//! - `src/sampling/client.ts` + types
//! - `src/elicitation/client.ts` + types
//!
//! Pure param/result envelopes + method constants. Transport `send` is injected
//! at the application layer (async). WAVE4: rust_impl product surface.

use mcp_protocol::{
    elicitation_create_params, elicitation_create_result, is_valid_elicitation_action, methods,
    sampling_create_params, sampling_create_params_ext, sampling_create_result, sampling_message,
    Content,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

// ─── Method constants ───────────────────────────────────────────────────────

/// Sampling createMessage method (parity: Method.SamplingCreateMessage).
#[must_use]
pub fn sampling_method() -> &'static str {
    methods::SAMPLING_CREATE_MESSAGE
}

/// Elicitation create method.
#[must_use]
pub fn elicitation_method() -> &'static str {
    methods::ELICITATION_CREATE
}

// ─── Sampling ───────────────────────────────────────────────────────────────

/// Sampling message role constraint.
pub const SAMPLING_ROLES: &[&str] = &["user", "assistant"];

/// Build sampling createMessage params (core fields).
#[must_use]
pub fn build_sampling_params(
    messages: &[Value],
    max_tokens: u64,
    system_prompt: Option<String>,
    temperature: Option<f64>,
) -> Value {
    sampling_create_params(messages, max_tokens, system_prompt, temperature)
}

/// Build sampling createMessage params with extended options.
#[must_use]
pub fn build_sampling_params_ext(
    messages: &[Value],
    max_tokens: u64,
    system_prompt: Option<String>,
    temperature: Option<f64>,
    include_context: Option<String>,
    model_preferences: Option<Value>,
) -> Value {
    sampling_create_params_ext(
        messages,
        max_tokens,
        system_prompt,
        temperature,
        include_context,
        model_preferences,
    )
}

/// Text sampling user message envelope.
#[must_use]
pub fn sampling_user_text(text: impl Into<String>) -> Value {
    sampling_message("user", Content::text(text))
}

/// Text sampling assistant message envelope.
#[must_use]
pub fn sampling_assistant_text(text: impl Into<String>) -> Value {
    sampling_message("assistant", Content::text(text))
}

/// Build a successful sampling result body.
#[must_use]
pub fn build_sampling_result(
    role: impl Into<String>,
    content: Content,
    model: impl Into<String>,
    stop_reason: Option<String>,
) -> Value {
    sampling_create_result(role, content, model, stop_reason)
}

/// Extract text from a sampling result content field when type=text.
#[must_use]
pub fn sampling_result_text(result: &Value) -> Option<&str> {
    result.get("content").and_then(|c| {
        if c.get("type").and_then(|t| t.as_str()) == Some("text") {
            c.get("text").and_then(|t| t.as_str())
        } else {
            None
        }
    })
}

/// Extract model from sampling result.
#[must_use]
pub fn sampling_result_model(result: &Value) -> Option<&str> {
    result.get("model").and_then(|v| v.as_str())
}

/// Pure sampling "client" that only records the method + params it would send.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SamplingCall {
    pub method: String,
    pub params: Value,
}

/// Create a sampling call envelope (parity: createSamplingClient.createMessage body).
#[must_use]
pub fn sampling_create_message_call(params: Value) -> SamplingCall {
    SamplingCall {
        method: sampling_method().into(),
        params,
    }
}

// ─── Elicitation ────────────────────────────────────────────────────────────

/// Allowed elicitation property types.
pub const ELICITATION_PROPERTY_TYPES: &[&str] = &["string", "number", "integer", "boolean"];

/// Elicitation schema (flat object of primitives).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ElicitationSchemaMeta {
    #[serde(rename = "type")]
    pub schema_type: String,
    pub properties: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
}

impl ElicitationSchemaMeta {
    #[must_use]
    pub fn object(properties: Value, required: Option<Vec<String>>) -> Self {
        Self {
            schema_type: "object".into(),
            properties,
            required,
        }
    }

    #[must_use]
    pub fn to_value(&self) -> Value {
        serde_json::to_value(self).unwrap_or_else(|_| json!({}))
    }
}

/// Build elicitation/create params (parity: createElicitationClient.elicit).
#[must_use]
pub fn build_elicitation_params(
    message: impl Into<String>,
    schema: &ElicitationSchemaMeta,
) -> Value {
    elicitation_create_params(message, schema.to_value())
}

/// Build elicitation result.
#[must_use]
pub fn build_elicitation_result(action: impl Into<String>, content: Option<Value>) -> Value {
    elicitation_create_result(action, content)
}

/// Pure elicitation call envelope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ElicitationCall {
    pub method: String,
    pub params: Value,
}

/// Create an elicitation call (parity: elicit(message, schema)).
#[must_use]
pub fn elicitation_create_call(
    message: impl Into<String>,
    schema: &ElicitationSchemaMeta,
) -> ElicitationCall {
    ElicitationCall {
        method: elicitation_method().into(),
        params: build_elicitation_params(message, schema),
    }
}

/// True when action is accept/decline/cancel.
#[must_use]
pub fn is_elicitation_action(action: &str) -> bool {
    is_valid_elicitation_action(action)
}

/// Extract action from elicitation result.
#[must_use]
pub fn elicitation_result_action(result: &Value) -> Option<&str> {
    result.get("action").and_then(|v| v.as_str())
}

/// Extract content object from accept result.
#[must_use]
pub fn elicitation_result_content(result: &Value) -> Option<&Value> {
    if elicitation_result_action(result) == Some("accept") {
        result.get("content")
    } else {
        None
    }
}

/// Validate a property type string against allowed elicitation types.
#[must_use]
pub fn is_valid_property_type(t: &str) -> bool {
    ELICITATION_PROPERTY_TYPES.contains(&t)
}

/// Build a simple string property schema entry.
#[must_use]
pub fn string_property(description: Option<&str>) -> Value {
    let mut map = serde_json::Map::new();
    map.insert("type".into(), Value::String("string".into()));
    if let Some(d) = description {
        map.insert("description".into(), Value::String(d.into()));
    }
    Value::Object(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sampling_method_and_call() {
        assert_eq!(sampling_method(), "sampling/createMessage");
        let msgs = [sampling_user_text("Hello")];
        let params = build_sampling_params(&msgs, 100, None, None);
        assert_eq!(params["maxTokens"], 100);
        assert_eq!(params["messages"][0]["role"], "user");
        let call = sampling_create_message_call(params);
        assert_eq!(call.method, "sampling/createMessage");
        let result = build_sampling_result(
            "assistant",
            Content::text("Hi"),
            "test-model",
            Some("endTurn".into()),
        );
        assert_eq!(sampling_result_text(&result), Some("Hi"));
        assert_eq!(sampling_result_model(&result), Some("test-model"));
        assert_eq!(result["stopReason"], "endTurn");
    }

    #[test]
    fn sampling_params_ext() {
        let msgs = [sampling_assistant_text("prev")];
        let params = build_sampling_params_ext(
            &msgs,
            50,
            Some("sys".into()),
            Some(0.2),
            Some("none".into()),
            Some(json!({"costPriority": 0.5})),
        );
        assert_eq!(params["systemPrompt"], "sys");
        assert_eq!(params["includeContext"], "none");
        assert_eq!(params["modelPreferences"]["costPriority"], 0.5);
    }

    #[test]
    fn elicitation_call_and_result() {
        assert_eq!(elicitation_method(), "elicitation/create");
        let schema = ElicitationSchemaMeta::object(
            json!({
                "apiKey": string_property(Some("Your API key")),
            }),
            Some(vec!["apiKey".into()]),
        );
        let call = elicitation_create_call("Please provide key", &schema);
        assert_eq!(call.method, "elicitation/create");
        assert_eq!(call.params["message"], "Please provide key");
        assert_eq!(call.params["requestedSchema"]["type"], "object");

        let accepted = build_elicitation_result("accept", Some(json!({"apiKey":"k"})));
        assert_eq!(elicitation_result_action(&accepted), Some("accept"));
        assert_eq!(
            elicitation_result_content(&accepted)
                .and_then(|c| c.get("apiKey"))
                .and_then(|v| v.as_str()),
            Some("k")
        );
        let declined = build_elicitation_result("decline", None);
        assert!(elicitation_result_content(&declined).is_none());
        assert!(is_elicitation_action("cancel"));
        assert!(!is_elicitation_action("maybe"));
        assert!(is_valid_property_type("integer"));
        assert!(!is_valid_property_type("object"));
    }

    #[test]
    fn wave4_product_clients_roles() {
        assert!(SAMPLING_ROLES.contains(&"user"));
        assert!(SAMPLING_ROLES.contains(&"assistant"));
        assert_eq!(ELICITATION_PROPERTY_TYPES.len(), 4);
    }
}
