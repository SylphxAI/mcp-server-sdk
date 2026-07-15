//! Session store and guard helpers for MCP Streamable HTTP.
//!
//! Parity with the `Mcp-Session-Id` lifecycle embedded in:
//! - `src/transports/http.ts`
//! - `src/app/serve.ts`
//! - `src/app/http.ts`
//! - `fixtures/streamable-http/session-not-found.json`
//!
//! Pure bookkeeping + decision kernels. Async timers and HTTP binding stay
//! application-layer. WAVE5 product surface: `transport/session-guard-helpers`.
//! TS remains a read-only oracle. No authority_rust / ts_deleted claim.

use std::collections::BTreeMap;

use mcp_stdio::PendingRequests;
use serde_json::{json, Value};
use uuid::Uuid;

/// Response header / request header name for MCP sessions.
pub const MCP_SESSION_ID_HEADER: &str = "Mcp-Session-Id";

/// Lower-case form (HTTP headers are case-insensitive).
pub const MCP_SESSION_ID_HEADER_LOWER: &str = "mcp-session-id";

/// HTTP status when a provided session id is unknown.
pub const SESSION_NOT_FOUND_STATUS: u16 = 404;

/// Canonical error body for unknown session.
#[must_use]
pub fn session_not_found_body() -> Value {
    json!({ "error": "Session not found" })
}

/// Canonical `Content-Type` for session-not-found JSON body.
pub const SESSION_NOT_FOUND_CONTENT_TYPE: &str = "application/json";

/// Default timeout for server→client pending requests (ms), parity with TS.
pub const DEFAULT_PENDING_TIMEOUT_MS: u64 = mcp_stdio::DEFAULT_REQUEST_TIMEOUT_MS;

/// UUID v4 pattern used by golden fixtures for `Mcp-Session-Id`.
pub const UUID_V4_PATTERN: &str =
    r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$";

/// Mint a new session id (UUID v4, lowercase hex — parity with `crypto.randomUUID()`).
#[must_use]
pub fn new_session_id() -> String {
    Uuid::new_v4().to_string()
}

/// True when `id` matches the loose UUID shape used by the transport contract.
#[must_use]
pub fn looks_like_session_id(id: &str) -> bool {
    let b = id.as_bytes();
    if b.len() != 36 {
        return false;
    }
    // 8-4-4-4-12 hex with dashes
    let dash = |i: usize| b[i] == b'-';
    let hex = |i: usize| b[i].is_ascii_hexdigit();
    for (i, ch) in b.iter().enumerate() {
        match i {
            8 | 13 | 18 | 23 => {
                if !dash(i) {
                    return false;
                }
            }
            _ => {
                if !hex(i) || !ch.is_ascii_hexdigit() {
                    return false;
                }
            }
        }
    }
    true
}

/// Read `Mcp-Session-Id` from a flat header map (case-insensitive).
#[must_use]
pub fn read_session_id_header(headers: &BTreeMap<String, String>) -> Option<String> {
    for (k, v) in headers {
        if k.eq_ignore_ascii_case(MCP_SESSION_ID_HEADER) {
            let t = v.trim();
            if t.is_empty() {
                return None;
            }
            return Some(t.to_string());
        }
    }
    None
}

/// One MCP session (created on `initialize`).
#[derive(Debug, Clone, Default)]
pub struct Session {
    pub created_at_ms: u64,
    /// Pending server→client request ids (sampling/elicitation).
    pub pending: PendingRequests,
}

impl Session {
    #[must_use]
    pub fn new(created_at_ms: u64) -> Self {
        Self {
            created_at_ms,
            pending: PendingRequests::new(),
        }
    }
}

/// In-memory session map (transport-scoped).
#[derive(Debug, Clone, Default)]
pub struct SessionStore {
    sessions: BTreeMap<String, Session>,
}

impl SessionStore {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.sessions.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.sessions.is_empty()
    }

    /// Insert a session under `id`.
    pub fn insert(&mut self, id: impl Into<String>, session: Session) {
        self.sessions.insert(id.into(), session);
    }

    /// Create and store a fresh session; returns the new id.
    pub fn create(&mut self, created_at_ms: u64) -> String {
        let id = new_session_id();
        self.sessions.insert(id.clone(), Session::new(created_at_ms));
        id
    }

    /// Create with a caller-provided id (tests / deterministic fixtures).
    pub fn create_with_id(&mut self, id: impl Into<String>, created_at_ms: u64) -> String {
        let id = id.into();
        self.sessions.insert(id.clone(), Session::new(created_at_ms));
        id
    }

    #[must_use]
    pub fn get(&self, id: &str) -> Option<&Session> {
        self.sessions.get(id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut Session> {
        self.sessions.get_mut(id)
    }

    #[must_use]
    pub fn contains(&self, id: &str) -> bool {
        self.sessions.contains_key(id)
    }

    /// Remove a session; returns true if it existed.
    pub fn remove(&mut self, id: &str) -> bool {
        self.sessions.remove(id).is_some()
    }

    /// Clear all sessions (parity: `stop()` / server shutdown).
    pub fn clear(&mut self) {
        self.sessions.clear();
    }
}

/// Outcome of guarding an inbound request that may carry a session id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionGuardResult {
    /// No session header provided — anonymous request allowed.
    Anonymous,
    /// Known session.
    Known { session_id: String },
    /// Header present but not in the store → 404.
    NotFound { session_id: String },
}

/// Guard an optional session id against the store.
///
/// Parity with:
/// ```ts
/// if (sessionId && !session) return 404 Session not found
/// ```
#[must_use]
pub fn guard_session_id(store: &SessionStore, session_id: Option<&str>) -> SessionGuardResult {
    match session_id {
        None | Some("") => SessionGuardResult::Anonymous,
        Some(id) if store.contains(id) => SessionGuardResult::Known {
            session_id: id.to_string(),
        },
        Some(id) => SessionGuardResult::NotFound {
            session_id: id.to_string(),
        },
    }
}

/// Convenience: should the transport return 404 Session not found?
#[must_use]
pub fn is_unknown_session(store: &SessionStore, session_id: Option<&str>) -> bool {
    matches!(
        guard_session_id(store, session_id),
        SessionGuardResult::NotFound { .. }
    )
}

/// Whether this JSON-RPC method allocates a new session (`initialize`).
#[must_use]
pub fn method_allocates_session(method: &str) -> bool {
    method == "initialize"
}

/// Decide whether to emit a new `Mcp-Session-Id` response header.
///
/// New session header is set when:
/// - request is `initialize`, AND
/// - client did not already send a (known) session id for that allocation path
///
/// TS sets the header when allocating (both JSON and SSE paths).
#[must_use]
pub fn should_emit_new_session_header(
    is_initialize: bool,
    client_sent_session_id: bool,
    allocated_new: bool,
) -> bool {
    is_initialize && allocated_new && !client_sent_session_id
}

/// Response headers for a newly allocated session.
#[must_use]
pub fn new_session_response_headers(session_id: &str) -> Vec<(String, String)> {
    vec![
        (MCP_SESSION_ID_HEADER.to_string(), session_id.to_string()),
        (
            "Content-Type".into(),
            SESSION_NOT_FOUND_CONTENT_TYPE.into(),
        ),
    ]
}

/// Response headers for session-not-found.
#[must_use]
pub fn session_not_found_headers() -> Vec<(String, String)> {
    vec![(
        "Content-Type".into(),
        SESSION_NOT_FOUND_CONTENT_TYPE.into(),
    )]
}

/// Allocate a pending server→client request id on a known session.
///
/// Returns `None` if the session does not exist.
pub fn allocate_pending_on_session(
    store: &mut SessionStore,
    session_id: &str,
) -> Option<String> {
    let session = store.get_mut(session_id)?;
    Some(session.pending.allocate())
}

/// Resolve a pending server→client request (client POSTed a JSON-RPC response).
///
/// Returns true if the id was pending and is now taken.
pub fn take_pending_on_session(
    store: &mut SessionStore,
    session_id: &str,
    request_id: &str,
) -> bool {
    match store.get_mut(session_id) {
        Some(session) => session.pending.take(request_id),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_session_id_looks_like_uuid() {
        let id = new_session_id();
        assert!(looks_like_session_id(&id), "id={id}");
        assert_eq!(id.len(), 36);
    }

    #[test]
    fn looks_like_session_id_rejects_junk() {
        assert!(!looks_like_session_id(""));
        assert!(!looks_like_session_id("not-a-uuid"));
        assert!(!looks_like_session_id("00000000-0000-4000-8000-00000000000")); // short
        assert!(looks_like_session_id("00000000-0000-4000-8000-000000000000"));
    }

    #[test]
    fn store_create_get_clear() {
        let mut store = SessionStore::new();
        assert!(store.is_empty());
        let id = store.create(1_000);
        assert_eq!(store.len(), 1);
        assert!(store.contains(&id));
        assert_eq!(store.get(&id).map(|s| s.created_at_ms), Some(1_000));
        store.clear();
        assert!(store.is_empty());
    }

    #[test]
    fn guard_anonymous_known_not_found() {
        let mut store = SessionStore::new();
        let id = store.create_with_id("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee", 0);
        assert_eq!(
            guard_session_id(&store, None),
            SessionGuardResult::Anonymous
        );
        assert_eq!(
            guard_session_id(&store, Some("")),
            SessionGuardResult::Anonymous
        );
        assert_eq!(
            guard_session_id(&store, Some(&id)),
            SessionGuardResult::Known {
                session_id: id.clone()
            }
        );
        assert_eq!(
            guard_session_id(&store, Some("00000000-0000-4000-8000-000000000000")),
            SessionGuardResult::NotFound {
                session_id: "00000000-0000-4000-8000-000000000000".into()
            }
        );
        assert!(is_unknown_session(
            &store,
            Some("00000000-0000-4000-8000-000000000000")
        ));
        assert!(!is_unknown_session(&store, Some(&id)));
        assert!(!is_unknown_session(&store, None));
    }

    #[test]
    fn session_not_found_body_shape() {
        let body = session_not_found_body();
        assert_eq!(body["error"], "Session not found");
        assert_eq!(SESSION_NOT_FOUND_STATUS, 404);
    }

    #[test]
    fn read_session_id_header_case_insensitive() {
        let mut h = BTreeMap::new();
        h.insert("MCP-SESSION-ID".into(), "  abc  ".into());
        assert_eq!(read_session_id_header(&h).as_deref(), Some("abc"));
        let empty = BTreeMap::new();
        assert!(read_session_id_header(&empty).is_none());
    }

    #[test]
    fn method_allocates_and_emit_header() {
        assert!(method_allocates_session("initialize"));
        assert!(!method_allocates_session("tools/list"));
        assert!(should_emit_new_session_header(true, false, true));
        assert!(!should_emit_new_session_header(true, true, true));
        assert!(!should_emit_new_session_header(true, false, false));
        assert!(!should_emit_new_session_header(false, false, true));
    }

    #[test]
    fn pending_allocate_and_take() {
        let mut store = SessionStore::new();
        let id = store.create(0);
        let rid = allocate_pending_on_session(&mut store, &id).expect("session");
        assert!(rid.starts_with("server-"));
        assert!(take_pending_on_session(&mut store, &id, &rid));
        assert!(!take_pending_on_session(&mut store, &id, &rid));
        assert!(allocate_pending_on_session(&mut store, "missing").is_none());
    }

    #[test]
    fn new_session_response_headers_include_id() {
        let h = new_session_response_headers("sid-1");
        assert!(h.iter().any(|(k, v)| k == MCP_SESSION_ID_HEADER && v == "sid-1"));
    }

    #[test]
    fn wave5_product_session_guard_surface() {
        // Dense smoke: fixture-shaped unknown session path.
        let store = SessionStore::new();
        let unknown = "00000000-0000-4000-8000-000000000000";
        assert!(is_unknown_session(&store, Some(unknown)));
        assert_eq!(
            session_not_found_body(),
            json!({ "error": "Session not found" })
        );
        assert!(UUID_V4_PATTERN.contains("0-9a-f"));
        assert_eq!(DEFAULT_PENDING_TIMEOUT_MS, 30_000);
    }
}
