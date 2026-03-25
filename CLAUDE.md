# LocalMonitor — Development Standards

## Project Overview

Terminal-based system monitor for Windows. Rust + Ratatui + SQLite.

- Spec: `docs/superpowers/specs/2026-03-25-local-monitor-design.md`
- Plan: `docs/superpowers/plans/2026-03-25-local-monitor-plan.md`

## Rust Standards

### Error Handling

- Use `anyhow::Result` for application-level errors (main, DB, collectors).
- Use `thiserror` for library-style errors only if we ever expose a public API (not now).
- Never use `.unwrap()` in production code. Use `.unwrap_or_default()`, `if let`, `match`, or `?`.
- `.unwrap()` is acceptable ONLY in tests and `const` contexts.
- For optional hardware data (GPU, CPU temp), use `Option<T>` — never sentinel values like `-1.0`.

### Ownership & Borrowing

- Prefer borrowing (`&T`, `&mut T`) over cloning. Clone only when ownership transfer is genuinely needed.
- Use `&str` in function parameters, `String` for owned fields in structs.
- Avoid `Rc`/`Arc` unless there's a genuine shared ownership need — this is a single-threaded app.
- Prefer stack allocation. Use `Box<dyn Trait>` only for the collector/panel registries where dynamic dispatch is the design.

### Types & Safety

- Use newtypes or enums over raw primitives for domain concepts (e.g., `Granularity` enum, not `u32`).
- Make illegal states unrepresentable: if a value can't be negative, use `u64`, not `i64`.
- Derive `Debug, Clone` on all data types. Derive `Copy` when the type is small and stack-only.
- No `unsafe` code. Period. There is no need for it in this project.

### Performance

- This app runs a 1-second tick loop. Do not over-optimize — clarity beats nanoseconds.
- Avoid allocations in the hot render path where easy (reuse buffers, use `&[DataPoint]` not `Vec<DataPoint>`).
- SQLite queries should use parameterized statements (no string formatting with user data).
- Use `Cow<str>` only if profiling shows string allocation is a bottleneck. Otherwise, just use `String`.

### Code Organization

- One responsibility per file. If a file exceeds ~200 lines, it's probably doing too much.
- Module files (`mod.rs`) contain trait definitions and the registry/wiring. Implementations go in their own files.
- All `pub` items need to justify their visibility. Default to private, expose only what's needed.
- No `pub use` re-exports unless there's a clear ergonomic win at the crate root.

### Naming

- Follow Rust naming conventions strictly: `snake_case` for functions/variables, `CamelCase` for types, `SCREAMING_SNAKE` for constants.
- Collector files match their metric: `cpu.rs`, `ram.rs`, `gpu.rs`.
- HUD panel files match their collector counterpart.
- Graph view files describe their content: `cpu_ram.rs`, `gpu_vram.rs`.

### Testing

- Unit tests go in `#[cfg(test)] mod tests` at the bottom of the file they test.
- Use `Db::open_in_memory()` for all DB tests — never touch the filesystem.
- Test data logic, not rendering. Ratatui rendering is tested by running the app.
- Assertions should be specific: `assert!((val - expected).abs() < 0.01)` for floats, not `assert!(val > 0.0)`.

### Dependencies

- Minimize dependencies. Every crate is attack surface and compile time.
- Current approved deps: `ratatui`, `crossterm`, `rusqlite` (bundled), `sysinfo`, `nvml-wrapper`, `dirs`, `chrono`, `anyhow`.
- Do not add new dependencies without justification. Prefer stdlib solutions.

### Formatting & Linting

- `cargo fmt` before every commit.
- `cargo clippy -- -D warnings` must pass with zero warnings.
- Fix clippy suggestions, don't suppress them with `#[allow(...)]` unless there's a documented reason.

### Git Practices

- Small, focused commits. One logical change per commit.
- Commit messages: `feat:`, `fix:`, `refactor:`, `test:`, `docs:` prefixes.
- No large "WIP" or "various fixes" commits.
- Don't commit `.db` files, `/target`, or IDE config.

## Architecture Invariants

- **Collectors never panic.** If hardware is unavailable, return `None`/default — never crash.
- **UI never blocks.** Rendering must complete in < 16ms. DB queries in the render path are reads only.
- **DB is optional.** If SQLite fails, the app runs in display-only mode with a warning banner.
- **Modular by design.** Adding a new metric = new collector + new HUD panel + optional graph view. No changes to core loop.
