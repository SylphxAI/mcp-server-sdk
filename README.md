# @sylphx/mcp-server-sdk

Pure functional MCP (Model Context Protocol) server library for Bun.

## Features

- **Pure Functional**: Immutable data, composable handlers
- **Type-Safe**: First-class TypeScript with Zod schema integration
- **Builder Pattern**: Fluent API for defining tools, resources, and prompts
- **Fast**: Built for Bun with minimal dependencies
- **Complete**: Tools, resources, prompts, notifications, sampling, elicitation

## Installation

```bash
bun add @sylphx/mcp-server-sdk zod
```

## Quick Start

```typescript
import { createServer, tool, text, stdio } from "@sylphx/mcp-server-sdk"
import { z } from "zod"

// Define tools using builder pattern
const greet = tool()
  .description("Greet someone")
  .input(z.object({ name: z.string() }))
  .handler(({ input }) => text(`Hello, ${input.name}!`))

const ping = tool()
  .handler(() => text("pong"))

// Create and start server
const server = createServer({
  name: "my-server",
  version: "1.0.0",
  tools: { greet, ping },  // Names from object keys
  transport: stdio()
})

await server.start()
```

## Tools

Tools are callable functions exposed to the AI.

```typescript
import { tool, text, contents, json, toolError } from "@sylphx/mcp-server-sdk"
import { z } from "zod"

// Simple tool (no input)
const ping = tool()
  .description("Health check")
  .handler(() => text("pong"))

// Tool with typed input
const calculator = tool()
  .description("Perform arithmetic")
  .input(z.object({
    a: z.number().describe("First number"),
    b: z.number().describe("Second number"),
    op: z.enum(["+", "-", "*", "/"]).describe("Operation"),
  }))
  .handler(({ input }) => {
    const { a, b, op } = input
    const result = op === "+" ? a + b
      : op === "-" ? a - b
      : op === "*" ? a * b
      : a / b
    return text(`${a} ${op} ${b} = ${result}`)
  })

// Tool with multiple content items
const systemInfo = tool()
  .description("Get system information")
  .handler(() => contents(
    { type: "text", text: "CPU: 8 cores" },
    { type: "text", text: "Memory: 16GB" }
  ))

// Return JSON data
const getUser = tool()
  .description("Get user data")
  .input(z.object({ id: z.string() }))
  .handler(({ input }) => json({ id: input.id, name: "Alice" }))

// Return error
const riskyOperation = tool()
  .description("May fail")
  .handler(() => toolError("Something went wrong"))
```

## Resources

Resources provide data to the AI.

```typescript
import { resource, resourceTemplate, resourceText, resourceBlob } from "@sylphx/mcp-server-sdk"

// Static resource with fixed URI
const readme = resource()
  .uri("file:///readme.md")
  .description("Project readme")
  .mimeType("text/markdown")
  .handler(({ uri }) => resourceText(uri, "# My Project\n\nWelcome!"))

// Resource template for dynamic URIs
const fileReader = resourceTemplate()
  .uriTemplate("file:///{path}")
  .description("Read any file")
  .handler(async ({ uri, params }) => {
    const content = await Bun.file(`/${params.path}`).text()
    return resourceText(uri, content)
  })

// Binary resource
const logo = resource()
  .uri("image:///logo.png")
  .mimeType("image/png")
  .handler(async ({ uri }) => {
    const data = await Bun.file("./logo.png").bytes()
    const base64 = Buffer.from(data).toString("base64")
    return resourceBlob(uri, base64, "image/png")
  })
```

## Prompts

Prompts are reusable conversation templates.

```typescript
import { prompt, user, assistant, messages, promptResult } from "@sylphx/mcp-server-sdk"
import { z } from "zod"

// Simple prompt (no arguments)
const greeting = prompt()
  .description("A friendly greeting")
  .handler(() => messages(
    user("Hello!"),
    assistant("Hi there! How can I help you today?")
  ))

// Prompt with typed arguments
const codeReview = prompt()
  .description("Review code for issues")
  .args(z.object({
    code: z.string().describe("Code to review"),
    language: z.string().optional().describe("Programming language"),
  }))
  .handler(({ args }) => messages(
    user(`Please review this ${args.language ?? "code"}:\n\`\`\`\n${args.code}\n\`\`\``),
    assistant("I'll analyze this code for potential issues, best practices, and improvements.")
  ))

// Prompt with description in result
const translate = prompt()
  .description("Translate text between languages")
  .args(z.object({
    text: z.string(),
    from: z.string().default("auto"),
    to: z.string(),
  }))
  .handler(({ args }) => promptResult(
    `Translation from ${args.from} to ${args.to}`,
    messages(user(`Translate "${args.text}" from ${args.from} to ${args.to}`))
  ))
```

## Server Configuration

```typescript
import { createServer, stdio, http } from "@sylphx/mcp-server-sdk"

const server = createServer({
  // Server identity
  name: "my-server",
  version: "1.0.0",
  instructions: "This server provides...",

  // Handlers (names from object keys)
  tools: { greet, ping, calculator },
  resources: { readme, config },
  resourceTemplates: { file: fileReader },
  prompts: { codeReview, translate },

  // Transport
  transport: stdio()  // or http({ port: 3000 })
})

await server.start()
```

## Transports

### Stdio Transport

For CLI tools and subprocess communication.

```typescript
import { stdio } from "@sylphx/mcp-server-sdk"

const server = createServer({
  tools: { ping },
  transport: stdio()
})

await server.start()
```

### HTTP Transport

For web services with Server-Sent Events support.

```typescript
import { http } from "@sylphx/mcp-server-sdk"

const server = createServer({
  tools: { ping },
  transport: http({ port: 3000 })
})

await server.start()
// Server running at http://localhost:3000
```

**Endpoints:**
- `POST /mcp` - JSON-RPC request/response
- `GET /mcp/sse` - SSE stream for notifications
- `POST /mcp/sse` - Send message via SSE (requires session ID)
- `GET /mcp/health` - Health check

## Notifications

Send server-to-client notifications for progress and logging.

```typescript
import {
  createLogger,
  createProgressReporter,
  withProgress
} from "@sylphx/mcp-server-sdk"

const processFiles = tool()
  .description("Process multiple files")
  .input(z.object({ files: z.array(z.string()) }))
  .handler(async ({ input, ctx }) => {
    // Logging
    const logger = createLogger(ctx.notify, "process-files")
    logger.info("Starting processing")

    // Manual progress reporting
    const report = createProgressReporter(ctx.notify, "process", input.files.length)
    for (let i = 0; i < input.files.length; i++) {
      report(i + 1, `Processing ${input.files[i]}`)
      // ... process file
    }

    // Or automatic progress tracking
    const results = await withProgress(
      ctx.notify,
      "process",
      input.files,
      async (file, report) => {
        report(`Processing ${file}`)
        return processFile(file)
      }
    )

    logger.info("Complete")
    return text(`Processed ${results.length} files`)
  })
```

## Sampling

Request LLM completions from the client.

```typescript
import { createSamplingClient, samplingText } from "@sylphx/mcp-server-sdk"

const summarize = tool()
  .description("Summarize text using AI")
  .input(z.object({ text: z.string() }))
  .handler(async ({ input, ctx }) => {
    const sampling = createSamplingClient(ctx.requestSampling)

    const result = await sampling.createMessage({
      messages: [samplingText("user", `Summarize: ${input.text}`)],
      maxTokens: 500,
    })

    return text(result.content[0].text)
  })
```

## Elicitation

Request user input from the client.

```typescript
import { createElicitationClient, elicitString, elicitBoolean } from "@sylphx/mcp-server-sdk"

const confirmAction = tool()
  .description("Confirm before proceeding")
  .input(z.object({ action: z.string() }))
  .handler(async ({ input, ctx }) => {
    const elicit = createElicitationClient(ctx.requestElicitation)

    const result = await elicit.elicit({
      message: `Are you sure you want to ${input.action}?`,
      schema: {
        confirm: elicitBoolean("Confirm action"),
        reason: elicitString("Optional reason", false),
      }
    })

    if (result.action === "accept" && result.content?.confirm) {
      // Proceed with action
      return text(`Proceeding with ${input.action}`)
    }

    return text("Action cancelled")
  })
```

## API Reference

### Server

```typescript
createServer(config: ServerConfig): Server

interface ServerConfig {
  name?: string                    // Default: "mcp-server"
  version?: string                 // Default: "1.0.0"
  instructions?: string            // Instructions for the LLM
  tools?: Record<string, ToolDefinition>
  resources?: Record<string, ResourceDefinition>
  resourceTemplates?: Record<string, ResourceTemplateDefinition>
  prompts?: Record<string, PromptDefinition>
  transport: TransportFactory
}
```

### Tool Builder

```typescript
tool()
  .description(string)                    // Optional description
  .input(ZodSchema)                       // Optional input schema
  .handler(fn: HandlerFn) -> ToolDefinition

// Handler signature (with input)
({ input, ctx }) => ToolsCallResult | Promise<ToolsCallResult>

// Handler signature (without input)
({ ctx }) => ToolsCallResult | Promise<ToolsCallResult>

// Or simplified (no ctx needed)
() => ToolsCallResult
```

### Resource Builder

```typescript
resource()
  .uri(string)                            // Required URI
  .description(string)                    // Optional description
  .mimeType(string)                       // Optional MIME type
  .handler(fn) -> ResourceDefinition

resourceTemplate()
  .uriTemplate(string)                    // Required URI template (RFC 6570)
  .description(string)                    // Optional description
  .mimeType(string)                       // Optional MIME type
  .handler(fn) -> ResourceTemplateDefinition

// Handler receives { uri, ctx } or { uri, params, ctx }
```

### Prompt Builder

```typescript
prompt()
  .description(string)                    // Optional description
  .args(ZodSchema)                        // Optional arguments schema
  .handler(fn) -> PromptDefinition

// Handler receives { args, ctx } or { ctx }
```

### Content Helpers

```typescript
// Tools
text(content: string): ToolsCallResult
json(data: unknown): ToolsCallResult
contents(...items: Content[]): ToolsCallResult
toolError(message: string): ToolsCallResult

// Resources
resourceText(uri: string, text: string, mimeType?: string): ResourcesReadResult
resourceBlob(uri: string, blob: string, mimeType: string): ResourcesReadResult
resourceContents(...items: EmbeddedResource[]): ResourcesReadResult

// Prompts
user(content: string): PromptMessage
assistant(content: string): PromptMessage
messages(...msgs: PromptMessage[]): PromptsGetResult
promptResult(description: string, result: PromptsGetResult): PromptsGetResult
```

### Transports

```typescript
stdio(options?: StdioOptions): TransportFactory
http(options?: HttpOptions): TransportFactory

interface StdioOptions {
  // No options currently
}

interface HttpOptions {
  port?: number        // Default: 3000
  hostname?: string    // Default: "localhost"
}
```

### Notifications

```typescript
createEmitter(sender: NotificationSender): NotificationEmitter
noopEmitter: NotificationEmitter
progress(token, current, options?): ProgressNotification
log(level, data, logger?): LogNotification
createProgressReporter(emitter, token, total?): (current, message?) => void
createLogger(emitter, namespace?): Logger
withProgress(emitter, token, items, processor): Promise<T[]>
resourcesListChanged(): Notification
toolsListChanged(): Notification
promptsListChanged(): Notification
```

## License

MIT
