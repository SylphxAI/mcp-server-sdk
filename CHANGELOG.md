# Changelog

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
