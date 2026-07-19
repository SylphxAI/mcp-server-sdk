# Security

Report vulnerabilities privately through GitHub Security Advisories.

Security-sensitive changes must pass:

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Transport and authentication changes require behavior tests for rejection,
session isolation, and protocol error handling.
