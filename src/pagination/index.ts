/**
 * Pagination Utilities
 *
 * Cursor-based pagination for MCP list operations.
 */

// ============================================================================
// Types
// ============================================================================

export interface PaginationOptions {
	/** Default page size if not specified */
	readonly defaultPageSize?: number
	/** Maximum page size allowed */
	readonly maxPageSize?: number
}

export interface PageResult<T> {
	readonly items: readonly T[]
	readonly nextCursor?: string
}

interface CursorData {
	readonly offset: number
	readonly pageSize: number
}

// ============================================================================
// Internal
// ============================================================================

const encodeCursor = (data: CursorData): string => {
	return Buffer.from(JSON.stringify(data)).toString("base64url")
}

const decodeCursor = (cursor: string): CursorData | null => {
	try {
		const json = Buffer.from(cursor, "base64url").toString("utf-8")
		const data = JSON.parse(json) as CursorData
		if (typeof data.offset !== "number" || typeof data.pageSize !== "number") {
			return null
		}
		return data
	} catch {
		return null
	}
}

// ============================================================================
// Pagination
// ============================================================================

/**
 * Paginate an array of items.
 *
 * @example
 * ```ts
 * const result = paginate(allItems, cursor, { defaultPageSize: 10 })
 * // { items: [...], nextCursor: "..." }
 * ```
 */
export const paginate = <T>(
	items: readonly T[],
	cursor?: string,
	options?: PaginationOptions
): PageResult<T> => {
	const defaultPageSize = options?.defaultPageSize ?? 50
	const maxPageSize = options?.maxPageSize ?? 100

	let offset = 0
	let pageSize = defaultPageSize

	if (cursor) {
		const data = decodeCursor(cursor)
		if (data) {
			offset = data.offset
			pageSize = Math.min(data.pageSize, maxPageSize)
		}
	}

	const page = items.slice(offset, offset + pageSize)
	const hasMore = offset + pageSize < items.length

	return {
		items: page,
		nextCursor: hasMore ? encodeCursor({ offset: offset + pageSize, pageSize }) : undefined,
	}
}
