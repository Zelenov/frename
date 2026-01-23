# frename - AI Context

## Project Overview

**Name**: frename  
**Type**: Rust GUI application for Windows  
**Purpose**: File renaming utility (functionality being defined incrementally)  
**Target Platform**: Windows 10/11  
**Rust Edition**: 2021  
**GUI Framework**: Iced (Elm-architecture GUI)

## Recommended Architecture Pattern: Agent-Skills-Commands

When implementing functionality, organize code using this pattern:

```
src/
├── main.rs           # Entry point, GUI setup, application state
├── ui/               # UI components and layouts
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
- **NEVER use Russian or any non-English language in code** - all code, comments, variable names, function names, documentation must be in English only

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

**Current**:
- `iced` (0.14.0-dev) - Cross-platform GUI framework with Elm architecture

**Guidelines**:
- Minimize dependencies
- Prefer well-maintained crates with Windows support
- Document why each dependency is needed (in this file or Cargo.toml)

## AI Collaboration Guidelines

### CRITICAL: User's Working Style
**The user works in SMALL, EXPLICIT steps. Follow these rules strictly:**

1. **ONLY implement what is explicitly requested** - nothing more
2. **Do NOT add extra files, documentation, or helpers** unless asked
3. **Do NOT anticipate next steps** or add "nice to have" features
4. **WAIT for explicit instruction** before proceeding to next step
5. **Minimal changes only** - if asked for a button, add ONLY a button

**Example:**
- User asks: "Create an empty window"
- ✅ Correct: Create main.rs with empty window, update Cargo.toml if needed
- ❌ Wrong: Also create README.md, INSTALL.md, setup scripts, documentation

**Work step-by-step. Stop after each step. Wait for next instruction.**

### Before Implementing
1. **Ask if requirements are unclear** - Don't assume functionality
2. **Confirm approach** for significant architectural changes
3. **Propose alternatives** with trade-offs for complex decisions

### When Coding
1. **Incremental implementation** - One feature/module at a time
2. **Minimal additions** - Only what is requested
3. **Update context** - Update CLAUDE.md only when working style changes

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

**Phase**: Basic GUI window implemented  
**Code Status**: Empty window with basic egui setup  
**Next Steps**: Define and implement file renaming UI and functionality

## Notes for AI

- This is a greenfield project - no legacy code to maintain
- User will define features incrementally - don't assume functionality
- Windows platform allows Windows-specific optimizations
- Safety and user experience are priorities (dry-run, preview, undo)