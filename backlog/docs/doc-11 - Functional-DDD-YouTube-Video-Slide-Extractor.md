---
id: doc-11
title: Functional-DDD-YouTube-Video-Slide-Extractor
type: specification
created_date: '2026-01-15 22:15'
---
# Functional DDD: YouTube Video Slide Extractor

> Functional Domain-Driven Design specification for a CLI tool that extracts unique slides from YouTube videos with OCR and Markdown output.

---

## Table of Contents

1. [Quick Reference](#quick-reference)
2. [Architecture Overview](#architecture-overview)
3. [Functional DDD Principles](#functional-ddd-principles)
4. [Context Map](#context-map)
5. [Context Documents](#context-documents)
6. [Implementation](#implementation)

---

## Quick Reference

### Core Flow
```
User Request → Validate Command → Load State → Derive Event → Apply Event → Publish Event
```

### Key Type Definitions
| Type | Purpose | Example |
|------|---------|---------|
| `Command` | Intent to change state | `ExtractSlidesFromVideo` |
| `Event` | Something that happened | `VideoDownloaded` |
| `State` | Current entity state | `VideoProcessingSession` |
| `Error` | Domain errors | `VideoUnavailable` |
| `Id<T>` | Typed identifier | `Id<VideoProcessingSession>` |

### External Dependencies
- **yt-dlp**: Video downloading
- **FFmpeg**: Frame extraction, audio extraction
- **Whisper**: Audio transcription
- **Tesseract**: OCR processing
- **Tera**: Markdown templating

---

## Architecture Overview

### System Architecture

```mermaid
graph TB
    User[User / Researcher]

    subgraph CLI["CLI Interface"]
        Cmd[Command Parser]
        Progress[Progress Display]
    end

    subgraph Session["Session Context"]
        State[State Machine]
        Events[Event Bus]
    end

    subgraph Pipeline["Processing Pipeline"]
        direction TB
        Download[Video Downloader<br/>yt-dlp]
        AudioExtract[Audio Extractor<br/>FFmpeg]
        Transcribe[Whisper Transcriber<br/>whisper-rs]
        Extract[Frame Extractor<br/>FFmpeg]
        Hash[Perceptual Hasher<br/>dHash/aHash]
        Dedup[Slide Deduplicator]
        OCR[OCR Engine<br/>Tesseract]
        Export[Markdown Exporter<br/>Tera]
    end

    User --> Cmd
    Cmd --> State
    State --> Events
    Events -->|commands| Pipeline
    Pipeline -->|events| State
    Progress --> User

    Download -->|VideoFile| AudioExtract
    AudioExtract -->|WAV Audio| Transcribe
    Transcribe -->|TranscribedText| Export
    AudioExtract -->|VideoFile| Extract
    Extract -->|Frames| Hash
    Hash -->|Hashes| Dedup
    Dedup -->|UniqueSlides| OCR
    OCR -->|Slides+Text| Export
    Export -->|Markdown| User

    style User fill:#e1f5ff
    style CLI fill:#fff4e1
    style Session fill:#e8f5e9
    style Pipeline fill:#f3e5f5
```

---

## Functional DDD Principles

### Core Concepts

**Data > Behavior**
- Entities are plain structs with data only
- Behavior implemented as pure functions
- No methods on domain objects

**Immutability**
- All domain types derive `Clone`
- State transitions create new values
- Use `&` references for read-only access

**Explicit State Transitions**
- Events are first-class citizens
- State changes via event application
- Event sourcing pattern by default

**Type-Driven Development**
- Strong typing for domain concepts
- Newtypes prevent primitive obsession
- Compile-time validation where possible

### Command Handler Lifecycle

The command handler follows a pure functional pattern:

1. Validate command input
2. Load current state
3. Derive event from command + state
4. Apply event to update state
5. Publish event to event bus

All handlers are pure functions that return a Result<event, error>.

---

## Context Map

```mermaid
graph TB
    subgraph CLI["CLI INTERFACE"]
        C1[Command Parser]
        C2[Progress Display]
    end

    subgraph SESSION["SESSION CONTEXT"]
        S1[Command Handlers]
        S2[Event Store]
        S3[State Projections]
    end

    subgraph VIDEO["VIDEO CONTEXT"]
        V1[URL Validator]
        V2[Downloader]
        V3[Video State]
    end

     subgraph TRANSCRIPTION["TRANSCRIPTION CONTEXT"]
         T1[Audio Extractor]
         T2[Whisper Transcriber]
         T3[Model Manager]
         T4[Transcript State]
     end

    subgraph FRAME["FRAME CONTEXT"]
        F1[Frame Extractor]
        F2[Hash Calculator]
        F3[Frame State]
    end

    subgraph DEDUP["DEDUPLICATION CONTEXT"]
        D1[Uniqueness Checker]
        D2[Slide State]
    end

    subgraph OCR["OCR CONTEXT"]
        O1[Text Extractor]
        O2[Language Detector]
    end

    subgraph DOC["DOCUMENT CONTEXT"]
        M1[Markdown Generator]
        M2[Template Engine]
    end

    CLI -->|Commands| SESSION
    VIDEO -->|VideoDownloaded| SESSION
    TRANSCRIPTION -->|AudioExtracted| SESSION
    TRANSCRIPTION -->|TextTranscribed| SESSION
    FRAME -->|FrameExtracted| SESSION
    DEDUP -->|UniqueSlideIdentified| SESSION
    OCR -->|TextExtracted| SESSION
    DOC -->|MarkdownGenerated| SESSION

    style CLI fill:#e1f5ff
    style SESSION fill:#fff4e1
    style VIDEO fill:#e8f5e9
    style FRAME fill:#f3e5f5
    style DEDUP fill:#fce4ec
    style OCR fill:#fff9c4
    style DOC fill:#e0f2f1
```

### Context Implementation Pattern

Each bounded context follows this structure:

```
docs/
└── session/
    ├── domain/
    │   ├── commands.rs      // Command types
    │   ├── events.rs        // Event types
    │   ├── state.rs         // State types
    │   ├── handlers.rs      // Pure handler functions
    │   └── policies.rs      // Business rules
    ├── application/
    │   ├── service.rs       // Service coordination
    │   └── bus.rs           // Event bus
    └── infrastructure/
        ├── repository.rs    // State persistence
        └── publisher.rs     // Event publishing
```

---

## Context Documents

| Context | Document | Responsibility |
|---------|----------|----------------|
| **Shared** | [SHARED_FDD.md](SHARED_FDD.md) | Events, commands, errors, IDs |
| **Session** | [SESSION_FDD.md](SESSION_FDD.md) | Orchestration, state management |
| **Video** | [VIDEO_FDD.md](VIDEO_FDD.md) | YouTube interaction, download |
| **Transcription** | [TRANSCRIPTION_FDD.md](TRANSCRIPTION_FDD.md) | Audio extraction, speech-to-text |
| **Frame** | [FRAME_FDD.md](FRAME_FDD.md) | Frame extraction, hashing |
| **Deduplication** | [DEDUP_FDD.md](DEDUP_FDD.md) | Slide identification, verification |
| **OCR** | [OCR_FDD.md](OCR_FDD.md) | Text recognition |
| **Document** | [DOCUMENT_FDD.md](DOCUMENT_FDD.md) | Markdown generation |

### Context Dependencies

```
CLI Interface
    ↓
Session Context (orchestrator)
    ↓
    ├─→ Video Context ─────┐
    ├─→ Transcription Context ──┐
    ├─→ Frame Context ─────────┼─→ Document Context
    ├─→ Deduplication Context ──┘
    └─→ OCR Context ────────┘
```

---

## Implementation

### Feature List (FDD)

### Video Acquisition Feature Set
- Validate YouTube URL
- Fetch video metadata
- Download video file with retry logic

### Frame Processing Feature Set
- Extract frames at regular intervals
- Compute perceptual hash for each frame

### Deduplication Feature Set
- Compare frames using similarity threshold
- Identify unique slide candidates
- [NEW] Verify slide content using Cloud LLM
- [NEW] Tag non-slide frames for human review

### OCR Feature Set
- Extract text from slide images
- Detect slide language
- Filter results by confidence threshold

### Document Generation Feature Set
- Format slide content as Markdown
- Embed slide images and timestamps
- [NEW] Ask for deletion of human-review tagged slides

### Session & CLI Feature Set
- Orchestrate end-to-end pipeline
- Display real-time progress
- Handle session recovery
- [NEW] Prompt for conditional cleanup of tagged frames

### Testing Strategy

```
        Property Tests (20%)
    Invariant preservation

        Unit Tests (50%)
    Pure functions, handlers

    Integration Tests (30%)
    External dependencies
```

### Success Metrics

**Functional Requirements**
- Extract unique slides from YouTube videos (95% accuracy)
- Generate Markdown with embedded images
- Extract text via OCR (80%+ accuracy on clear text)
- Transcribe audio from YouTube videos (English, Whisper Small model)
- Handle videos up to 4 hours length

**Quality Metrics**
- Test coverage > 80%
- Zero unsafe code in domain layer
- All property tests passing

**Performance Metrics**
- < 5 minutes processing for 30-minute video
- Memory usage < 500MB during processing
- Zero-copy operations where possible

### Implementation Phases

| Phase | Duration | Deliverables |
|-------|----------|--------------|
| 1: Core Types | Week 1 | ID types, error types, basic event/command structures |
| 2: Video Context | Week 2-3 | URL validation, video download, video events |
| 3: Frame Context | Week 4 | Frame extraction, hashing, frame events |
| 4: Deduplication Context | Week 5 | Similarity calculation, slide identification |
| 5: OCR Context | Week 6-7 | Text extraction, language detection |
| 6: Document Context | Week 8 | Markdown generation, templating |
| 7: Session Context | Week 9 | Orchestration, event bus, state projections |
| 8: CLI & Polish | Week 10-11 | CLI interface, progress display, error messages |
| 9: Testing | Week 12 | Unit, integration, property tests |
| 10: Documentation | Week 13-14 | User docs, API docs, architecture docs |

---

## References

Inspired by:
- Functional DDD patterns from PensionBee ddd-workshop
- Scott Wlaschin's "Domain Modeling Made Functional"
- Event sourcing patterns
- Rust's ownership and type system for domain modeling
