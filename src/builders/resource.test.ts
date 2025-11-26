import { describe, expect, test } from "bun:test"
import {
	resource,
	resourceTemplate,
	resourceText,
	resourceBlob,
	resourceContents,
	matchesTemplate,
	extractParams,
	toProtocolResource,
	toProtocolTemplate,
} from "./resource.js"

describe("Resource Builder", () => {
	describe("resource", () => {
		test("creates static resource", () => {
			const r = resource({
				uri: "file:///test.txt",
				name: "Test File",
				handler: () => () => resourceText("file:///test.txt", "content"),
			})

			expect(r.uri).toBe("file:///test.txt")
			expect(r.name).toBe("Test File")
			expect(r.type).toBe("static")
		})

		test("creates resource with optional fields", () => {
			const r = resource({
				uri: "file:///data.json",
				name: "Data",
				description: "JSON data file",
				mimeType: "application/json",
				handler: () => () => resourceText("file:///data.json", "{}"),
			})

			expect(r.description).toBe("JSON data file")
			expect(r.mimeType).toBe("application/json")
		})
	})

	describe("resourceTemplate", () => {
		test("creates resource template", () => {
			const t = resourceTemplate({
				uriTemplate: "file:///{path}",
				name: "File",
				handler: () => () => resourceText("test", "content"),
			})

			expect(t.uriTemplate).toBe("file:///{path}")
			expect(t.name).toBe("File")
			expect(t.type).toBe("template")
		})

		test("creates template with optional fields", () => {
			const t = resourceTemplate({
				uriTemplate: "db://{table}/{id}",
				name: "Database Record",
				description: "Fetch database record",
				mimeType: "application/json",
				handler: () => () => resourceText("test", "{}"),
			})

			expect(t.description).toBe("Fetch database record")
			expect(t.mimeType).toBe("application/json")
		})
	})

	describe("resourceText", () => {
		test("creates text resource content", () => {
			const result = resourceText("file:///test.txt", "Hello")

			expect(result.contents).toHaveLength(1)
			expect(result.contents[0]).toEqual({
				type: "resource",
				uri: "file:///test.txt",
				text: "Hello",
			})
		})

		test("creates text resource with mimeType", () => {
			const result = resourceText("file:///data.json", "{}", "application/json")

			expect(result.contents[0]).toEqual({
				type: "resource",
				uri: "file:///data.json",
				text: "{}",
				mimeType: "application/json",
			})
		})
	})

	describe("resourceBlob", () => {
		test("creates blob resource content", () => {
			const result = resourceBlob("file:///image.png", "base64data", "image/png")

			expect(result.contents).toHaveLength(1)
			expect(result.contents[0]).toEqual({
				type: "resource",
				uri: "file:///image.png",
				blob: "base64data",
				mimeType: "image/png",
			})
		})
	})

	describe("resourceContents", () => {
		test("wraps multiple contents", () => {
			const result = resourceContents(
				{ type: "resource", uri: "a", text: "A" },
				{ type: "resource", uri: "b", text: "B" },
			)

			expect(result.contents).toHaveLength(2)
		})

		test("handles single content", () => {
			const result = resourceContents({ type: "resource", uri: "a", text: "A" })

			expect(result.contents).toHaveLength(1)
		})
	})

	describe("matchesTemplate", () => {
		test("matches simple template", () => {
			expect(matchesTemplate("file:///{path}", "file:///test.txt")).toBe(true)
			expect(matchesTemplate("file:///{path}", "http:///test.txt")).toBe(false)
		})

		test("matches template with multiple params", () => {
			expect(matchesTemplate("db://{table}/{id}", "db://users/123")).toBe(true)
			expect(matchesTemplate("db://{table}/{id}", "db://users")).toBe(false)
		})

		test("matches exact string without params", () => {
			expect(matchesTemplate("file:///fixed.txt", "file:///fixed.txt")).toBe(true)
			expect(matchesTemplate("file:///fixed.txt", "file:///other.txt")).toBe(false)
		})

		test("handles special regex characters in template", () => {
			expect(matchesTemplate("file:///path.{ext}", "file:///path.txt")).toBe(true)
		})

		test("matches params with slashes", () => {
			// Single param should NOT match slashes (it's not a wildcard)
			expect(matchesTemplate("file:///{name}", "file:///a/b")).toBe(false)
		})
	})

	describe("extractParams", () => {
		test("extracts single parameter", () => {
			const params = extractParams("file:///{path}", "file:///test.txt")

			expect(params).toEqual({ path: "test.txt" })
		})

		test("extracts multiple parameters", () => {
			const params = extractParams("db://{table}/{id}", "db://users/123")

			expect(params).toEqual({ table: "users", id: "123" })
		})

		test("returns null for non-matching URI", () => {
			const params = extractParams("file:///{path}", "http:///test.txt")

			expect(params).toBeNull()
		})

		test("handles template without parameters", () => {
			const params = extractParams("file:///fixed.txt", "file:///fixed.txt")

			expect(params).toEqual({})
		})

		test("extracts params with special characters", () => {
			const params = extractParams("file:///{name}", "file:///my-file_v2.txt")

			expect(params).toEqual({ name: "my-file_v2.txt" })
		})
	})

	describe("toProtocolResource", () => {
		test("converts to protocol resource", () => {
			const r = resource({
				uri: "file:///test.txt",
				name: "Test",
				description: "Desc",
				mimeType: "text/plain",
				handler: () => () => resourceText("test", ""),
			})

			const proto = toProtocolResource(r)

			expect(proto).toEqual({
				uri: "file:///test.txt",
				name: "Test",
				description: "Desc",
				mimeType: "text/plain",
			})
		})
	})

	describe("toProtocolTemplate", () => {
		test("converts to protocol template", () => {
			const t = resourceTemplate({
				uriTemplate: "file:///{path}",
				name: "File",
				description: "Read file",
				mimeType: "text/plain",
				handler: () => () => resourceText("test", ""),
			})

			const proto = toProtocolTemplate(t)

			expect(proto).toEqual({
				uriTemplate: "file:///{path}",
				name: "File",
				description: "Read file",
				mimeType: "text/plain",
			})
		})
	})
})
