---
id: doc-5
title: Session-Context
type: specification
created_date: '2026-01-15 22:15'
---
# Session Context

> Orchestrates the entire slide extraction pipeline, manages session state, and coordinates communication between all bounded contexts.

---

## Table of Contents

1. [Responsibility](#responsibility)
2. [Domain Types](#domain-types)
3. [State Machine](#state-machine)
4. [Handlers](#handlers)
5. [External Dependencies](#external-dependencies)

---

## Responsibility

The Session Context is the central orchestrator that:

- **Coordinates** communication between all other contexts (Video, Transcription, Frame, Dedup, OCR, Document)
- **Manages** extraction session lifecycle from creation to completion/failure
- **Publishes** events to event bus for other contexts to react
- **Maintains** session state and progress tracking
- **Handles** session recovery and error management
- **Displays** progress to CLI users

---

## Domain Types

### VideoProcessingSession

Represents a single extraction session.

**Fields:**
- id - Unique session identifier
- video_id - Associated YouTube video
- state - Current session state
- config - Extraction configuration
- created_at - Session creation timestamp
- completed_at - Optional completion timestamp

### SessionState

State machine for session lifecycle.

**States:**
- Created - Session initialized
- Downloading - Video download in progress
- Transcribing - Audio transcription in progress
- Extracting - Frame extraction in progress
- Processing - Slide identification and OCR in progress
- Generating - Markdown generation in progress
- Completed - Session finished successfully
- Failed(reason) - Session failed with error message

**Methods:**
- `is_terminal()` - Returns true for Completed/Failed states
- `is_processing()` - Returns true for active processing states

---

## State Machine

### State Transitions

```
Created → Downloading → Transcribing → Extracting → Processing → Generating → Completed
     ↓              ↓                ↓             ↓              ↓
  Failed         Failed           Failed         Failed         Failed
```

**Transitions:**
- Created → Downloading: Start video download
- Downloading → Transcribing: Download complete, start audio extraction
- Transcribing → Extracting: Transcription complete, start frame extraction
- Extracting → Processing: Frame extraction complete, start processing
- Processing → Generating: Processing complete, start document generation
- Generating → Completed: Document generation complete
- Any state → Failed: Error occurs

### Event Application

Events trigger state transitions:

- VideoDownloaded → Transcribing
- TextTranscribed → Extracting
- MarkdownGenerated → Completed
- Error events → Failed

---

## Handlers

### Start Extraction

Creates a new session from user request.

**Input:** ExtractSlidesFromVideoCommand
- url - YouTube video URL
- config - Extraction configuration

**Output:** SessionStarted event
- session_id - New session identifier
- video_id - Extracted video ID
- config - Validated configuration
- created_at - Current timestamp

**Validation:**
- Validates YouTube URL
- Validates extraction configuration

---

### Download Video

Downloads video from YouTube.

**Input:** DownloadVideoCommand
- video_id - Video to download

**Output:** VideoDownloaded event
- video_id - Video identifier
- path - Downloaded file path
- duration - Video duration

**Behavior:**
- Uses yt-dlp for download
- Retries up to 3 times with exponential backoff
- Returns error on final failure

---

### Extract Audio

Extracts audio track from downloaded video.

**Input:** ExtractAudioCommand
- video_id - Video to extract audio from

**Output:** AudioExtracted event
- video_id - Video identifier
- path - Audio file path
- duration - Audio duration

**Behavior:**
- Uses FFmpeg to extract audio
- Converts to 16kHz mono WAV format
- Returns error on extraction failure

---

### Transcribe Audio

Transcribes audio to text using Whisper.

**Input:** TranscribeAudioCommand
- video_id - Video to transcribe
- model_size - Whisper model size

**Output:** TextTranscribed event
- video_id - Video identifier
- transcript - Full transcription text
- segments - Timestamped segments

**Behavior:**
- Uses Whisper model (Tiny/Base/Small/Medium/Large)
- Returns error on transcription failure
- Continues with warning if transcription fails (non-blocking)

---

### Extract Frame

Extracts a single frame from video.

**Input:** ExtractFrameCommand
- session_id - Session identifier
- timestamp - Frame timestamp

**Output:** FrameExtracted event
- session_id - Session identifier
- frame_number - Sequential frame number
- timestamp - Frame timestamp
- hash - Perceptual hash

**Behavior:**
- Uses FFmpeg to extract frame
- Computes perceptual hash for deduplication
- Returns error on extraction failure

---

### Identify Unique Slides

Identifies unique slides from extracted frames.

**Input:** IdentifyUniqueSlideCommand
- session_id - Session identifier
- frame_hashes - List of frame IDs with hashes

**Output:** Vec<UniqueSlideIdentified>
- slide_id - New slide identifier
- frame_id - Source frame identifier
- image_path - Saved slide image path

**Behavior:**
- Compares hashes against existing slides
- Saves unique slide images
- Returns list of new unique slides

---

### Extract Text

Extracts text from slide image using OCR.

**Input:** ExtractTextCommand
- slide_id - Slide to extract text from

**Output:** TextExtracted event
- slide_id - Slide identifier
- text - Extracted text
- confidence - OCR confidence score (0.0-1.0)
- language - Detected language

**Behavior:**
- Uses Tesseract for OCR
- Returns error on OCR failure
- Extracts slide without text on OCR failure (warning)

---

### Verify Slide

Verifies if frame contains slide using Cloud LLM.

**Input:** VerifySlideCommand
- slide_id - Slide to verify
- image_path - Slide image path
- config - LLM configuration

**Output:** SlideVerified event
- slide_id - Slide identifier
- is_slide - True if slide detected
- reason - LLM explanation

**Behavior:**
- Sends image to LLM for verification
- Tags non-slides for human review
- Returns error on LLM failure (non-blocking)

---

### Generate Markdown

Generates Markdown document from slides.

**Input:** GenerateMarkdownCommand
- session_id - Session identifier

**Output:** MarkdownGenerated event
- session_id - Session identifier
- path - Generated document path
- slide_count - Number of slides in document

**Behavior:**
- Retrieves all slides for session
- Generates Markdown with embedded images
- Includes timestamps if configured
- Returns error on generation failure

---

### Complete Session

Marks session as completed.

**Input:** CompleteSessionCommand
- session_id - Session to complete

**Output:** SessionCompleted event
- session_id - Session identifier
- duration - Total session duration

**Behavior:**
- Calculates total duration
- Marks session as terminal state
- Cleans up temporary files

---

## External Dependencies

| Dependency | Type | Purpose |
|------------|------|---------|
| VideoDownloader | Async function | Download video from YouTube |
| AudioExtractor | Async function | Extract audio from video |
| WhisperTranscriber | Async function | Transcribe audio to text |
| FrameExtractor | Async function | Extract frame and compute hash |
| SlideRepository | Async trait | CRUD operations for slides |
| OcrEngine | Async function | Extract text from slide image |
| LlmVerifier | Async function | Verify slide content with LLM |
| MarkdownGenerator | Async function | Generate markdown document |
| SessionRepository | Async trait | CRUD operations for sessions |

---

## Testing

### Unit Tests

Test state transitions:
- Verify session state machine transitions
- Validate terminal state detection
- Validate processing state detection

### Integration Tests

Test full session flow:
- Start session → Download → Transcribe → Extract → Process → Generate
- Verify all events are published
- Verify session state progresses correctly
- Verify cleanup on failure

### Mock Dependencies

Use mock implementations for:
- VideoDownloader - Simulate download
- AudioExtractor - Simulate audio extraction
- WhisperTranscriber - Simulate transcription
- OcrEngine - Simulate OCR
- LlmVerifier - Simulate verification
- MarkdownGenerator - Simulate document generation

---

## CLI Integration

### Progress Display

Displays real-time progress using multi-progress bars:

- Session-level bar for overall progress
- Task-level bars for active operations
- Spinner for in-progress operations
- Elapsed time and ETA estimates

### Error Reporting

Displays errors to user with:
- Clear error message
- Suggested actions
- Session identifier for debugging

### User Prompts

Prompts user for:
- Deletion of tagged slides (after completion)
- Confirmation before destructive operations
- Retry options on recoverable errors
