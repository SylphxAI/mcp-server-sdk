# Local instructions

Validate changes with:

```sh
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Keep protocol/domain crates independent of transport adapters. Add behavior
tests beside the owning crate; do not add source-shape or migration gates.
