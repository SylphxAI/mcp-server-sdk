/**
 * Pagination Utilities
 *
 * Cursor-based pagination helpers.
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

export interface CursorData {
	readonly offset: number
	readonly pageSize: number
}

// ============================================================================
// Cursor Encoding
// ============================================================================

/**
 * Encode pagination cursor.
 */
export const encodeCursor = (data: CursorData): string => {
	return Buffer.from(JSON.stringify(data)).toString("base64url")
}

/**
 * Decode pagination cursor.
 */
export const decodeCursor = (cursor: string): CursorData | null => {
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
// Pagination Helpers
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

/**
 * Create a paginated list handler.
 *
 * @example
 * ```ts
 * const handler = createPaginatedHandler(
 *   () => getAllResources(),
 *   { defaultPageSize: 20 },
 * )
 * const result = await handler(cursor)
 * ```
 */
export const createPaginatedHandler = <T>(
	getItems: () => readonly T[] | Promise<readonly T[]>,
	options?: PaginationOptions
) => {
	return async (cursor?: string): Promise<PageResult<T>> => {
		const items = await getItems()
		return paginate(items, cursor, options)
	}
}

/**
 * Iterate through all pages.
 *
 * @example
 * ```ts
 * for await (const items of iteratePages(fetchPage)) {
 *   console.log(items)
 * }
 * ```
 */
export async function* iteratePages<T>(
	fetchPage: (cursor?: string) => Promise<PageResult<T>>
): AsyncGenerator<readonly T[], void, unknown> {
	let cursor: string | undefined

	do {
		const result = await fetchPage(cursor)
		yield result.items
		cursor = result.nextCursor
	} while (cursor)
}

/**
 * Collect all pages into a single array.
 */
export const collectAllPages = async <T>(
	fetchPage: (cursor?: string) => Promise<PageResult<T>>
): Promise<T[]> => {
	const all: T[] = []
	for await (const items of iteratePages(fetchPage)) {
		all.push(...items)
	}
	return all
}
