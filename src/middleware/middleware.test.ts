import { describe, expect, test } from "bun:test"
import {
	cache,
	compose,
	createStack,
	forName,
	forType,
	logging,
	retry,
	timeout,
	when,
} from "./index.js"
import type { Middleware, RequestInfo } from "./types.js"

describe("Middleware", () => {
	const makeInfo = (type: "tool" | "resource" | "prompt" = "tool", name = "test"): RequestInfo => ({
		type,
		name,
		input: {},
		startTime: Date.now(),
	})

	describe("compose", () => {
		test("composes multiple middleware", async () => {
			const order: number[] = []

			const mw1: Middleware<unknown, string> = async (_ctx, _info, next) => {
				order.push(1)
				const result = await next()
				order.push(4)
				return result
			}

			const mw2: Middleware<unknown, string> = async (_ctx, _info, next) => {
				order.push(2)
				const result = await next()
				order.push(3)
				return result
			}

			const composed = compose(mw1, mw2)
			const result = await composed({}, makeInfo(), async () => "done")

			expect(result).toBe("done")
			expect(order).toEqual([1, 2, 3, 4])
		})

		test("empty compose returns identity", async () => {
			const composed = compose<unknown, string>()
			const result = await composed({}, makeInfo(), async () => "done")
			expect(result).toBe("done")
		})

		test("single middleware works", async () => {
			const mw: Middleware<unknown, string> = async (_ctx, _info, next) => {
				const result = await next()
				return `wrapped: ${result}`
			}

			const composed = compose(mw)
			const result = await composed({}, makeInfo(), async () => "done")
			expect(result).toBe("wrapped: done")
		})

		test("prevents multiple next() calls", async () => {
			const mw: Middleware<unknown, string> = async (_ctx, _info, next) => {
				await next()
				return next() // Second call should fail
			}

			const composed = compose(mw)
			await expect(composed({}, makeInfo(), async () => "done")).rejects.toThrow(
				"next() called multiple times"
			)
		})
	})

	describe("createStack", () => {
		test("creates empty stack", () => {
			const stack = createStack<unknown, string>()
			expect(stack.middlewares).toHaveLength(0)
		})

		test("use adds middleware immutably", () => {
			const mw1: Middleware<unknown, string> = async (_ctx, _info, next) => next()
			const mw2: Middleware<unknown, string> = async (_ctx, _info, next) => next()

			const stack1 = createStack<unknown, string>()
			const stack2 = stack1.use(mw1)
			const stack3 = stack2.use(mw2)

			expect(stack1.middlewares).toHaveLength(0)
			expect(stack2.middlewares).toHaveLength(1)
			expect(stack3.middlewares).toHaveLength(2)
		})

		test("execute runs middleware stack", async () => {
			const order: number[] = []

			const stack = createStack<unknown, string>()
				.use(async (_ctx, _info, next) => {
					order.push(1)
					return next()
				})
				.use(async (_ctx, _info, next) => {
					order.push(2)
					return next()
				})

			const result = await stack.execute({}, makeInfo(), async () => {
				order.push(3)
				return "done"
			})

			expect(result).toBe("done")
			expect(order).toEqual([1, 2, 3])
		})
	})

	describe("when", () => {
		test("applies middleware when predicate is true", async () => {
			let applied = false
			const mw: Middleware<unknown, string> = async (_ctx, _info, next) => {
				applied = true
				return next()
			}

			const conditional = when(() => true, mw)
			await conditional({}, makeInfo(), async () => "done")

			expect(applied).toBe(true)
		})

		test("skips middleware when predicate is false", async () => {
			let applied = false
			const mw: Middleware<unknown, string> = async (_ctx, _info, next) => {
				applied = true
				return next()
			}

			const conditional = when(() => false, mw)
			await conditional({}, makeInfo(), async () => "done")

			expect(applied).toBe(false)
		})
	})

	describe("forType", () => {
		test("applies for matching type", async () => {
			let applied = false
			const mw: Middleware<unknown, string> = async (_ctx, _info, next) => {
				applied = true
				return next()
			}

			const conditional = forType("tool", mw)
			await conditional({}, makeInfo("tool"), async () => "done")

			expect(applied).toBe(true)
		})

		test("skips for non-matching type", async () => {
			let applied = false
			const mw: Middleware<unknown, string> = async (_ctx, _info, next) => {
				applied = true
				return next()
			}

			const conditional = forType("tool", mw)
			await conditional({}, makeInfo("resource"), async () => "done")

			expect(applied).toBe(false)
		})
	})

	describe("forName", () => {
		test("applies for matching name", async () => {
			let applied = false
			const mw: Middleware<unknown, string> = async (_ctx, _info, next) => {
				applied = true
				return next()
			}

			const conditional = forName("test", mw)
			await conditional({}, makeInfo("tool", "test"), async () => "done")

			expect(applied).toBe(true)
		})

		test("supports glob patterns", async () => {
			let applied = false
			const mw: Middleware<unknown, string> = async (_ctx, _info, next) => {
				applied = true
				return next()
			}

			const conditional = forName("read_*", mw)
			await conditional({}, makeInfo("tool", "read_file"), async () => "done")

			expect(applied).toBe(true)
		})

		test("supports regex", async () => {
			let applied = false
			const mw: Middleware<unknown, string> = async (_ctx, _info, next) => {
				applied = true
				return next()
			}

			const conditional = forName(/^read_/, mw)
			await conditional({}, makeInfo("tool", "read_file"), async () => "done")

			expect(applied).toBe(true)
		})
	})

	describe("logging", () => {
		test("logs request and response", async () => {
			const logs: string[] = []
			const mw = logging<unknown, string>({
				log: (msg) => logs.push(msg),
			})

			await mw({}, makeInfo("tool", "greet"), async () => "done")

			expect(logs).toHaveLength(2)
			expect(logs[0]).toContain("greet")
			expect(logs[0]).toContain("started")
			expect(logs[1]).toContain("completed")
		})

		test("logs error on failure", async () => {
			const logs: string[] = []
			const mw = logging<unknown, string>({
				log: (msg) => logs.push(msg),
			})

			await expect(
				mw({}, makeInfo(), async () => {
					throw new Error("oops")
				})
			).rejects.toThrow("oops")

			expect(logs[1]).toContain("failed")
		})
	})

	describe("timeout", () => {
		test("allows fast handlers", async () => {
			const mw = timeout<unknown, string>({ ms: 100 })
			const result = await mw({}, makeInfo(), async () => "done")
			expect(result).toBe("done")
		})

		test("fails slow handlers", async () => {
			const mw = timeout<unknown, string>({ ms: 10 })
			await expect(
				mw({}, makeInfo(), async () => {
					await new Promise((r) => setTimeout(r, 50))
					return "done"
				})
			).rejects.toThrow("timed out")
		})
	})

	describe("retry", () => {
		test("succeeds without retry if handler succeeds", async () => {
			let attempts = 0
			const mw = retry<unknown, string>({ maxAttempts: 3 })

			const result = await mw({}, makeInfo(), async () => {
				attempts++
				return "done"
			})

			expect(result).toBe("done")
			expect(attempts).toBe(1)
		})

		test("retries on failure", async () => {
			let attempts = 0
			const mw = retry<unknown, string>({ maxAttempts: 3 })

			const result = await mw({}, makeInfo(), async () => {
				attempts++
				if (attempts < 3) throw new Error("fail")
				return "done"
			})

			expect(result).toBe("done")
			expect(attempts).toBe(3)
		})

		test("gives up after max attempts", async () => {
			let attempts = 0
			const mw = retry<unknown, string>({ maxAttempts: 3 })

			await expect(
				mw({}, makeInfo(), async () => {
					attempts++
					throw new Error("fail")
				})
			).rejects.toThrow("fail")

			expect(attempts).toBe(3)
		})

		test("respects shouldRetry", async () => {
			let attempts = 0
			const mw = retry<unknown, string>({
				maxAttempts: 3,
				shouldRetry: (e) => (e as Error).message !== "fatal",
			})

			await expect(
				mw({}, makeInfo(), async () => {
					attempts++
					throw new Error("fatal")
				})
			).rejects.toThrow("fatal")

			expect(attempts).toBe(1)
		})
	})

	describe("cache", () => {
		test("caches results", async () => {
			let calls = 0
			const mw = cache<unknown, string>({
				key: (info) => info.name,
				ttl: 1000,
			})

			const handler = async () => {
				calls++
				return "result"
			}

			const result1 = await mw({}, makeInfo("tool", "test"), handler)
			const result2 = await mw({}, makeInfo("tool", "test"), handler)

			expect(result1).toBe("result")
			expect(result2).toBe("result")
			expect(calls).toBe(1)
		})

		test("expires cache", async () => {
			let calls = 0
			const mw = cache<unknown, string>({
				key: (info) => info.name,
				ttl: 10,
			})

			const handler = async () => {
				calls++
				return "result"
			}

			await mw({}, makeInfo("tool", "test"), handler)
			await new Promise((r) => setTimeout(r, 20))
			await mw({}, makeInfo("tool", "test"), handler)

			expect(calls).toBe(2)
		})
	})
})
