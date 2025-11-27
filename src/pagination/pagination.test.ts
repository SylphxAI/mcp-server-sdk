import { describe, expect, test } from "bun:test"
import { paginate } from "./index.js"

describe("Pagination", () => {
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

		test("uses default page size", () => {
			const result = paginate(items, undefined, { defaultPageSize: 50 })
			expect(result.items).toEqual(items)
		})

		test("empty items returns empty result", () => {
			const result = paginate([], undefined)
			expect(result.items).toEqual([])
			expect(result.nextCursor).toBeUndefined()
		})

		test("handles invalid cursor gracefully", () => {
			const result = paginate(items, "invalid-cursor", { defaultPageSize: 3 })
			expect(result.items).toEqual([1, 2, 3])
		})
	})
})
