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

/// Clamp a requested page size into `[1, max_page_size]` (pure residual helper).
#[must_use]
pub fn clamp_page_size(requested: usize, max_page_size: usize) -> usize {
    if max_page_size == 0 {
        return 0;
    }
    requested.clamp(1, max_page_size)
}

/// Encode a pagination cursor from offset + page size (parity with internal TS encoder).
#[must_use]
pub fn encode_page_cursor(offset: usize, page_size: usize) -> String {
    encode_cursor(&CursorData { offset, page_size })
}

/// Decode a pagination cursor into `(offset, page_size)`; `None` on garbage.
#[must_use]
pub fn decode_page_cursor(cursor: &str) -> Option<(usize, usize)> {
    decode_cursor(cursor).map(|d| (d.offset, d.page_size))
}

/// Default page size (parity with TS PaginationOptions default).
pub const DEFAULT_PAGE_SIZE: usize = 50;

/// Default max page size (parity with TS PaginationOptions default).
pub const MAX_PAGE_SIZE: usize = 100;

/// Construct pagination options.
#[must_use]
pub fn pagination_options(default_page_size: usize, max_page_size: usize) -> PaginationOptions {
    PaginationOptions {
        default_page_size,
        max_page_size,
    }
}

/// Pure: whether another page exists after `offset` with `page_size` over `total_len`.
#[must_use]
pub fn page_has_more(offset: usize, page_size: usize, total_len: usize) -> bool {
    offset.saturating_add(page_size) < total_len
}

/// Pure: next page offset (`offset + page_size`, saturating).
#[must_use]
pub fn next_page_offset(offset: usize, page_size: usize) -> usize {
    offset.saturating_add(page_size)
}

/// Empty page with no next cursor.
#[must_use]
pub fn empty_page<T>() -> PageResult<T> {
    PageResult {
        items: Vec::new(),
        next_cursor: None,
    }
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
            // Parity with TS: Math.min(data.pageSize, maxPageSize) — do not force min 1.
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


    #[test]
    fn bulk_paginate_empty_and_offset_past_end() {
        let items: Vec<i32> = vec![];
        let page = paginate(&items, None, PaginationOptions::default());
        assert!(page.items.is_empty());
        assert!(page.next_cursor.is_none());
        let items = vec![1, 2, 3];
        let cursor = encode_cursor(&CursorData { offset: 99, page_size: 2 });
        let page = paginate(&items, Some(&cursor), PaginationOptions::default());
        assert!(page.items.is_empty());
        assert!(page.next_cursor.is_none());
    }

    #[test]
    fn bulk_decode_cursor_rejects_garbage() {
        assert!(decode_cursor("!!!not-base64!!!").is_none());
        assert!(decode_cursor("").is_none());
        let good = encode_cursor(&CursorData {
            offset: 0,
            page_size: 10,
        });
        assert!(decode_cursor(&good).is_some());
    }

    #[test]
    fn wave12_clamp_and_public_cursor_roundtrip() {
        assert_eq!(clamp_page_size(0, 100), 1);
        assert_eq!(clamp_page_size(50, 100), 50);
        assert_eq!(clamp_page_size(10_000, 100), 100);
        assert_eq!(clamp_page_size(5, 0), 0);

        let c = encode_page_cursor(25, 10);
        assert_eq!(decode_page_cursor(&c), Some((25, 10)));
        assert!(decode_page_cursor("!!!").is_none());

        let items: Vec<i32> = (0..40).collect();
        let page = paginate(
            &items,
            Some(&c),
            PaginationOptions {
                default_page_size: 50,
                max_page_size: 100,
            },
        );
        assert_eq!(page.items, (25..35).collect::<Vec<i32>>());
        assert!(page.next_cursor.is_some());
    }


    #[test]
    fn wave13_page_math_and_empty_page() {
        assert_eq!(DEFAULT_PAGE_SIZE, 50);
        assert_eq!(MAX_PAGE_SIZE, 100);
        assert_eq!(PaginationOptions::default().default_page_size, DEFAULT_PAGE_SIZE);
        assert_eq!(PaginationOptions::default().max_page_size, MAX_PAGE_SIZE);

        assert!(page_has_more(0, 50, 120));
        assert!(!page_has_more(0, 50, 50));
        assert!(!page_has_more(0, 50, 10));
        assert!(!page_has_more(100, 50, 120));
        assert_eq!(next_page_offset(0, 50), 50);
        assert_eq!(next_page_offset(usize::MAX - 1, 10), usize::MAX);

        let empty: PageResult<i32> = empty_page();
        assert!(empty.items.is_empty());
        assert!(empty.next_cursor.is_none());

        let opts = pagination_options(10, 20);
        assert_eq!(opts.default_page_size, 10);
        assert_eq!(opts.max_page_size, 20);

        let items: Vec<i32> = (0..75).collect();
        let first = paginate(&items, None, PaginationOptions::default());
        assert!(page_has_more(0, DEFAULT_PAGE_SIZE, items.len()));
        assert_eq!(first.items.len(), 50);
        let cursor = match first.next_cursor.as_deref() {
            Some(c) => c,
            None => panic!("expected next cursor"),
        };
        let (off, _ps) = match decode_page_cursor(cursor) {
            Some(v) => v,
            None => panic!("expected decode"),
        };
        assert_eq!(off, next_page_offset(0, DEFAULT_PAGE_SIZE));
    }

    /// Load committed golden fixture (TS oracle surface contract) and assert Rust paginate.
    #[test]
    fn pure_residual_pagination_golden_fixture() {
        let raw = include_str!("../fixtures/pagination_golden.json");
        let doc: serde_json::Value = match serde_json::from_str(raw) {
            Ok(v) => v,
            Err(e) => panic!("pagination_golden.json: {e}"),
        };
        assert_eq!(doc["schema"], "mcp-pagination-golden/v1");
        let cases = match doc["cases"].as_array() {
            Some(a) => a,
            None => panic!("cases"),
        };
        for case in cases {
            let name = case["name"].as_str().unwrap_or("?");
            let item_count = case["itemCount"].as_u64().unwrap_or(0) as usize;
            let items: Vec<i32> = (0..item_count as i32).collect();
            let cursor = case.get("cursor").and_then(|v| {
                if v.is_null() {
                    None
                } else {
                    v.as_str()
                }
            });
            let opts = PaginationOptions {
                default_page_size: case["options"]["defaultPageSize"]
                    .as_u64()
                    .unwrap_or(50) as usize,
                max_page_size: case["options"]["maxPageSize"].as_u64().unwrap_or(100) as usize,
            };
            let page = paginate(&items, cursor, opts);
            let expected_len = case["expectedLen"].as_u64().unwrap_or(0) as usize;
            assert_eq!(page.items.len(), expected_len, "case {name} len");
            if let Some(first) = case.get("expectedFirst").and_then(|v| v.as_i64()) {
                if !page.items.is_empty() {
                    assert_eq!(i64::from(page.items[0]), first, "case {name} first");
                }
            }
            let has_next = case["hasNext"].as_bool().unwrap_or(false);
            assert_eq!(page.next_cursor.is_some(), has_next, "case {name} next");
        }
        if let Some(math) = doc.get("pageMath").and_then(|v| v.as_array()) {
            for case in math {
                let name = case["name"].as_str().unwrap_or("?");
                let offset = case["offset"].as_u64().unwrap_or(0) as usize;
                let page_size = case["pageSize"].as_u64().unwrap_or(0) as usize;
                let total = case["total"].as_u64().unwrap_or(0) as usize;
                let has_more = case["hasMore"].as_bool().unwrap_or(false);
                assert_eq!(
                    page_has_more(offset, page_size, total),
                    has_more,
                    "case {name}"
                );
                if let Some(next) = case.get("nextOffset").and_then(|v| v.as_u64()) {
                    assert_eq!(
                        next_page_offset(offset, page_size),
                        next as usize,
                        "case {name} next"
                    );
                }
            }
        }
        if let Some(defs) = doc.get("defaults") {
            assert_eq!(
                defs["defaultPageSize"].as_u64().unwrap_or(0) as usize,
                DEFAULT_PAGE_SIZE
            );
            assert_eq!(
                defs["maxPageSize"].as_u64().unwrap_or(0) as usize,
                MAX_PAGE_SIZE
            );
        }
    }
}
