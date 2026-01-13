# Research & Implementation Plan: Audio Extraction & Transcription

## 1. Objective
Extract audio from YouTube videos and transcribe it locally using OpenAI's Whisper model, optimized for speed and local execution on macOS (Apple Silicon).

## 2. Requirements
- **Speed**: Prioritize fast processing over maximum accuracy.
- **Model**: Use the "small.en" model (~464MB).
- **Language**: English only.
- **Environment**: Local execution with Metal (GPU) acceleration on macOS.

## 3. Architecture (DDD)
Following the project's Domain-Driven Design approach, we will introduce a new bounded context: `transcription`.

### 3.1 Bounded Context: `transcription`
- **Domain**: Commands (`ExtractAudio`, `TranscribeAudio`), Events (`AudioExtracted`, `TextTranscribed`), State (`TranscriptionResult`).
- **Infrastructure**:
    - `AudioExtractor`: FFmpeg-based implementation to extract 16kHz mono WAV.
    - `WhisperTranscriber`: `whisper-rs` implementation with Metal acceleration.

## 4. Implementation Details

### 4.1 Audio Extraction (FFmpeg)
Whisper requires audio in a specific format:
- **Format**: WAV (PCM)
- **Sample Rate**: 16,000 Hz
- **Channels**: 1 (Mono)
- **Bit Depth**: 16-bit

**Command**:
```bash
ffmpeg -i input.mp4 -vn -acodec pcm_s16le -ar 16000 -ac 1 output.wav
```

### 4.2 Transcription (whisper-rs)
- **Library**: [whisper-rs](https://codeberg.org/tazz4843/whisper-rs)
- **Feature Flags**: `metal` (for macOS GPU acceleration).
- **Model Management**:
    - Download `ggml-small.en.bin` on first use.
    - Store in `~/.cache/whisper/` or project `.cache/`.

### 4.3 Integration Pipeline
1. **Video Context**: Download video via `yt-dlp`.
2. **Transcription Context**:
    - Extract audio track using `FFmpeg`.
    - Run Whisper transcription on the extracted WAV.
3. **Document Context**: Append transcription text to the generated Markdown.

## 5. Implementation Steps

1. **Dependency Update**:
    - Add `whisper-rs` to `poc/Cargo.toml`.
    - Update `shared/infrastructure/dependencies.rs` to track Whisper model availability.
2. **Infrastructure Development**:
    - Implement `AudioExtractor` using `tokio::process::Command`.
    - Implement `WhisperTranscriber` using `whisper-rs`.
3. **Domain Development**:
    - Define commands, events, and results in the `transcription` context.
4. **Orchestration**:
    - Integrate the new context into `SessionOrchestrator`.
5. **UI/CLI**:
    - Add progress reporting for audio extraction and transcription phases.

## 6. Success Criteria
- [ ] Successfully extract 16kHz mono WAV from MP4.
- [ ] Successfully load Whisper "small.en" model with Metal support.
- [ ] Transcribe a 10-minute video in < 1 minute (on M1/M2/M3 chips).
- [ ] Transcription text included in final Markdown report.
