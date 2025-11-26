# @sylphx/mcp-server-sdk

Pure functional MCP (Model Context Protocol) server library for Bun.

## Features

- **Pure Functional**: Immutable data, composable handlers
- **Type-Safe**: First-class TypeScript with Zod schema integration
- **Fast**: Built for Bun with zero dependencies (except Zod optional)
- **Flexible**: Middleware system for logging, caching, retry, timeout
- **Complete**: Tools, resources, prompts, notifications

## Installation

```bash
bun add @sylphx/mcp-server-sdk
```

## Quick Start

```typescript
import { createServer, tool, text, stdio } from "@sylphx/mcp-server-sdk"

const greet = tool({
  name: "greet",
  description: "Greet someone",
  input: {
    type: "object",
    properties: { name: { type: "string" } },
    required: ["name"],
  },
  handler: ({ name }) => () => text(`Hello, ${name}!`),
})

const server = createServer({
  name: "my-server",
  version: "1.0.0",
  tools: [greet],
})

const transport = stdio(server)
await transport.start()
```

## Core Concepts

### Handler Pattern

All handlers follow the Reader pattern: `(input) => (context) => result`

This enables:
- Pure functions (input → output)
- Dependency injection via context
- Easy testing and composition

### Tools

Tools are callable functions exposed to the AI.

```typescript
import { tool, text, contents } from "@sylphx/mcp-server-sdk"

// Simple tool
const echo = tool({
  name: "echo",
  description: "Echo a message",
  input: {
    type: "object",
    properties: { message: { type: "string" } },
    required: ["message"],
  },
  handler: ({ message }) => () => text(message),
})

// Tool with multiple content
const info = tool({
  name: "info",
  description: "Get system info",
  input: { type: "object", properties: {} },
  handler: () => () => contents(
    text("CPU: 8 cores"),
    text("Memory: 16GB"),
  ),
})
```

### Type-Safe Tools with Zod

```typescript
import { z } from "zod/v4"
import { defineTool, text } from "@sylphx/mcp-server-sdk"

const calculator = defineTool({
  name: "calculate",
  description: "Perform arithmetic",
  input: z.object({
    a: z.number().describe("First number"),
    b: z.number().describe("Second number"),
    op: z.enum(["+", "-", "*", "/"]).describe("Operation"),
  }),
  handler: ({ a, b, op }) => () => {
    const result = op === "+" ? a + b
      : op === "-" ? a - b
      : op === "*" ? a * b
      : a / b
    return text(`${a} ${op} ${b} = ${result}`)
  },
})
```

### Resources

Resources provide data to the AI.

```typescript
import { resource, resourceTemplate, resourceText } from "@sylphx/mcp-server-sdk"

// Static resource
const readme = resource({
  uri: "file:///readme.md",
  name: "README",
  description: "Project readme",
  handler: (uri) => async () => resourceText(uri, "# My Project\n..."),
})

// Template resource (dynamic URI)
const fileResource = resourceTemplate({
  uriTemplate: "file:///{path}",
  name: "File",
  description: "Read any file",
  handler: (uri) => async (ctx) => {
    const path = uri.replace("file:///", "/")
    const content = await Bun.file(path).text()
    return resourceText(uri, content)
  },
})
```

### Prompts

Prompts are reusable conversation templates.

```typescript
import { prompt, user, assistant, messages } from "@sylphx/mcp-server-sdk"

const codeReview = prompt({
  name: "code-review",
  description: "Review code for issues",
  arguments: [
    { name: "code", description: "Code to review", required: true },
    { name: "language", description: "Programming language" },
  ],
  handler: (args) => () => messages(
    user(`Review this ${args.language ?? "code"}:\n\`\`\`\n${args.code}\n\`\`\``),
    assistant("I'll analyze this code for potential issues..."),
  ),
})
```

### Type-Safe Prompts with Zod

```typescript
import { z } from "zod/v4"
import { definePrompt, user, messages } from "@sylphx/mcp-server-sdk"

const translate = definePrompt({
  name: "translate",
  description: "Translate text",
  arguments: z.object({
    text: z.string().describe("Text to translate"),
    from: z.string().default("auto").describe("Source language"),
    to: z.string().describe("Target language"),
  }),
  handler: ({ text, from, to }) => () => messages(
    user(`Translate from ${from} to ${to}: "${text}"`),
  ),
})
```

## Middleware

Middleware wraps handlers for cross-cutting concerns.

```typescript
import {
  createServer,
  compose,
  logging,
  timeout,
  retry,
  cache,
  forType,
  forName,
} from "@sylphx/mcp-server-sdk"

const server = createServer({
  name: "my-server",
  version: "1.0.0",
  tools: [/* ... */],
  middleware: [
    // Log all requests
    logging({ log: console.log }),

    // Timeout after 30s
    timeout({ ms: 30000 }),

    // Retry failed tool calls
    forType("tool", retry({ maxAttempts: 3 })),

    // Cache specific resources
    forName("config*", cache({
      key: (info) => info.name,
      ttl: 60000,
    })),
  ],
})
```

### Built-in Middleware

| Middleware | Description |
|------------|-------------|
| `logging` | Log requests and responses |
| `timing` | Add timing info to context |
| `timeout` | Fail if handler takes too long |
| `retry` | Retry failed handlers |
| `cache` | Cache handler results |
| `errorHandler` | Catch and transform errors |
| `toolErrorHandler` | Convert errors to tool error results |

### Conditional Middleware

```typescript
import { when, forType, forName } from "@sylphx/mcp-server-sdk"

// Apply only when condition is true
when((info) => info.type === "tool", loggingMiddleware)

// Apply only for specific type
forType("tool", retryMiddleware)
forType("resource", cacheMiddleware)

// Apply only for specific names (supports glob patterns)
forName("read_*", authMiddleware)
forName(/^dangerous_/, confirmMiddleware)
```

### Custom Middleware

```typescript
import type { Middleware } from "@sylphx/mcp-server-sdk"

const authMiddleware: Middleware<MyContext, unknown> = async (ctx, info, next) => {
  if (!ctx.toolContext.user) {
    throw new Error("Unauthorized")
  }
  return next()
}
```

## Notifications

Send server-to-client notifications for progress, logging, and updates.

```typescript
import {
  createLogger,
  createProgressReporter,
  withProgress,
} from "@sylphx/mcp-server-sdk"

const processFiles = tool({
  name: "process-files",
  description: "Process multiple files",
  input: z.object({ files: z.array(z.string()) }),
  handler: ({ files }) => async (ctx) => {
    const logger = createLogger(ctx.notify, "process-files")
    logger.info("Starting processing")

    // Option 1: Manual progress reporting
    const report = createProgressReporter(ctx.notify, "process", files.length)
    for (let i = 0; i < files.length; i++) {
      report(i + 1, `Processing ${files[i]}`)
      // ... process file
    }

    // Option 2: Automatic progress tracking
    const results = await withProgress(
      ctx.notify,
      "process",
      files,
      async (file, report) => {
        report(`Processing ${file}`)
        return processFile(file)
      },
    )

    logger.info("Complete")
    return text(`Processed ${results.length} files`)
  },
})
```

### Notification Types

```typescript
// Progress notification
notify.emit({
  type: "progress",
  progressToken: "token",
  progress: 50,
  total: 100,
  message: "Halfway done",
})

// Log notification
notify.emit({
  type: "log",
  level: "info", // debug | info | notice | warning | error | critical | alert | emergency
  logger: "my-tool",
  data: { message: "Something happened" },
})

// List changed notifications
notify.emit({ type: "resources/list_changed" })
notify.emit({ type: "tools/list_changed" })
notify.emit({ type: "prompts/list_changed" })
```

## Transports

### Stdio Transport

For CLI tools and subprocess communication.

```typescript
import { stdio } from "@sylphx/mcp-server-sdk"

const transport = stdio(server)
await transport.start()

// Send notifications
transport.notify.emit({ type: "log", level: "info", data: "Started" })
```

### HTTP Transport

For web services with SSE support.

```typescript
import { http } from "@sylphx/mcp-server-sdk"

const transport = http(server, {
  port: 3000,
  cors: "*",
})

await transport.start()
console.log(`Server: ${transport.url}`)

// Broadcast to all sessions
transport.broadcast.emit({ type: "tools/list_changed" })

// Send to specific session
const sessionNotify = transport.getSessionNotifier(sessionId)
sessionNotify?.emit({ type: "progress", progressToken: "t", progress: 50 })
```

**Endpoints:**
- `POST /mcp` - JSON-RPC request/response
- `GET /mcp/sse` - SSE stream (returns session ID)
- `POST /mcp/sse` - Send message to SSE stream (requires `X-Session-ID` header)
- `GET /mcp/health` - Health check

## Custom Context

Inject dependencies into handlers via context.

```typescript
interface MyToolContext extends ToolContext {
  db: Database
  user: User
}

const server = createServer<MyToolContext>({
  name: "my-server",
  version: "1.0.0",
  tools: [/* ... */],
})

const transport = stdio(server, {
  createContext: (notify) => ({
    toolContext: {
      signal: undefined,
      notify,
      db: database,
      user: currentUser,
    },
    resourceContext: { signal: undefined, notify },
    promptContext: { signal: undefined, notify },
  }),
})
```

## API Reference

### Server

```typescript
createServer<TToolCtx, TResourceCtx, TPromptCtx>(config: ServerConfig): Server
createContext(overrides?): HandlerContext
```

### Builders

```typescript
// Tools
tool(config): ToolDefinition
defineTool(config): TypedToolDefinition  // with Zod
text(content): ToolsCallResult
contents(...items): ToolsCallResult
toolError(message): ToolsCallResult
structured(data, content?): ToolsCallResult

// Resources
resource(config): ResourceDefinition
resourceTemplate(config): ResourceTemplateDefinition
resourceText(uri, text): ResourcesReadResult
resourceBlob(uri, data, mimeType): ResourcesReadResult
resourceContents(...items): ResourcesReadResult

// Prompts
prompt(config): PromptDefinition
definePrompt(config): TypedPromptDefinition  // with Zod
user(content): PromptMessage
assistant(content): PromptMessage
messages(...msgs): PromptsGetResult
```

### Middleware

```typescript
compose(...middlewares): Middleware
createStack(): MiddlewareStack
when(predicate, middleware): Middleware
forType(type, middleware): Middleware
forName(pattern, middleware): Middleware

logging(options?): Middleware
timing(): Middleware
timeout(options): Middleware
retry(options): Middleware
cache(options): Middleware
errorHandler(options?): Middleware
toolErrorHandler(options?): Middleware
```

### Notifications

```typescript
createEmitter(sender): NotificationEmitter
noopEmitter: NotificationEmitter
progress(token, current, options?): Notification
log(level, data, logger?): Notification
createProgressReporter(emitter, token, total?): (current, message?) => void
createLogger(emitter, namespace?): Logger
withProgress(emitter, token, items, processor): Promise<results>
resourcesListChanged(): Notification
toolsListChanged(): Notification
promptsListChanged(): Notification
```

### Schema

```typescript
zodToJsonSchema(schema): JsonSchema
validate(schema, input): ValidationResult
isZodSchema(value): boolean
```

## License

MIT
