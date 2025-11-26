import { describe, expect, test } from "bun:test"
import * as Rpc from "./jsonrpc.js"

describe("JSON-RPC", () => {
	describe("constructors", () => {
		test("request creates valid request", () => {
			const req = Rpc.request(1, "test", { foo: "bar" })
			expect(req).toEqual({
				jsonrpc: "2.0",
				id: 1,
				method: "test",
				params: { foo: "bar" },
			})
		})

		test("request without params omits params", () => {
			const req = Rpc.request(1, "test")
			expect(req).toEqual({
				jsonrpc: "2.0",
				id: 1,
				method: "test",
			})
			expect("params" in req).toBe(false)
		})

		test("notification creates valid notification", () => {
			const notif = Rpc.notification("event", { data: 123 })
			expect(notif).toEqual({
				jsonrpc: "2.0",
				method: "event",
				params: { data: 123 },
			})
			expect("id" in notif).toBe(false)
		})

		test("success creates valid success response", () => {
			const res = Rpc.success(1, { result: "ok" })
			expect(res).toEqual({
				jsonrpc: "2.0",
				id: 1,
				result: { result: "ok" },
			})
		})

		test("error creates valid error response", () => {
			const res = Rpc.error(1, -32600, "Invalid Request", { detail: "x" })
			expect(res).toEqual({
				jsonrpc: "2.0",
				id: 1,
				error: {
					code: -32600,
					message: "Invalid Request",
					data: { detail: "x" },
				},
			})
		})

		test("error with null id", () => {
			const res = Rpc.error(null, -32700, "Parse error")
			expect(res.id).toBe(null)
		})
	})

	describe("type guards", () => {
		test("isRequest identifies requests", () => {
			const req = Rpc.request(1, "test")
			const notif = Rpc.notification("test")
			const res = Rpc.success(1, {})

			expect(Rpc.isRequest(req)).toBe(true)
			expect(Rpc.isRequest(notif)).toBe(false)
			expect(Rpc.isRequest(res)).toBe(false)
		})

		test("isNotification identifies notifications", () => {
			const req = Rpc.request(1, "test")
			const notif = Rpc.notification("test")

			expect(Rpc.isNotification(notif)).toBe(true)
			expect(Rpc.isNotification(req)).toBe(false)
		})

		test("isResponse identifies responses", () => {
			const req = Rpc.request(1, "test")
			const success = Rpc.success(1, {})
			const err = Rpc.error(1, -32600, "error")

			expect(Rpc.isResponse(success)).toBe(true)
			expect(Rpc.isResponse(err)).toBe(true)
			expect(Rpc.isResponse(req)).toBe(false)
		})

		test("isSuccess identifies success responses", () => {
			const success = Rpc.success(1, {})
			const err = Rpc.error(1, -32600, "error")

			expect(Rpc.isSuccess(success)).toBe(true)
			expect(Rpc.isSuccess(err)).toBe(false)
		})

		test("isError identifies error responses", () => {
			const success = Rpc.success(1, {})
			const err = Rpc.error(1, -32600, "error")

			expect(Rpc.isError(err)).toBe(true)
			expect(Rpc.isError(success)).toBe(false)
		})
	})

	describe("parseMessage", () => {
		test("parses valid request", () => {
			const input = JSON.stringify(Rpc.request(1, "test", {}))
			const result = Rpc.parseMessage(input)

			expect(result.ok).toBe(true)
			if (result.ok) {
				expect(Rpc.isRequest(result.value)).toBe(true)
			}
		})

		test("fails on invalid JSON", () => {
			const result = Rpc.parseMessage("{invalid}")
			expect(result.ok).toBe(false)
		})

		test("fails on non-object", () => {
			const result = Rpc.parseMessage('"string"')
			expect(result.ok).toBe(false)
			if (!result.ok) {
				expect(result.error).toContain("must be an object")
			}
		})

		test("fails on wrong jsonrpc version", () => {
			const result = Rpc.parseMessage('{"jsonrpc":"1.0","id":1,"method":"x"}')
			expect(result.ok).toBe(false)
			if (!result.ok) {
				expect(result.error).toContain("Invalid jsonrpc version")
			}
		})
	})

	describe("stringify", () => {
		test("serializes message to JSON", () => {
			const req = Rpc.request(1, "test")
			const str = Rpc.stringify(req)
			expect(JSON.parse(str)).toEqual(req)
		})
	})
})
