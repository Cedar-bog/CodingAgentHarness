# Task 1: Convert to Cargo Workspace + Shared Types

**Files:**
- Create: `crates/harness-core/src/lib.rs`
- Create: `crates/harness-core/src/types.rs`
- Create: `crates/harness-core/src/error.rs`
- Modify: `Cargo.toml` (convert to workspace)
- Delete: `src/main.rs`

**Interfaces:**
- Consumes: none (foundation)
- Produces: `Message`, `Role`, `ToolCall`, `ToolResult`, `Action`, `HarnessError` used by all crates

## Steps

1. Convert root `Cargo.toml` to workspace:

```toml
[workspace]
members = ["crates/*"]
resolver = "2"

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"
thiserror = "2"
```

2. Create `crates/harness-core/Cargo.toml`:

```toml
[package]
name = "harness-core"
version = "0.1.0"
edition = "2024"

[dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
```

3. Create `crates/harness-core/src/types.rs` with: `Message`, `Role`, `ToolCall`, `ToolResult`, `Action`, `FinishReason`, `Usage`, `CompletionResponse`, `ToolSchema`, `FunctionSchema`

4. Create `crates/harness-core/src/error.rs` with `HarnessError` enum and `Result<T>` type alias

5. Create `crates/harness-core/src/lib.rs` with re-exports

6. Delete `src/main.rs`

7. Run `cargo check --workspace` to verify compilation

8. Commit
