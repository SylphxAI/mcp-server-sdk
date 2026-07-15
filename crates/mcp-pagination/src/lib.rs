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

/// True when `cursor` decodes as a valid page cursor.
#[must_use]
pub fn is_valid_page_cursor(cursor: &str) -> bool {
    decode_page_cursor(cursor).is_some()
}

/// Exclusive end index of a page slice (`min(offset + page_size, total_len)`).
#[must_use]
pub fn page_end(offset: usize, page_size: usize, total_len: usize) -> usize {
    offset.saturating_add(page_size).min(total_len)
}

/// Items remaining after `offset` (saturating).
#[must_use]
pub fn items_remaining(offset: usize, total_len: usize) -> usize {
    total_len.saturating_sub(offset)
}

// ============================================================================
// WAVE15 pure residual page math
// ============================================================================

/// Number of items in the page slice (`page_end - clamp_offset`).
#[must_use]
pub fn page_item_count(offset: usize, page_size: usize, total_len: usize) -> usize {
    let start = clamp_offset(offset, total_len);
    page_end(offset, page_size, total_len).saturating_sub(start)
}

/// Ceiling pages needed to cover `total_len` at `page_size` (`0` when page_size is 0).
#[must_use]
pub fn pages_needed(total_len: usize, page_size: usize) -> usize {
    if page_size == 0 {
        return 0;
    }
    if total_len == 0 {
        return 0;
    }
    total_len.div_ceil(page_size)
}

/// Clamp `offset` into `[0, total_len]`.
#[must_use]
pub fn clamp_offset(offset: usize, total_len: usize) -> usize {
    offset.min(total_len)
}

/// Inclusive-start exclusive-end bounds for a page slice: `(start, end)`.
#[must_use]
pub fn page_bounds(offset: usize, page_size: usize, total_len: usize) -> (usize, usize) {
    let start = clamp_offset(offset, total_len);
    let end = page_end(offset, page_size, total_len);
    (start, end)
}

/// Encode the next-page cursor when another page exists; `None` at end.
#[must_use]
pub fn next_page_cursor(offset: usize, page_size: usize, total_len: usize) -> Option<String> {
    if page_has_more(offset, page_size, total_len) {
        Some(encode_page_cursor(next_page_offset(offset, page_size), page_size))
    } else {
        None
    }
}

// ============================================================================
// WAVE16 pure residual page index math
// ============================================================================

/// Zero-based page index for `offset` at `page_size` (`0` when `page_size` is 0).
#[must_use]
pub fn page_index(offset: usize, page_size: usize) -> usize {
    if page_size == 0 {
        return 0;
    }
    offset / page_size
}

/// Offset of the first item on zero-based `page` at `page_size`.
#[must_use]
pub fn offset_for_page(page: usize, page_size: usize) -> usize {
    page.saturating_mul(page_size)
}

/// True when `offset` is at the start of the collection.
#[must_use]
pub fn is_first_page(offset: usize) -> bool {
    offset == 0
}

/// True when no further page exists after this window (`!page_has_more`).
#[must_use]
pub fn is_last_page(offset: usize, page_size: usize, total_len: usize) -> bool {
    !page_has_more(offset, page_size, total_len)
}

/// Pages still remaining **after** the current page window (not including current).
///
/// `0` when on/after the last page or when `page_size` is 0.
#[must_use]
pub fn pages_remaining(offset: usize, page_size: usize, total_len: usize) -> usize {
    if page_size == 0 {
        return 0;
    }
    let end = page_end(offset, page_size, total_len);
    if end >= total_len {
        return 0;
    }
    pages_needed(total_len.saturating_sub(end), page_size)
}

/// Item count on the final page (`0` when empty or `page_size` is 0).
#[must_use]
pub fn items_on_last_page(total_len: usize, page_size: usize) -> usize {
    if page_size == 0 || total_len == 0 {
        return 0;
    }
    let rem = total_len % page_size;
    if rem == 0 {
        page_size
    } else {
        rem
    }
}

// --- WAVE17 pure residual ---

/// Maximum page size constant used when options omit a max (default 100).
pub const DEFAULT_MAX_PAGE_SIZE: usize = 100;

/// Build default pagination options (50 / 100) matching TS defaults.
#[must_use]
pub fn default_pagination_options() -> PaginationOptions {
    PaginationOptions {
        default_page_size: DEFAULT_PAGE_SIZE,
        max_page_size: DEFAULT_MAX_PAGE_SIZE,
    }
}

/// True when offset points past the end of the collection.
#[must_use]
pub fn is_offset_past_end(offset: usize, total_len: usize) -> bool {
    offset >= total_len
}

/// Normalize requested page size: clamp to `[1, max]` when requested is 0 → max default path.
/// Product TS keeps `0` as-is after min with max; this helper documents the pure residual clamp for
/// non-zero requested sizes only.
#[must_use]
pub fn effective_page_size(requested: usize, default_page_size: usize, max_page_size: usize) -> usize {
    if requested == 0 {
        default_page_size.min(max_page_size)
    } else {
        requested.min(max_page_size)
    }
}

/// Whether a page of items is empty.
#[must_use]
pub fn page_is_empty<T>(page: &PageResult<T>) -> bool {
    page.items.is_empty()
}

/// Whether a page carries a next cursor.
#[must_use]
pub fn page_has_next_cursor<T>(page: &PageResult<T>) -> bool {
    page.next_cursor.is_some()
}

// --- WAVE18 pure residual ---

/// Decode cursor to offset only (None if invalid).
#[must_use]
pub fn cursor_offset(cursor: &str) -> Option<usize> {
    decode_page_cursor(cursor).map(|(offset, _)| offset)
}

/// Decode cursor to page size only (None if invalid).
#[must_use]
pub fn cursor_page_size(cursor: &str) -> Option<usize> {
    decode_page_cursor(cursor).map(|(_, size)| size)
}

/// True when a cursor string is present, non-empty, and valid.
#[must_use]
pub fn cursor_present_and_valid(cursor: Option<&str>) -> bool {
    match cursor {
        Some(c) if !c.is_empty() => is_valid_page_cursor(c),
        _ => false,
    }
}

/// Compute next offset after consuming a page (saturating).
#[must_use]
pub fn advance_offset(offset: usize, page_size: usize) -> usize {
    offset.saturating_add(page_size)
}

/// Whether requesting `page_size` from `offset` would return any items.
#[must_use]
pub fn would_return_items(offset: usize, page_size: usize, total_len: usize) -> bool {
    page_size > 0 && offset < total_len
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


// --- WAVE19 pure residual ---

/// Remaining item count after `offset` (0 when past end).
#[must_use]
pub fn remaining_items(offset: usize, total_len: usize) -> usize {
    total_len.saturating_sub(offset)
}

/// Inclusive start / exclusive end window for a page, clamped to collection bounds.
#[must_use]
pub fn page_window(offset: usize, page_size: usize, total_len: usize) -> (usize, usize) {
    let start = clamp_offset(offset, total_len);
    let end = start.saturating_add(page_size).min(total_len);
    (start, end)
}

/// True when two optional cursors are equal (both None, or same non-empty string).
#[must_use]
pub fn cursors_equal(a: Option<&str>, b: Option<&str>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(x), Some(y)) => x == y,
        _ => false,
    }
}

/// Normalize cursor option: empty/whitespace-only → None.
#[must_use]
pub fn normalize_cursor_option(cursor: Option<&str>) -> Option<&str> {
    cursor.and_then(|c| {
        let t = c.trim();
        if t.is_empty() { None } else { Some(t) }
    })
}

/// True when page_size is within (0, max] inclusive of max, exclusive of 0.
#[must_use]
pub fn is_valid_page_size(page_size: usize, max_page_size: usize) -> bool {
    page_size > 0 && page_size <= max_page_size
}



// --- WAVE20 pure residual ---

/// Offset for the next page after this window, or None if last page.
/// Dual-oracle of "has more → next offset = end" without encoding a cursor.
#[must_use]
pub fn next_offset_after_page(offset: usize, page_size: usize, total_len: usize) -> Option<usize> {
    let (_start, end) = page_window(offset, page_size, total_len);
    if end >= total_len {
        None
    } else {
        Some(end)
    }
}

/// True when remaining items after offset is zero (empty page / past end).
#[must_use]
pub fn is_empty_page(offset: usize, total_len: usize) -> bool {
    remaining_items(offset, total_len) == 0
}

/// Inclusive coverage fraction of a page over the collection: items_on_page / total
/// (0.0 when total is 0).
#[must_use]
pub fn page_coverage_ratio(offset: usize, page_size: usize, total_len: usize) -> f64 {
    if total_len == 0 {
        return 0.0;
    }
    let n = page_item_count(offset, page_size, total_len) as f64;
    n / (total_len as f64)
}

/// True when a previous page exists (offset > 0 after clamp).
#[must_use]
pub fn has_previous_page(offset: usize, total_len: usize) -> bool {
    clamp_offset(offset, total_len) > 0
}

/// Previous page offset stepping back by page_size (saturating at 0).
#[must_use]
pub fn previous_offset(offset: usize, page_size: usize) -> usize {
    offset.saturating_sub(page_size)
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

    #[test]
    fn wave14_page_end_remaining_and_cursor_valid() {
        assert_eq!(page_end(0, 50, 120), 50);
        assert_eq!(page_end(100, 50, 120), 120);
        assert_eq!(page_end(0, 50, 10), 10);
        assert_eq!(page_end(200, 50, 120), 120);

        assert_eq!(items_remaining(0, 120), 120);
        assert_eq!(items_remaining(50, 120), 70);
        assert_eq!(items_remaining(200, 120), 0);

        let c = encode_page_cursor(10, 20);
        assert!(is_valid_page_cursor(&c));
        assert!(!is_valid_page_cursor("!!!"));
        assert!(!is_valid_page_cursor(""));

        // page_end aligns with paginate slice length for first page
        let items: Vec<i32> = (0..75).collect();
        let page = paginate(&items, None, PaginationOptions::default());
        assert_eq!(
            page.items.len(),
            page_end(0, DEFAULT_PAGE_SIZE, items.len())
        );
    }

    #[test]
    fn wave15_page_item_count_bounds_and_next_cursor() {
        assert_eq!(page_item_count(0, 50, 120), 50);
        assert_eq!(page_item_count(100, 50, 120), 20);
        assert_eq!(page_item_count(200, 50, 120), 0);
        assert_eq!(page_item_count(0, 50, 10), 10);

        assert_eq!(pages_needed(120, 50), 3);
        assert_eq!(pages_needed(100, 50), 2);
        assert_eq!(pages_needed(0, 50), 0);
        assert_eq!(pages_needed(10, 0), 0);
        assert_eq!(pages_needed(1, 50), 1);

        assert_eq!(clamp_offset(0, 10), 0);
        assert_eq!(clamp_offset(5, 10), 5);
        assert_eq!(clamp_offset(99, 10), 10);

        assert_eq!(page_bounds(0, 50, 120), (0, 50));
        assert_eq!(page_bounds(100, 50, 120), (100, 120));
        assert_eq!(page_bounds(200, 50, 120), (120, 120));

        let next = match next_page_cursor(0, 50, 120) {
            Some(c) => c,
            None => panic!("expected next cursor"),
        };
        assert_eq!(decode_page_cursor(&next), Some((50, 50)));
        assert!(next_page_cursor(100, 50, 120).is_none());
        assert!(next_page_cursor(0, 50, 10).is_none());

        // Align with paginate next_cursor
        let items: Vec<i32> = (0..120).collect();
        let page = paginate(&items, None, PaginationOptions::default());
        assert_eq!(
            page.next_cursor.as_deref(),
            next_page_cursor(0, DEFAULT_PAGE_SIZE, items.len()).as_deref()
        );
    }

    #[test]
    fn wave16_page_index_remaining_and_last_page() {
        assert_eq!(page_index(0, 50), 0);
        assert_eq!(page_index(50, 50), 1);
        assert_eq!(page_index(100, 50), 2);
        assert_eq!(page_index(49, 50), 0);
        assert_eq!(page_index(10, 0), 0);

        assert_eq!(offset_for_page(0, 50), 0);
        assert_eq!(offset_for_page(2, 50), 100);
        assert_eq!(offset_for_page(3, 0), 0);

        assert!(is_first_page(0));
        assert!(!is_first_page(1));

        assert!(!is_last_page(0, 50, 120));
        assert!(is_last_page(100, 50, 120));
        assert!(is_last_page(0, 50, 10));
        assert!(is_last_page(0, 50, 0));

        assert_eq!(pages_remaining(0, 50, 120), 2);
        assert_eq!(pages_remaining(50, 50, 120), 1);
        assert_eq!(pages_remaining(100, 50, 120), 0);
        assert_eq!(pages_remaining(0, 50, 50), 0);
        assert_eq!(pages_remaining(0, 0, 120), 0);

        assert_eq!(items_on_last_page(120, 50), 20);
        assert_eq!(items_on_last_page(100, 50), 50);
        assert_eq!(items_on_last_page(0, 50), 0);
        assert_eq!(items_on_last_page(10, 0), 0);
        assert_eq!(items_on_last_page(1, 50), 1);

        // Round-trip with page_index / offset_for_page
        for page in 0..3 {
            let off = offset_for_page(page, 50);
            assert_eq!(page_index(off, 50), page);
        }
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
        if let Some(cases) = doc.get("pageEnd").and_then(|v| v.as_array()) {
            for case in cases {
                let name = case["name"].as_str().unwrap_or("?");
                let offset = case["offset"].as_u64().unwrap_or(0) as usize;
                let page_size = case["pageSize"].as_u64().unwrap_or(0) as usize;
                let total = case["total"].as_u64().unwrap_or(0) as usize;
                let expected = case["expectedEnd"].as_u64().unwrap_or(0) as usize;
                assert_eq!(
                    page_end(offset, page_size, total),
                    expected,
                    "case {name}"
                );
                if let Some(rem) = case.get("remaining").and_then(|v| v.as_u64()) {
                    assert_eq!(
                        items_remaining(offset, total),
                        rem as usize,
                        "case {name} remaining"
                    );
                }
            }
        }
        if let Some(cases) = doc.get("cursorValid").and_then(|v| v.as_array()) {
            for case in cases {
                let name = case["name"].as_str().unwrap_or("?");
                let expected = case["valid"].as_bool().unwrap_or(false);
                if let Some(cursor) = case.get("cursor").and_then(|v| v.as_str()) {
                    assert_eq!(is_valid_page_cursor(cursor), expected, "case {name}");
                } else if case.get("encode").is_some() {
                    let offset = case["encode"]["offset"].as_u64().unwrap_or(0) as usize;
                    let page_size = case["encode"]["pageSize"].as_u64().unwrap_or(0) as usize;
                    let c = encode_page_cursor(offset, page_size);
                    assert_eq!(is_valid_page_cursor(&c), expected, "case {name}");
                }
            }
        }
        if let Some(cases) = doc.get("pageItemCount").and_then(|v| v.as_array()) {
            for case in cases {
                let name = case["name"].as_str().unwrap_or("?");
                let offset = case["offset"].as_u64().unwrap_or(0) as usize;
                let page_size = case["pageSize"].as_u64().unwrap_or(0) as usize;
                let total = case["total"].as_u64().unwrap_or(0) as usize;
                let expected = case["expectedCount"].as_u64().unwrap_or(0) as usize;
                assert_eq!(
                    page_item_count(offset, page_size, total),
                    expected,
                    "case {name}"
                );
                if let Some(pages) = case.get("pagesNeeded").and_then(|v| v.as_u64()) {
                    assert_eq!(
                        pages_needed(total, page_size),
                        pages as usize,
                        "case {name} pages"
                    );
                }
                if let Some(bounds) = case.get("bounds").and_then(|v| v.as_array()) {
                    if bounds.len() == 2 {
                        let start = bounds[0].as_u64().unwrap_or(0) as usize;
                        let end = bounds[1].as_u64().unwrap_or(0) as usize;
                        assert_eq!(
                            page_bounds(offset, page_size, total),
                            (start, end),
                            "case {name} bounds"
                        );
                    }
                }
            }
        }
        if let Some(cases) = doc.get("nextPageCursor").and_then(|v| v.as_array()) {
            for case in cases {
                let name = case["name"].as_str().unwrap_or("?");
                let offset = case["offset"].as_u64().unwrap_or(0) as usize;
                let page_size = case["pageSize"].as_u64().unwrap_or(0) as usize;
                let total = case["total"].as_u64().unwrap_or(0) as usize;
                let has = case["hasNext"].as_bool().unwrap_or(false);
                let got = next_page_cursor(offset, page_size, total);
                assert_eq!(got.is_some(), has, "case {name}");
                if has {
                    if let Some(exp_off) = case.get("nextOffset").and_then(|v| v.as_u64()) {
                        let c = match got.as_deref() {
                            Some(c) => c,
                            None => panic!("expected cursor {name}"),
                        };
                        let (off, ps) = match decode_page_cursor(c) {
                            Some(v) => v,
                            None => panic!("decode {name}"),
                        };
                        assert_eq!(off, exp_off as usize, "case {name} nextOffset");
                        assert_eq!(ps, page_size, "case {name} pageSize");
                    }
                }
            }
        }
        // WAVE16: page index / remaining / last-page math
        if let Some(cases) = doc.get("pageIndex").and_then(|v| v.as_array()) {
            for case in cases {
                let name = case["name"].as_str().unwrap_or("?");
                let offset = case["offset"].as_u64().unwrap_or(0) as usize;
                let page_size = case["pageSize"].as_u64().unwrap_or(0) as usize;
                let expected = case["pageIndex"].as_u64().unwrap_or(0) as usize;
                assert_eq!(page_index(offset, page_size), expected, "page_index {name}");
                if let Some(first) = case.get("isFirst").and_then(|v| v.as_bool()) {
                    assert_eq!(is_first_page(offset), first, "is_first {name}");
                }
            }
        }
        if let Some(cases) = doc.get("offsetForPage").and_then(|v| v.as_array()) {
            for case in cases {
                let page = case["page"].as_u64().unwrap_or(0) as usize;
                let page_size = case["pageSize"].as_u64().unwrap_or(0) as usize;
                let expected = case["offset"].as_u64().unwrap_or(0) as usize;
                assert_eq!(
                    offset_for_page(page, page_size),
                    expected,
                    "offset_for_page {page}"
                );
            }
        }
        if let Some(cases) = doc.get("pagesRemaining").and_then(|v| v.as_array()) {
            for case in cases {
                let name = case["name"].as_str().unwrap_or("?");
                let offset = case["offset"].as_u64().unwrap_or(0) as usize;
                let page_size = case["pageSize"].as_u64().unwrap_or(0) as usize;
                let total = case["total"].as_u64().unwrap_or(0) as usize;
                let remaining = case["remaining"].as_u64().unwrap_or(0) as usize;
                assert_eq!(
                    pages_remaining(offset, page_size, total),
                    remaining,
                    "pages_remaining {name}"
                );
                if let Some(last) = case.get("isLast").and_then(|v| v.as_bool()) {
                    assert_eq!(
                        is_last_page(offset, page_size, total),
                        last,
                        "is_last {name}"
                    );
                }
            }
        }
        if let Some(cases) = doc.get("itemsOnLastPage").and_then(|v| v.as_array()) {
            for case in cases {
                let total = case["total"].as_u64().unwrap_or(0) as usize;
                let page_size = case["pageSize"].as_u64().unwrap_or(0) as usize;
                let expected = case["expected"].as_u64().unwrap_or(0) as usize;
                assert_eq!(
                    items_on_last_page(total, page_size),
                    expected,
                    "items_on_last_page {total}/{page_size}"
                );
            }
        }
    }

    #[test]
    fn wave17_defaults_offset_and_page_flags() {
        let opts = default_pagination_options();
        assert_eq!(opts.default_page_size, DEFAULT_PAGE_SIZE);
        assert_eq!(opts.max_page_size, DEFAULT_MAX_PAGE_SIZE);
        assert!(is_offset_past_end(10, 10));
        assert!(!is_offset_past_end(9, 10));
        assert_eq!(effective_page_size(0, 50, 100), 50);
        assert_eq!(effective_page_size(200, 50, 100), 100);
        assert_eq!(effective_page_size(25, 50, 100), 25);

        let items: Vec<i32> = (0..5).collect();
        let page = paginate(&items, None, PaginationOptions {
            default_page_size: 2,
            max_page_size: 100,
        });
        assert!(!page_is_empty(&page));
        assert!(page_has_next_cursor(&page));
        let empty = empty_page::<i32>();
        assert!(page_is_empty(&empty));
        assert!(!page_has_next_cursor(&empty));
    }

    #[test]
    fn wave18_cursor_offset_size_and_advance() {
        let cur = encode_page_cursor(20, 10);
        assert_eq!(cursor_offset(&cur), Some(20));
        assert_eq!(cursor_page_size(&cur), Some(10));
        assert!(cursor_present_and_valid(Some(&cur)));
        assert!(!cursor_present_and_valid(None));
        assert!(!cursor_present_and_valid(Some("")));
        assert!(!cursor_present_and_valid(Some("!!!")));
        assert_eq!(advance_offset(20, 10), 30);
        assert!(would_return_items(0, 10, 5));
        assert!(!would_return_items(5, 10, 5));
        assert!(!would_return_items(0, 0, 5));
    }
    #[test]
    fn wave19_remaining_window_cursor_normalize() {
        assert_eq!(remaining_items(0, 10), 10);
        assert_eq!(remaining_items(7, 10), 3);
        assert_eq!(remaining_items(10, 10), 0);
        assert_eq!(remaining_items(12, 10), 0);

        assert_eq!(page_window(0, 50, 120), (0, 50));
        assert_eq!(page_window(100, 50, 120), (100, 120));
        assert_eq!(page_window(200, 50, 120), (120, 120));
        assert_eq!(page_window(0, 50, 0), (0, 0));

        assert!(cursors_equal(None, None));
        assert!(cursors_equal(Some("a"), Some("a")));
        assert!(!cursors_equal(Some("a"), Some("b")));
        assert!(!cursors_equal(None, Some("a")));

        assert_eq!(normalize_cursor_option(Some("  abc  ")), Some("abc"));
        assert!(normalize_cursor_option(Some("   ")).is_none());
        assert!(normalize_cursor_option(None).is_none());
        assert!(normalize_cursor_option(Some("")).is_none());

        assert!(is_valid_page_size(1, 50));
        assert!(is_valid_page_size(50, 50));
        assert!(!is_valid_page_size(0, 50));
        assert!(!is_valid_page_size(51, 50));
    }


    #[test]
    fn wave20_page_window_helpers() {
        assert_eq!(next_offset_after_page(0, 50, 120), Some(50));
        assert_eq!(next_offset_after_page(100, 50, 120), None);
        assert!(is_empty_page(10, 10));
        assert!(!is_empty_page(0, 10));
        assert!((page_coverage_ratio(0, 50, 100) - 0.5).abs() < 1e-12);
        assert_eq!(page_coverage_ratio(0, 50, 0), 0.0);
        assert!(has_previous_page(50, 100));
        assert!(!has_previous_page(0, 100));
        assert_eq!(previous_offset(50, 50), 0);
        assert_eq!(previous_offset(10, 50), 0);
        assert_eq!(previous_offset(120, 50), 70);
    }
}
