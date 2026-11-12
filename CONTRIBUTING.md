# Contributing

Bayleaf is in its design phase. Issues with ideas, hardware reports (especially low-end
tablets and 2-in-1s) and design feedback are the most useful contributions right now.

## Building
Linux or WSL2 with WSLg, Rust stable (see `rust-version` in `Cargo.toml`):

```sh
cargo build --workspace
cargo test --workspace
```

Before sending a change: `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --workspace`.

## Rules
See [CLAUDE.md](CLAUDE.md) — it applies to humans too: small focused changes, English everywhere,
GPL-3-compatible dependencies only, mind the memory budget.

By contributing you agree your contribution is licensed under GPL-3.0-or-later.
