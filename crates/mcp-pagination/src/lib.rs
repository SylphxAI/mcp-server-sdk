//! Pure cursor-based pagination (parity with `src/pagination/index.ts`).
//!
//! BW1 pure residual — no transport/authority claims.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde::{Deserialize, Serialize};

/// Pagination options.
#[derive(Debug, Clone, Copy)]
pub struct PaginationOptions {
    pub default_page_size: usize,
    pub max_page_size: usize,
}

impl Default for PaginationOptions {
    fn default() -> Self {
        Self {
            default_page_size: 50,
            max_page_size: 100,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CursorData {
    offset: usize,
    page_size: usize,
}

/// Page of items with optional next cursor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageResult<T> {
    pub items: Vec<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

fn encode_cursor(data: &CursorData) -> String {
    let json = match serde_json::to_vec(data) {
        Ok(v) => v,
        Err(_) => return String::new(),
    };
    URL_SAFE_NO_PAD.encode(json)
}

fn decode_cursor(cursor: &str) -> Option<CursorData> {
    let bytes = URL_SAFE_NO_PAD.decode(cursor.as_bytes()).ok()?;
    let data: CursorData = serde_json::from_slice(&bytes).ok()?;
    Some(data)
}

/// Paginate a slice of items (parity with TS `paginate`).
#[must_use]
pub fn paginate<T: Clone>(
    items: &[T],
    cursor: Option<&str>,
    options: PaginationOptions,
) -> PageResult<T> {
    let mut offset = 0usize;
    let mut page_size = options.default_page_size;

    if let Some(c) = cursor {
        if let Some(data) = decode_cursor(c) {
            offset = data.offset;
            page_size = data.page_size.min(options.max_page_size);
        }
    }

    let end = offset.saturating_add(page_size).min(items.len());
    let page = if offset >= items.len() {
        Vec::new()
    } else {
        items[offset..end].to_vec()
    };
    let has_more = offset.saturating_add(page_size) < items.len();

    PageResult {
        items: page,
        next_cursor: if has_more {
            Some(encode_cursor(&CursorData {
                offset: offset.saturating_add(page_size),
                page_size,
            }))
        } else {
            None
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_page_default() {
        let items: Vec<i32> = (0..120).collect();
        let page = paginate(&items, None, PaginationOptions::default());
        assert_eq!(page.items.len(), 50);
        assert!(page.next_cursor.is_some());
        assert_eq!(page.items[0], 0);
        assert_eq!(page.items[49], 49);
    }

    #[test]
    fn second_page_via_cursor() {
        let items: Vec<i32> = (0..120).collect();
        let first = paginate(&items, None, PaginationOptions::default());
        let second = paginate(
            &items,
            first.next_cursor.as_deref(),
            PaginationOptions::default(),
        );
        assert_eq!(second.items[0], 50);
        assert_eq!(second.items.len(), 50);
    }

    #[test]
    fn last_page_no_cursor() {
        let items: Vec<i32> = (0..10).collect();
        let page = paginate(
            &items,
            None,
            PaginationOptions {
                default_page_size: 50,
                max_page_size: 100,
            },
        );
        assert_eq!(page.items.len(), 10);
        assert!(page.next_cursor.is_none());
    }

    #[test]
    fn bad_cursor_falls_back() {
        let items = vec![1, 2, 3];
        let page = paginate(&items, Some("!!!"), PaginationOptions::default());
        assert_eq!(page.items, vec![1, 2, 3]);
    }

    #[test]
    fn max_page_size_caps() {
        let items: Vec<i32> = (0..200).collect();
        // Craft a cursor with huge page size
        let data = CursorData {
            offset: 0,
            page_size: 10_000,
        };
        let cursor = encode_cursor(&data);
        let page = paginate(
            &items,
            Some(&cursor),
            PaginationOptions {
                default_page_size: 50,
                max_page_size: 100,
            },
        );
        assert_eq!(page.items.len(), 100);
    }
}
