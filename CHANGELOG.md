# Changelog

## 3.0.0 (2025-12-05)

### ♻️ Refactoring

- 💥 **schema:** remove Zod compatibility layer ([91e14cd](https://github.com/SylphxAI/mcp-server-sdk/commit/91e14cd3e3aedfbae35d00cd26edc7f93ad4761d))

### 💥 Breaking Changes

- **schema:** remove Zod compatibility layer ([91e14cd](https://github.com/SylphxAI/mcp-server-sdk/commit/91e14cd3e3aedfbae35d00cd26edc7f93ad4761d))
  Zod schema support has been removed.

## 2.1.0 (2025-12-05)

### ✨ Features

- **schema:** add Zod compatibility layer ([8ffacea](https://github.com/SylphxAI/mcp-server-sdk/commit/8ffacea4cb076995b6cece44f0329d42fac98840))

## 2.0.0 (2025-12-05)

### ✨ Features

- **app:** add runStdio adapter for CLI tools ([81d270e](https://github.com/SylphxAI/mcp-server-sdk/commit/81d270e91793a125a201594c8354433cac60b9af))
- 💥 introduce createMcpApp + serve architecture ([0269d9d](https://github.com/SylphxAI/mcp-server-sdk/commit/0269d9dd4668407172ce69724f1599781a0a90e3))
- **deps:** upgrade vex to 0.1.11 with vex-json-schema ([3057e9a](https://github.com/SylphxAI/mcp-server-sdk/commit/3057e9a1be15e81df0efc3af94eb4ed510262a38))
- 💥 replace zod with @sylphx/vex for schema validation ([b06a2cc](https://github.com/SylphxAI/mcp-server-sdk/commit/b06a2cceff687d8ab2fb4d2e9c7a4be654b4a199))

### 🐛 Bug Fixes

- **deps:** upgrade gust to 0.1.13 for server bind fix ([76c831e](https://github.com/SylphxAI/mcp-server-sdk/commit/76c831e1304532d7cefb0909f1c8539c79d10bc1))

### 💥 Breaking Changes

- introduce createMcpApp + serve architecture ([0269d9d](https://github.com/SylphxAI/mcp-server-sdk/commit/0269d9dd4668407172ce69724f1599781a0a90e3))
  New recommended API separates app logic from server.
- replace zod with @sylphx/vex for schema validation ([b06a2cc](https://github.com/SylphxAI/mcp-server-sdk/commit/b06a2cceff687d8ab2fb4d2e9c7a4be654b4a199))
  zod is replaced with @sylphx/vex for schema validation

## 1.3.0 (2025-11-29)

### ✨ Features

- **conformance:** add SEP-1034 elicitation defaults test tool ([58da35f](https://github.com/SylphxAI/mcp-server-sdk/commit/58da35f3017b3456bbad26774db4cda2b7d3c5c6))
- **http:** implement true streaming SSE with bidirectional RPC ([9b99d2d](https://github.com/SylphxAI/mcp-server-sdk/commit/9b99d2d06b8d00401c58047b760a8dc8221f5071))
- **http:** add bidirectional RPC infrastructure ([f4d7922](https://github.com/SylphxAI/mcp-server-sdk/commit/f4d79221ba0269aa48ca7ea8d953a8da8aaf2a76))
- add sampling and elicitation capabilities ([107c750](https://github.com/SylphxAI/mcp-server-sdk/commit/107c750f6787ddecaa828639d7f2160dd726c1b4))

### ♻️ Refactoring

- **http:** migrate to gust 0.1.7 middleware API ([5137e95](https://github.com/SylphxAI/mcp-server-sdk/commit/5137e95ceb6a226accebd46f35f29ff812b1e9e7))
- **http:** simplify SSE and document streaming limitation ([24050dc](https://github.com/SylphxAI/mcp-server-sdk/commit/24050dc0045d12b78a6f8928818f8a3b62759324))

### 📚 Documentation

- **readme:** update for v1.2.0 features ([6d31fd6](https://github.com/SylphxAI/mcp-server-sdk/commit/6d31fd6390b44f33ceeaaed67dbe0ff7cb527e6c))

## 1.2.0 (2025-11-27)

### ✨ Features

- **http:** implement MCP Streamable HTTP transport with SSE notifications ([8d75bbc](https://github.com/SylphxAI/mcp-server-sdk/commit/8d75bbcc728415acbd8b9f3d4ca60aaec9f743e2))

## 1.1.2 (2025-11-27)

### 🐛 Bug Fixes

- use correct MCP protocol field names for list responses ([b4f072d](https://github.com/SylphxAI/mcp-server-sdk/commit/b4f072d340ed09155f9a3db3f3ea934dfb711db5))

## 1.1.1 (2025-11-27)

### 🐛 Bug Fixes

- replace Bun.stdin/stdout with Node.js streams ([17425f6](https://github.com/SylphxAI/mcp-server-sdk/commit/17425f6ec63a538f45e76b05a5d0bf551f16b396))

## 1.2.0 (2025-11-27)

### 🐛 Bug Fixes

- replace Bun.stdin/stdout with Node.js streams for full Node.js compatibility

### 📚 Documentation

- update package description to reflect Node.js support

## 1.1.0 (2025-11-27)

### ✨ Features

- replace Bun.serve with Hono for Node.js compatibility ([8307e17](https://github.com/SylphxAI/mcp-server-sdk/commit/8307e170702a3d2e0b2f73b4cc9a0ff5a86fe32b))

### 🔧 Chores

- upgrade biome to v2 and update dependencies ([2beb11e](https://github.com/SylphxAI/mcp-server-sdk/commit/2beb11e55fa5d0358951fa3859a28f440fba90b6))

## 1.0.0 (2025-11-27)

### 🐛 Bug Fixes

- update isZodSchema to detect Zod 4 via ~standard interface ([8d986f3](https://github.com/SylphxAI/mcp-server-sdk/commit/8d986f31f7a17fa30eb26957b9bd7f72bd5ed213))
- add prepack script to build before publish ([dd70335](https://github.com/SylphxAI/mcp-server-sdk/commit/dd7033527d479e7ece6be3996b0607eb54b366d1))
- resolve type error in resource.ts param indexing ([e86b73e](https://github.com/SylphxAI/mcp-server-sdk/commit/e86b73ebd02c8b300e73ca360be4efa63dee5260))

### ♻️ Refactoring

- 💥 simplify SDK by removing non-essential helpers ([98b1b3d](https://github.com/SylphxAI/mcp-server-sdk/commit/98b1b3da8b89d3009687b42e992dec8675ca337c))
- 💥 simplify schema - remove isZodSchema, toJsonSchema, SchemaInput ([bad36a0](https://github.com/SylphxAI/mcp-server-sdk/commit/bad36a07aa459bc7d7f5629264c4c4120ba85782))

### 📚 Documentation

- update README to reflect simplified API ([ff55526](https://github.com/SylphxAI/mcp-server-sdk/commit/ff55526347cadc3c9c24762c8ece0925a535714a))

### 💥 Breaking Changes

- simplify SDK by removing non-essential helpers ([98b1b3d](https://github.com/SylphxAI/mcp-server-sdk/commit/98b1b3da8b89d3009687b42e992dec8675ca337c))
  Removed helper wrappers that users can write directly
- simplify schema - remove isZodSchema, toJsonSchema, SchemaInput ([bad36a0](https://github.com/SylphxAI/mcp-server-sdk/commit/bad36a07aa459bc7d7f5629264c4c4120ba85782))
  removed isZodSchema, toJsonSchema, SchemaInput exports

## 0.2.1 (2025-11-27)

### 🐛 Bug Fixes

- correct repository URL in package.json ([1bfa991](https://github.com/SylphxAI/mcp-server-sdk/commit/1bfa991b98d91a082a9162562802d94479cdb187))

### 🔧 Chores

- update @sylphx/bump to v0.7.0 and @sylphx/doctor to v1.20.0 ([8da449b](https://github.com/SylphxAI/mcp-server-sdk/commit/8da449bb0f58a70e14f6ce7ff6a2d18d50d1ea50))
- update @sylphx/doctor to v1.19.2 ([4e60959](https://github.com/SylphxAI/mcp-server-sdk/commit/4e609590fce6c099183ebefbd46159b8e3cc61ff))
- update @sylphx/bump and @sylphx/doctor ([7b68ebc](https://github.com/SylphxAI/mcp-server-sdk/commit/7b68ebcab4bf8e36c6bdb033b72161954b9c0a29))
- add sylphx devDependencies and credits section ([95c4321](https://github.com/SylphxAI/mcp-server-sdk/commit/95c432174820e9361ee724f8a46802dec87d6018))
- add pre-push hook and update doctor package name ([9db9058](https://github.com/SylphxAI/mcp-server-sdk/commit/9db9058eab57bc759d45faae37a015feca9a4d32))
- use doctor prepublish and add typecheck script ([acabf44](https://github.com/SylphxAI/mcp-server-sdk/commit/acabf445e1e1635abb836ee104e1962be0fd0f4f))

## 0.2.0 (2025-11-26)

### ✨ Features

- **tool:** add complete content type support ([c878811](https://github.com/SylphxAI/mcp-server-sdk/commit/c8788115b070bd8422bf04d3f762efe59d030cf0))
- **tool:** add image and imageContent helpers ([6311520](https://github.com/SylphxAI/mcp-server-sdk/commit/6311520a6da79a2dd57af0685533cc00994e0cae))
- add elicitation support and update protocol types ([758d674](https://github.com/SylphxAI/mcp-server-sdk/commit/758d674a909792820016f0370a0abbe09bfd1c78))
- complete MCP protocol implementation ([158568f](https://github.com/SylphxAI/mcp-server-sdk/commit/158568f7607fda3d3befc0fff31ace2093cfd574))
- add notification system ([1532223](https://github.com/SylphxAI/mcp-server-sdk/commit/1532223592e1983f15a59e4eae9d49511eadff92))
- add middleware system ([42266de](https://github.com/SylphxAI/mcp-server-sdk/commit/42266de5dcd8fc5d8849fe8b4f44a105f1392e5c))
- add Zod schema integration ([88f4290](https://github.com/SylphxAI/mcp-server-sdk/commit/88f4290ae341872782fb194caf895c3cf74c0747))
- initial implementation of @sylphx/mcp-server ([7974dd2](https://github.com/SylphxAI/mcp-server-sdk/commit/7974dd246fbf5afcb5984f0e0cd9fa3b241a8a35))

### 🐛 Bug Fixes

- complete MCP protocol verification ([0744bdf](https://github.com/SylphxAI/mcp-server-sdk/commit/0744bdfb7a2dcf3e6d575e25f8e5c86d2cae826a))

### ♻️ Refactoring

- **tool:** simplify content API - return Content directly ([40de7cd](https://github.com/SylphxAI/mcp-server-sdk/commit/40de7cdba9ad70d1586481ce9f57ac826fec8b4b))
- **api:** simplify to builder pattern with pure function transports ([e00cd1e](https://github.com/SylphxAI/mcp-server-sdk/commit/e00cd1efe6d408d6974b8898a9acbd01b395f241))

### 📚 Documentation

- add comprehensive README ([a222141](https://github.com/SylphxAI/mcp-server-sdk/commit/a22214184b39a4725f9f547158e524b84d6b83eb))

### ✅ Tests

- add comprehensive tests for builders and handlers ([652fae0](https://github.com/SylphxAI/mcp-server-sdk/commit/652fae0657b80122581e3a91baf57c3f289e04e8))

### 📦 Build

- switch to bunup for proper .d.ts generation ([f2d9027](https://github.com/SylphxAI/mcp-server-sdk/commit/f2d90277ae1a4ba989ec0912bfa4dcd21916763a))

### 🔧 Chores

- add progress.md ([b21d91d](https://github.com/SylphxAI/mcp-server-sdk/commit/b21d91ddb59d674a46686a102008da3d97a33a3b))
- remove outdated progress.md ([d16720c](https://github.com/SylphxAI/mcp-server-sdk/commit/d16720c040211eacf475d8b504900e586f4edf14))
- update benchmarks for new API and minor optimizations ([d9593c1](https://github.com/SylphxAI/mcp-server-sdk/commit/d9593c1f6617ae88e7db39da1146840e0ace3656))
- cleanup technical debt and remove unused code ([5df9d72](https://github.com/SylphxAI/mcp-server-sdk/commit/5df9d72b3d323d221c1250d99b78c20ccdece044))
- rename package to @sylphx/mcp-server-sdk ([544f927](https://github.com/SylphxAI/mcp-server-sdk/commit/544f927af57d7abbd1332b44241f17e06700848f))
- apply sylphx shared config and formatting ([5848004](https://github.com/SylphxAI/mcp-server-sdk/commit/584800402144e63fc608ff24cc036122161fcd28))
