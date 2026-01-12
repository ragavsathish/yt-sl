# Developer Agent Guide: YouTube Video Slide Extractor

This document provides instructions for agentic coding agents operating in this repository.

## Project Architecture
The project follows a **Domain-Driven Design (DDD)** approach with bounded contexts located in `poc/src/contexts/`.

- **Contexts**: `video`, `frame`, `dedup`, `ocr`, `document`, `session`.
- **Layers**: Each context typically has `domain/` (logic, commands, events, state) and `infrastructure/` (external integrations like FFmpeg or Tesseract).
- **Shared**: Common types, error handling, and cross-cutting concerns are in `poc/src/shared/`.

## Common Commands

### Build and Check
```bash
cargo build -p poc
cargo check -p poc
```

### Testing
Run all tests in the `poc` crate:
```bash
cargo test -p poc
```

Run tests for a specific context:
```bash
cargo test -p poc --lib contexts::frame
```

Run a single specific test:
```bash
cargo test -p poc --lib contexts::dedup::infrastructure::comparer::tests::test_calculate_similarity_identical
```

### Linting
```bash
cargo clippy -p poc --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

## Code Style & Guidelines

### 1. Error Handling
- Use the central `ExtractionError` enum located in `poc/src/shared/domain/error.rs`.
- Functions should return `DomainResult<T>`, which is a type alias for `Result<T, ExtractionError>`.
- Use `thiserror` for defining errors and `anyhow` is generally avoided in the domain layer.

### 2. Strong Typing & IDs
- Use the `Id<T>` wrapper for UUIDs to ensure type safety across different entities (e.g., `Id<YouTubeVideo>`, `Id<VideoFrame>`).
- Define markers in `shared/domain/error.rs` or locally if specific to a context.

### 3. Asynchronous Programming
- Use `tokio` for async/await operations.
- Long-running or I/O bound operations (Downloading, FFmpeg, OCR) **must** be async.

### 4. Logging & Progress
- Use the `tracing` crate for logging (`info!`, `warn!`, `error!`).
- For user-facing progress in the CLI, use the `CliProgressReporter` wrapper around `indicatif`.

### 5. Imports and Formatting
- Group imports: Standard library first, then external crates, then internal modules.
- Always run `cargo fmt` before committing.
- Use absolute paths for internal imports within the crate (e.g., `crate::shared::domain...`).

### 6. Naming Conventions
- Types/Structs/Enums: `PascalCase`.
- Functions/Variables/Modules: `snake_case`.
- Commands: `<Action><Entity>Command` (e.g., `ExtractTextCommand`).
- Events: `<Entity><Action>ed` (e.g., `TextExtracted`).

### 7. Testing Strategy
- Unit tests should be in a `mod tests` block at the bottom of the file they test, guarded by `#[cfg(test)]`.
- Use `tempfile` for any tests requiring filesystem access.
- Mock external dependencies where possible to keep tests fast and deterministic.

## External Dependencies
- **yt-dlp**: YouTube downloads.
- **FFmpeg**: Frame extraction.
- **Tesseract**: OCR.
- **Tera**: Markdown templating.

When adding new infrastructure, ensure the external tool is checked for availability using the patterns found in `poc/src/shared/infrastructure/dependencies.rs`.
