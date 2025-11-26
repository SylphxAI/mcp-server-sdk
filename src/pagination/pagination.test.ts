import { describe, expect, test } from "bun:test"
import {
	collectAllPages,
	createPaginatedHandler,
	decodeCursor,
	encodeCursor,
	iteratePages,
	paginate,
} from "./index.js"

describe("Pagination", () => {
	describe("encodeCursor/decodeCursor", () => {
		test("encodes and decodes cursor data", () => {
			const data = { offset: 10, pageSize: 20 }
			const cursor = encodeCursor(data)
			const decoded = decodeCursor(cursor)

			expect(decoded).toEqual(data)
		})

		test("returns null for invalid cursor", () => {
			expect(decodeCursor("invalid")).toBe(null)
			expect(decodeCursor("")).toBe(null)
		})

		test("returns null for malformed JSON", () => {
			const cursor = Buffer.from("not json").toString("base64url")
			expect(decodeCursor(cursor)).toBe(null)
		})

		test("returns null for missing fields", () => {
			const cursor = Buffer.from('{"offset": 10}').toString("base64url")
			expect(decodeCursor(cursor)).toBe(null)
		})
	})

	describe("paginate", () => {
		const items = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]

		test("returns first page without cursor", () => {
			const result = paginate(items, undefined, { defaultPageSize: 3 })

			expect(result.items).toEqual([1, 2, 3])
			expect(result.nextCursor).toBeDefined()
		})

		test("returns next page with cursor", () => {
			const page1 = paginate(items, undefined, { defaultPageSize: 3 })
			const page2 = paginate(items, page1.nextCursor, { defaultPageSize: 3 })

			expect(page2.items).toEqual([4, 5, 6])
			expect(page2.nextCursor).toBeDefined()
		})

		test("returns no cursor on last page", () => {
			const page1 = paginate(items, undefined, { defaultPageSize: 5 })
			const page2 = paginate(items, page1.nextCursor, { defaultPageSize: 5 })

			expect(page2.items).toEqual([6, 7, 8, 9, 10])
			expect(page2.nextCursor).toBeUndefined()
		})

		test("respects maxPageSize", () => {
			// Encode a cursor with large pageSize
			const cursor = encodeCursor({ offset: 0, pageSize: 1000 })
			const result = paginate(items, cursor, { maxPageSize: 5 })

			expect(result.items).toHaveLength(5)
		})

		test("uses default page size", () => {
			const result = paginate(items, undefined, { defaultPageSize: 50 })
			expect(result.items).toEqual(items) // All items fit
		})

		test("empty items returns empty result", () => {
			const result = paginate([], undefined)
			expect(result.items).toEqual([])
			expect(result.nextCursor).toBeUndefined()
		})
	})

	describe("createPaginatedHandler", () => {
		test("creates handler that paginates", async () => {
			const handler = createPaginatedHandler(() => [1, 2, 3, 4, 5], { defaultPageSize: 2 })

			const page1 = await handler()
			expect(page1.items).toEqual([1, 2])

			const page2 = await handler(page1.nextCursor)
			expect(page2.items).toEqual([3, 4])

			const page3 = await handler(page2.nextCursor)
			expect(page3.items).toEqual([5])
			expect(page3.nextCursor).toBeUndefined()
		})

		test("works with async getter", async () => {
			const handler = createPaginatedHandler(async () => [1, 2, 3], { defaultPageSize: 10 })

			const result = await handler()
			expect(result.items).toEqual([1, 2, 3])
		})
	})

	describe("iteratePages", () => {
		test("iterates through all pages", async () => {
			const pages: number[][] = []
			let _cursor: string | undefined

			const fetchPage = async (c?: string) => {
				const items = [1, 2, 3, 4, 5]
				return paginate(items, c, { defaultPageSize: 2 })
			}

			for await (const items of iteratePages(fetchPage)) {
				pages.push([...items] as number[])
			}

			expect(pages).toEqual([[1, 2], [3, 4], [5]])
		})
	})

	describe("collectAllPages", () => {
		test("collects all items into single array", async () => {
			const fetchPage = async (c?: string) => {
				const items = [1, 2, 3, 4, 5]
				return paginate(items, c, { defaultPageSize: 2 })
			}

			const all = await collectAllPages(fetchPage)

			expect(all).toEqual([1, 2, 3, 4, 5])
		})
	})
})
