---
id: doc-4
title: Shared-Domain-Concepts
type: specification
created_date: '2026-01-15 22:15'
---
# Shared Domain Concepts

> Domain concepts shared across all bounded contexts in YouTube Video Slide Extractor.

---

## Table of Contents

1. [ID Types](#id-types)
2. [Domain Events](#domain-events)
3. [Commands](#commands)
4. [Error Types](#error-types)
5. [Value Objects](#value-objects)
6. [Common Types](#common-types)

---

## ID Types

### Newtype Pattern

Uses a phantom type wrapper for UUIDs to ensure type safety across different entities.

**Type Aliases:**
- `Id<VideoProcessingSession>` - Session identifier
- `Id<YouTubeVideo>` - Video identifier
- `Id<VideoFrame>` - Frame identifier
- `Id<Slide>` - Slide identifier
- `Id<AudioFile>` - Audio file identifier
- `Id<Transcript>` - Transcript identifier

**Methods:**
- `new()` - Creates a new random UUID
- `from_uuid(uuid)` - Wraps an existing UUID
- `as_uuid()` - Unwraps to get the underlying UUID

---

## Domain Events

### Event Overview

| Event | Payload | When Emitted |
|-------|---------|--------------|
| VideoUrlValidated | url, video_id | URL validated successfully |
| VideoDownloaded | video_id, path, duration | Download completes |
| AudioExtracted | video_id, path, duration | Audio extraction completes |
| TextTranscribed | video_id, transcript, segments | Transcription completes |
| FrameExtracted | session_id, frame_number, timestamp, hash | Frame captured |
| UniqueSlideIdentified | slide_id, frame_id, image_path | Slide determined unique |
| TextExtracted | slide_id, text, confidence, language | OCR completes |
| SlideVerified | slide_id, is_slide, reason | LLM verification completes |
| MarkdownGenerated | session_id, path, slide_count | Document created |
| SessionCompleted | session_id, duration | All steps done |

### Event Structure

All events are serializable and contain:
- Event name as enum variant
- Structured payload with event-specific data
- Timestamps for audit trail

### Event Flow

Commands are validated, current state is loaded, events are derived from command+state combination, applied to update state, then published to the event bus.

---

## Commands

### Command Overview

| Command | Input | Output | Side Effects |
|---------|-------|--------|--------------|
| ExtractSlidesFromVideo | url, config | SessionStarted | Creates session |
| DownloadVideo | video_id | VideoDownloaded | Saves video file |
| ExtractAudio | video_id | AudioExtracted | Extracts WAV audio |
| TranscribeAudio | video_id, model_size | TextTranscribed | Runs Whisper |
| ExtractFrame | session_id, timestamp | FrameExtracted | Captures frame |
| IdentifyUniqueSlide | frame_hashes | UniqueSlideIdentified | Saves slide images |
| ExtractText | slide_id | TextExtracted | Runs OCR |
| VerifySlide | slide_id, image_path, llm_config | SlideVerified | Runs Cloud LLM verification |
| GenerateMarkdown | session_id | MarkdownGenerated | Writes file |

### Command Structure

All commands are serializable and contain:
- Command name as enum variant
- Structured input data
- Configuration parameters

---

## Error Types

### Error Categories

**Input Validation:**
- InvalidUrl - Invalid YouTube URL format
- VideoUnavailable - Video deleted/private/restricted
- InvalidConfig - Invalid extraction configuration

**Processing Failures:**
- DownloadFailed - Download retries exhausted
- AudioExtractionFailed - FFmpeg audio extraction failed
- TranscriptionFailed - Whisper transcription failed
- FrameExtractionFailed - FFmpeg frame extraction failed
- HashComputationFailed - Perceptual hash calculation failed
- OcrFailed - Tesseract OCR processing failed
- MarkdownGenerationFailed - Markdown generation failed

**Output Failures:**
- NoUniqueSlidesFound - All frames were duplicates
- OutputDirectoryNotWritable - Cannot write to output directory

**System Failures:**
- InsufficientMemory - Memory limit exceeded
- NetworkTimeout - Connection timeout
- ExternalDependencyUnavailable - FFmpeg/Tesseract/Whisper not available

### Error Handling Strategy

All operations use `Result<T, ExtractionError>`. Recoverable errors include retry logic with exponential backoff. Unrecoverable errors fail the session gracefully.

---

## Value Objects

### HashValue

Represents a perceptual hash for frame comparison.

**Methods:**
- `new(bytes)` - Creates hash from byte array
- `similarity(other)` - Calculates Hamming distance similarity (0.0-1.0)

**Usage:** Compare frames to identify duplicates.

### VideoResolution

Stores video width and height.

**Validation:** Resolution must be at least 1280x720 for slide extraction.

### ExtractionConfig

Encapsulates all extraction parameters.

**Fields:**
- frame_interval - Duration between frame captures
- similarity_threshold - 0.0-1.0 threshold for duplicate detection
- ocr_languages - List of languages for OCR
- output_directory - Directory for output files
- include_timestamps - Include timestamps in output
- llm_config - Optional LLM verification configuration

**Validation:** Similarity threshold must be between 0.0 and 1.0.

---

## Common Types

### Language

Enumeration of supported languages.

**Variants:** English, Spanish, French, German, Japanese, Chinese, Korean

**Methods:** `iso_639_1_code()` - Returns 2-letter ISO code

### AudioFormat

Audio file format enumeration.

**Variants:** WAV, MP3, FLAC

**Validation:** Only WAV is supported for Whisper transcription.

### ModelSize

Whisper model size enumeration.

**Variants:** Tiny, Base, Small, Medium, Large

**Trade-offs:**
- Tiny: Fastest, lowest accuracy
- Small: Balanced speed/accuracy (recommended)
- Large: Slowest, highest accuracy

### TranscriptSegment

Segment of transcribed text with timestamps.

**Fields:**
- start_time - Segment start timestamp
- end_time - Segment end timestamp
- text - Transcribed text
- confidence - Optional confidence score

**Method:** `duration()` - Returns segment duration

### LlmConfig

Cloud LLM verification configuration.

**Fields:**
- api_key - API key for LLM service
- base_url - Base URL for LLM API
- model - Model name to use

---

## Key Business Rules

### URL Validation

Videos must be publicly accessible YouTube URLs. URL must start with `youtube.com` or `youtu.be`.

### Slide Uniqueness

Frames must exceed similarity threshold to be considered unique. Lower threshold = more slides, higher threshold = fewer slides.

### Audio Format

Audio must be 16kHz mono WAV format for Whisper compatibility.

### Temporary File Cleanup

All temporary files are cleaned up on completion or failure.

### Configuration Validation

- Frame interval must be >= 1 second
- Similarity threshold must be 0.0-1.0
- Output directory must be writable
- Model size must be valid Whisper model
