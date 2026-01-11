# Functional DDD POC - YouTube Slide Extractor

Proof of concept demonstrating Functional Domain-Driven Design (FDDD) architecture for a YouTube video slide extractor.

## Overview

This POC implements the core FDDD patterns from `../docs/EVENTSTORMING_FDD.md`:

- **Data > Behavior**: Domain entities are plain structs with data only
- **Immutability**: All types derive `Clone`, state transitions create new values
- **Explicit State Transitions**: Events are first-class citizens
- **Type-Driven Development**: Strong typing via newtype pattern `Id<T>`

## Architecture

```
poc/
├── src/
│   ├── shared/domain/          # Core domain primitives
│   │   ├── id.rs             # Typed identifier: Id<T>
│   │   └── error.rs          # Domain errors: ExtractionError
│   ├── shared/infrastructure/  # Cross-cutting concerns
│   │   └── event_bus.rs      # Event types
│   └── contexts/video/        # Video bounded context
│       ├── domain/
│       │   ├── commands.rs    # Intent: DownloadVideoCommand
│       │   ├── events.rs      # What happened: VideoUrlValidated, VideoDownloaded
│       │   ├── handlers.rs    # Pure functions: handle_download_video()
│       │   └── state.rs       # Data structures: YouTubeVideo
│       └── infrastructure/
│           └── downloader.rs  # External dependency: VideoDownloader trait
└── Cargo.toml
```

## Running Tests

```bash
cd poc
cargo test              # Run all tests (22 tests)
cargo test -- --nocapture  # Show print output
```

## What's Implemented

### Core Domain Types
- `Id<T>`: Newtype pattern for typed identifiers with PhantomData
- `ExtractionError`: Domain-specific error types using `thiserror`
- `DomainResult<T>`: Type alias for `Result<T, ExtractionError>`

### Video Context
- **Commands**: `ValidateUrlCommand`, `DownloadVideoCommand`
- **Events**: `VideoUrlValidated`, `VideoDownloaded`
- **State**: `YouTubeVideo`, `VideoUrlValidated`, `VideoDownloaded`
- **Handlers**: `validate_video_url()`, `extract_video_id()`, `handle_download_video()`
- **Infrastructure**: `VideoDownloader` trait with `MockVideoDownloader`

### Test Coverage
- 22 unit tests, all passing
- TDD approach: tests written before implementation
- Simple implementations focused on making tests pass

## What's Missing

### Bounded Contexts
- **Session**: Orchestration, event bus, state projections
- **Frames**: Frame extraction, hashing, frame events
- **Deduplication**: Similarity calculation, slide identification
- **OCR**: Text extraction, language detection
- **Document**: Markdown generation, templating
- **CLI Interface**: User interaction, progress display

### Domain Events
- `FrameExtracted`
- `UniqueSlideIdentified`
- `TextExtracted`
- `MarkdownGenerated`
- `SessionCompleted`

### Pure Functions
- `extract_frame`, `compute_hash`, `is_unique_slide`
- `extract_text`, `generate_markdown`

### Infrastructure
- Real external integrations (yt-dlp, FFmpeg, Tesseract)
- Repository pattern for persistence
- Event publisher implementation
- Application service coordination

### Entry Point
- No `main.rs` CLI application

## Extending the POC

### Adding a New Bounded Context

```rust
// src/contexts/{context}/domain/state.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frame {
    pub id: Id<Frame>,
    pub timestamp: Duration,
}

// src/contexts/{context}/domain/events.rs
pub fn extract_frame(video: &Video, timestamp: Duration) -> DomainResult<Frame> {
    // Pure function implementation
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_extract_frame() {
        // Write test first (TDD)
    }
}
```

### Following TDD Workflow

1. Write a failing test
2. Write minimal code to make it pass
3. Run tests: `cargo test`
4. Evaluate SOLID principles
5. Refactor if needed

## Code Style

- 4 spaces, no tabs
- Max line width: 100 characters
- No comments in production code (unless asked)
- Newtype pattern for IDs: `Id<T>(Uuid)` with `PhantomData`
- Prefer `&str` over `String` for function parameters
- Domain types must derive `Clone`, `Debug`, `Serialize`, `Deserialize`

## Next Steps

1. Implement Frames context with extraction logic
2. Add Deduplication context with perceptual hashing
3. Implement Session orchestration layer
4. Add CLI entry point with clap
5. Add integration tests with real external calls
6. Add property-based tests with proptest
