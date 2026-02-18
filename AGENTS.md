# Agent rules for frename

## Project context

- **frename**: Rust GUI file-renaming utility for Windows 10/11. Iced (Elm-architecture), Rust 2021. Functionality is defined incrementally; this is greenfield with no legacy code. Safety and UX matter (e.g. dry-run, preview, undo).

## Architecture and skills

**Agents must follow the Iced Elm architecture and apply the project’s Rust skills.**

- **Iced Elm architecture**  
  Read and follow `.cursor/skills/iced-elm-architecture/SKILL.md` when working on UI: feature layout (`messages`, `state`, `view`), message flow, `Task::done` dispatch, core/UI separation, subscriptions, persistence/session, and all patterns described there (no direct `update` across components, `update` returns `Task<Message>`, flatten with early returns, read high-frequency data at view time, etc.).

- **Rust skills**  
  Use the skills under `.cursor/skills/` whenever they apply:
  - **rust-async-programming** – async operations, Tokio, channels, timeouts
  - **rust-cargo-ecosystem** – Cargo.toml, workspaces, build profiles, tooling
  - **rust-concurrency** – threads, channels, parallel iterators
  - **rust-error-handling** – `Result`/`Option`, error types, propagation, thiserror/anyhow
  - **rust-ownership-borrowing** – ownership, borrowing, lifetimes
  - **rust-performance** – profiling, benchmarking, optimization
  - **rust-testing** – unit/integration tests, mocking, TDD
  - **rust-trait-generics** – traits, generics, trait objects, operator overloading

When a task touches one of these areas, read the corresponding skill and follow its guidance.

- **Code layout (when adding app logic)**  
  Prefer: `main.rs`, `ui/`, `commands/` (user-facing ops), `agents/` (orchestration), `skills/` (atomic capabilities). See `CLAUDE.md` for full layout.

## Coding standards

- **Rust**: `rustfmt` and `clippy`; `Result<T, E>` in production (no `.unwrap()`); explicit types in public APIs; descriptive names. **English only** in code, comments, names, and docs (no Russian or other languages).
- **Windows**: `std::path::Path` / `PathBuf`; case-insensitive filesystems; long paths (`\\?\`) when needed; `windows` crate for Win APIs.
- **Errors**: Custom types (e.g. thiserror), context in messages, no panic in library code, log appropriately.
- **Testing**: Unit tests next to source (`#[cfg(test)]`), integration in `tests/`, fixtures in `tests/fixtures/`, cover Windows path edge cases.

## AI collaboration (critical)

**Work in small, explicit steps.**

1. Implement **only** what is explicitly requested.
2. Do **not** add extra files, docs, or helpers unless asked.
3. Do **not** anticipate next steps or “nice to have” features.
4. Wait for explicit instruction before the next step.
5. Minimal changes (e.g. if asked for a button, add only that button).

Before implementing: ask when unclear; confirm approach for big changes; offer alternatives for complex decisions.  
When coding: one feature/module at a time; minimal additions; update `CLAUDE.md` only when working style or conventions change.  
Documentation: inline comments for non-obvious logic; `///` for public APIs; keep “why” in `docs/DECISIONS.md`.

For more detail (dependencies, common commands, project state), see `CLAUDE.md`.
