# 0.3.0 Drag and Drop File Handling

## Added
- Drag and drop functionality for files
- Window title updates to show dropped file path
- Feature-based architecture with separate drag_drop module
- Event subscription system for window file drop events

## Changed
- Refactored app.rs into modular feature structure
- Empty window now accepts files anywhere in window

---

# 0.2.0 GUI Application Structure

## Added
- GUI application using egui + eframe framework
- Basic empty window (800x600) with close functionality
- Project structure: Cargo.toml, src/main.rs
- README.md with project overview in Russian
- INSTALL.md with detailed Rust installation instructions
- check-setup.ps1 script for environment verification
- .gitignore for Rust projects

## Changed
- Updated CLAUDE.md: CLI → GUI application
- Architecture now includes ui/ directory for GUI components
- Application uses immediate mode GUI (egui)

## Technical
- Dependencies: eframe 0.30, egui 0.30
- Windows subsystem configuration for GUI mode
- Release profile optimizations (LTO, single codegen unit)

---

# 0.1.0 Initial project setup

## Added
- AI context documentation in CLAUDE.md file
- Changelog generation skill for version tracking
- Project structure with agent-skills-commands pattern
- Windows-specific Rust development guidelines

