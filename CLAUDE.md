# frename - AI Context

## Project Overview

**Name**: frename  
**Type**: Rust CLI application for Windows  
**Purpose**: File renaming utility (functionality being defined incrementally)  
**Target Platform**: Windows 10/11  
**Rust Edition**: 2021

## Recommended Architecture Pattern: Agent-Skills-Commands

When implementing functionality, organize code using this pattern:

```
src/
├── main.rs           # Entry point, CLI parsing, orchestration
├── commands/         # High-level user commands (rename, batch, undo, etc.)
├── agents/           # Task orchestration and workflow logic
└── skills/           # Atomic capabilities (pattern matching, file ops, validation)
```

**Pattern Explanation**:
- **Commands**: User-facing operations (what the user asks for)
- **Agents**: Coordinate multiple skills to accomplish commands (workflow orchestration)
- **Skills**: Atomic, reusable capabilities (building blocks)

This structure will be created as functionality is implemented.

## Coding Standards

### Rust Style
- Follow `rustfmt` and `clippy` recommendations
- Use `Result<T, E>` for error handling (avoid `.unwrap()` in production)
- Prefer explicit types in public APIs
- Use descriptive names (`file_path` not `fp`)

### Windows-Specific
- Use `std::path::Path` and `PathBuf` for paths (handles Windows separators)
- Consider case-insensitive file systems
- Handle long path names (\\?\) if needed
- Use `windows` crate for Windows-specific APIs when needed

### Error Handling
- Define custom error types using `thiserror` or similar
- Provide context in error messages
- Never panic in library code
- Log errors appropriately

### Testing
- Unit tests co-located with source (`#[cfg(test)]`)
- Integration tests in `tests/`
- Use test fixtures in `tests/fixtures/`
- Test Windows path edge cases

## Dependencies

**Current**: None (add as needed)

**Guidelines**:
- Minimize dependencies
- Prefer well-maintained crates with Windows support
- Document why each dependency is needed (in this file or Cargo.toml)

## AI Collaboration Guidelines

### Before Implementing
1. **Ask if requirements are unclear** - Don't assume functionality
2. **Confirm approach** for significant architectural changes
3. **Propose alternatives** with trade-offs for complex decisions

### When Coding
1. **Incremental implementation** - One feature/module at a time
2. **Test alongside** - Write tests with implementation
3. **Update context** - Update relevant CLAUDE.md files when patterns change

### Documentation
- Update `CLAUDE.md` files when patterns or conventions change
- Keep `docs/DECISIONS.md` for "why" explanations
- Add inline comments for non-obvious logic
- Doc comments (`///`) for public APIs

## Common Commands

```powershell
# Development
cargo check                    # Quick compile check
cargo build                    # Build debug version
cargo build --release          # Build optimized version
cargo run -- [args]            # Run with arguments

# Testing
cargo test                     # Run all tests
cargo test --test integration  # Run specific test file
cargo test -- --nocapture      # Show println! output

# Quality
cargo fmt                      # Format code
cargo clippy                   # Lint checks
cargo clippy -- -W clippy::pedantic  # Strict linting

# Documentation
cargo doc --open               # Build and open docs
```

## Project State

**Phase**: AI workflow organization complete, awaiting functionality definition  
**Code Status**: Minimal placeholder (main.rs only)  
**Next Steps**: Define first feature/functionality to implement

## Notes for AI

- This is a greenfield project - no legacy code to maintain
- User will define features incrementally - don't assume functionality
- Windows platform allows Windows-specific optimizations
- Safety and user experience are priorities (dry-run, preview, undo)