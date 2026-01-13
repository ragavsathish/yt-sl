# Transcription Context

> Handles audio extraction from videos and speech-to-text transcription using Whisper.

---

## Table of Contents

1. [Responsibility](#responsibility)
2. [Domain Types](#domain-types)
3. [Handlers](#handlers)
4. [Policies](#policies)
5. [External Dependencies](#external-dependencies)
6. [Testing](#testing)

---

## Responsibility

The Transcription Context is responsible for:

- **Extracting** audio tracks from downloaded videos
- **Transcribing** audio to text using Whisper models
- **Managing** Whisper model caching and downloads
- **Segmenting** transcriptions with timestamps
- **Detecting** spoken language from audio

---

## Domain Types

### AudioFile

Represents extracted audio file.

**Fields:**
- id - Unique audio file identifier
- video_id - Associated video identifier
- path - File system path
- format - Audio format (WAV/MP3/FLAC)
- sample_rate - Sample rate in Hz
- channels - Number of audio channels
- duration - Audio duration

### Transcript

Represents full transcription.

**Fields:**
- id - Unique transcript identifier
- video_id - Associated video identifier
- text - Full transcribed text
- segments - Timestamped transcript segments
- language - Detected/specified language
- model_used - Whisper model size used
- confidence - Optional overall confidence

### TranscriptSegment

Represents timestamped text segment.

**Fields:**
- start_time - Segment start timestamp
- end_time - Segment end timestamp
- text - Transcribed text for segment
- confidence - Optional confidence score

**Method:** `duration()` - Returns segment duration

### AudioFormat

Audio file format enumeration.

**Variants:** WAV, MP3, FLAC

**Method:** `extension()` - Returns file extension

---

## Handlers

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
- Converts to 16kHz mono WAV format (required by Whisper)
- Returns error on extraction failure

---

### Transcribe Audio

Transcribes audio to text using Whisper.

**Input:** TranscribeAudioCommand
- video_id - Video to transcribe
- model_size - Whisper model size (Tiny/Base/Small/Medium/Large)

**Output:** TextTranscribed event
- video_id - Video identifier
- transcript - Full transcription text
- segments - Timestamped segments

**Behavior:**
- Loads Whisper model (cached or downloaded)
- Transcribes audio file
- Returns error on transcription failure
- Continues with warning if transcription fails (non-blocking)

---

## Policies

### Audio Format Requirements

Audio must meet Whisper requirements.

**Required Format:**
- Sample rate: 16000 Hz
- Channels: 1 (mono)
- Format: WAV

**Rationale:** Whisper models trained on 16kHz mono audio

**Error:** InvalidAudioFormat if requirements not met

---

### Model Size Selection

Selects Whisper model based on video duration and performance mode.

**Model Sizes:**
- Tiny: Fastest, lowest accuracy (~32MB)
- Base: Fast, low accuracy (~74MB)
- Small: Balanced (~244MB) - **Recommended**
- Medium: Slow, high accuracy (~769MB)
- Large: Slowest, highest accuracy (~1550MB)

**Selection Strategy:**

| Performance Mode | Default Model |
|----------------|---------------|
| Fast | Tiny |
| Balanced | Small |
| Accurate | Medium (if <10min), Small (if >10min) |

**Performance Mode Configuration:**
- Fast: Prioritize speed over accuracy
- Balanced: Balance speed and accuracy
- Accurate: Prioritize accuracy

---

### Confidence Filtering

Filters transcript segments by minimum confidence.

**Input:** List of transcript segments, minimum confidence threshold

**Output:** Filtered list of segments

**Behavior:**
- Removes segments with confidence below threshold
- Preserves segments without confidence value (default to include)

**Default Threshold:** 0.5 (50%)

---

## External Dependencies

### AudioExtractor

Extracts audio from video file.

**Signature:**
- `extract(&video_id) -> Result<AudioFile, error>`

**Implementation:** FFmpegAudioExtractor
- Uses FFmpeg command-line tool
- Extracts audio track
- Converts to 16kHz mono WAV format
- Returns error on extraction failure

**FFmpeg Flags:**
- `-vn` - No video
- `-acodec pcm_s16le` - 16-bit PCM codec
- `-ar 16000` - 16kHz sample rate
- `-ac 1` - Mono channel

---

### WhisperTranscriber

Transcribes audio to text.

**Signature:**
- `transcribe(&video_id, &model_size) -> Result<Transcript, error>`

**Implementation:** WhisperTranscriber
- Uses whisper-rs library
- Loads model from cache or downloads
- Transcribes audio file
- Returns error on transcription failure

**Transcription Configuration:**
- Language: English (default, auto-detect optional)
- Sampling strategy: Greedy (fast) or Beam search (accurate)
- Threads: Number of CPU cores
- Timestamps: Enabled for segments

---

### Model Caching

Models are cached locally to avoid re-downloads.

**Cache Location:** Configurable directory (default: ~/.cache/whisper/)

**Model Names:**
- `tiny.ggml`
- `base.ggml`
- `small.ggml`
- `medium.ggml`
- `large.ggml`

**Download URL:** HuggingFace (ggerganov/whisper.cpp)

**Download Behavior:**
- Check if model exists in cache
- Download from HuggingFace if not found
- Save to cache directory
- Verify model file integrity

---

## Testing

### Unit Tests

Test audio format validation:
- 16kHz mono WAV should pass
- Other sample rates should fail
- Stereo audio should fail
- Non-WAV formats should fail

Test model size selection:
- Fast mode should select Tiny model
- Balanced mode should select Small model
- Accurate mode should select Medium model (short video)
- Accurate mode should select Small model (long video)

Test confidence filtering:
- Segments above threshold should be included
- Segments below threshold should be filtered
- Segments without confidence should be included

### Integration Tests

Test audio extraction:
- Extract audio from sample video
- Verify output file format (WAV)
- Verify sample rate (16kHz)
- Verify channels (mono)

Test transcription:
- Transcribe sample audio
- Verify text is generated
- Verify segments have timestamps
- Test different model sizes

### Mock Dependencies

**MockAudioExtractor:**
- Simulates audio extraction delay
- Returns mock AudioFile objects
- Can simulate extraction failures

**MockWhisperTranscriber:**
- Simulates transcription delay
- Returns mock Transcript objects
- Can simulate transcription failures

---

## Implementation Notes

### FFmpeg Integration

Uses FFmpeg command-line tool for audio extraction:

**Command:**
```
ffmpeg -i <video_path> -vn -acodec pcm_s16le -ar 16000 -ac 1 <audio_path>
```

**Error Handling:**
- Check FFmpeg exit code
- Parse error messages
- Verify output file exists

### Whisper Integration

Uses whisper-rs library for transcription:

**Initialization:**
- Load model from file
- Create state for transcription
- Configure transcription parameters

**Transcription Process:**
- Set sampling strategy
- Set language (optional)
- Set thread count
- Execute transcription
- Extract segments with timestamps

### Performance Considerations

**Model Size Impact:**
- Tiny: ~10x faster, lower accuracy
- Small: ~5x faster, good accuracy
- Large: Full speed, highest accuracy

**CPU vs GPU:**
- CPU: Uses all available cores
- GPU: Requires CUDA/Metal support (future enhancement)

**Memory Usage:**
- Tiny: ~100MB
- Small: ~300MB
- Large: ~1.5GB

---

## Error Handling

### Audio Extraction Failures

**Causes:**
- FFmpeg not installed
- Video file corrupted
- Insufficient disk space
- Permission errors

**Handling:**
- Log error with details
- Return AudioExtractionFailed error
- Session continues with warning (non-blocking)

### Transcription Failures

**Causes:**
- Whisper model download failed
- Model file corrupted
- Memory insufficient
- Audio format incompatible

**Handling:**
- Log error with details
- Return TranscriptionFailed error
- Session continues without transcription (non-blocking)
- Document generated without transcript section

### Model Download Failures

**Causes:**
- Network connection issues
- HuggingFace API unavailable
- Insufficient disk space
- Permission errors

**Handling:**
- Retry with exponential backoff
- Log detailed error message
- Suggest manual download option
- Fail transcription operation
