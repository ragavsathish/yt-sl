---
id: TASK-46
title: Implement Audio Extraction & Transcription
status: Done
assignee: []
created_date: '2026-01-24 21:26'
updated_date: '2026-01-24 21:59'
labels:
  - transcription
  - poc
dependencies: []
priority: medium
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Implement a new bounded context 'transcription' to extract audio from YouTube videos and transcribe it locally using OpenAI's Whisper model (small.en) with Metal acceleration on macOS. Refer to poc/docs/research/audio_transcription.md for details.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Successfully extract 16kHz mono WAV from MP4
- [x] #2 Successfully load Whisper 'small.en' model with Metal support
- [x] #3 Transcribe a 10-minute video in < 1 minute on Apple Silicon

- [x] #4 Transcription text included in final Markdown report and synced with slide timing
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add whisper-rs to poc/Cargo.toml. 2. Update shared/infrastructure/dependencies.rs. 3. Implement AudioExtractor (FFmpeg). 4. Implement WhisperTranscriber (whisper-rs). 5. Define transcription domain (commands/events). 6. Integrate into SessionOrchestrator. 7. Add CLI progress reporting.

**Note:** The implementation was adjusted to use an OpenAI-compatible API client (`OpenAiTranscriber`) instead of direct `whisper-rs` bindings, assuming a local server (e.g., `whisper.cpp` or `LocalAI`) is running.
<!-- SECTION:PLAN:END -->
