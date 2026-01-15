---
id: doc-8
title: Frame-Context
type: specification
created_date: '2026-01-15 22:15'
---
# Frame Context

> Handles frame extraction from videos and perceptual hash calculation for deduplication.

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

The Frame Context is responsible for:

- **Extracting** frames from video files at specified intervals
- **Computing** perceptual hashes for frame comparison
- **Managing** frame storage and cleanup
- **Providing** frame metadata (timestamp, resolution)

---

## Domain Types

### VideoFrame

Represents an extracted video frame.

**Fields:**
- id - Unique frame identifier
- session_id - Associated session identifier
- timestamp - Frame timestamp in video
- hash - Perceptual hash for comparison
- image_path - Path to saved frame image
- width - Frame width in pixels
- height - Frame height in pixels

---

## Handlers

### Extract Frame

Extracts a single frame from video at specified timestamp.

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
- Saves frame as PNG image
- Computes perceptual hash
- Returns error on extraction failure

---

## Policies

### Frame Interval

Frames are extracted at regular intervals.

**Default:** 5 seconds between frames

**Configurable:** 1-60 seconds

**Rationale:** 5 seconds balances slide coverage and processing time

---

### Frame Quality

Frames must meet minimum quality requirements.

**Minimum Resolution:** 1280x720 (720p)

**Format:** PNG (lossless compression)

**Rationale:** Lossless format preserves details for OCR

---

### Hash Algorithm

Perceptual hash algorithm for frame comparison.

**Algorithm:** Difference Hash (dHash)

**Parameters:**
- Hash size: 64 bits (8x8 block)
- Similarity threshold: 0.9 (90% match)

**Alternatives:**
- Average Hash (aHash) - Faster, less accurate
- Perceptual Hash (pHash) - Slower, more accurate

---

## External Dependencies

### FrameExtractor

Extracts frames from video file.

**Signature:**
- `extract_frame(&session_id, timestamp) -> Result<(frame_number, timestamp, hash), error>`

**Implementation:** FFmpegFrameExtractor
- Uses FFmpeg command-line tool
- Extracts frame at specified timestamp
- Saves frame as PNG image
- Returns error on extraction failure

**FFmpeg Flags:**
- `-ss <timestamp>` - Seek to timestamp
- `-vframes 1` - Extract single frame
- `-q:v 2` - High quality
- `-f image2pipe` - Output to pipe

---

### HashCalculator

Computes perceptual hash from image.

**Signature:**
- `compute_hash(&image) -> HashValue`

**Implementation:** DHashCalculator
- Uses image_hasher library
- Implements difference hash algorithm
- Returns 64-bit hash value

**Algorithm Steps:**
1. Convert image to grayscale
2. Resize to 8x8 pixels
3. Compute differences between adjacent pixels
4. Convert to 64-bit hash

---

## Testing

### Unit Tests

Test frame interval:
- Verify frames extracted at correct intervals
- Test edge cases (start/end of video)
- Test invalid timestamps (beyond video duration)

Test hash calculation:
- Verify hash is deterministic (same image = same hash)
- Verify hash similarity is symmetric
- Test different images produce different hashes

Test hash similarity:
- Identical images should have 100% similarity
- Similar images should have high similarity (>0.9)
- Different images should have low similarity (<0.5)

### Integration Tests

Test frame extraction:
- Extract frames from sample video
- Verify frame images are saved
- Verify timestamps are correct
- Test extraction at various timestamps

Test hash calculation:
- Calculate hashes for extracted frames
- Verify similarity calculations
- Test hash collision detection

### Mock Dependencies

**MockFrameExtractor:**
- Simulates frame extraction delay
- Returns mock frame data
- Can simulate extraction failures

**MockHashCalculator:**
- Returns deterministic mock hashes
- Can simulate hash computation failures

---

## Implementation Notes

### FFmpeg Integration

Uses FFmpeg command-line tool for frame extraction:

**Command:**
```
ffmpeg -ss <timestamp> -i <video_path> -vframes 1 -q:v 2 -f image2pipe -vcodec png -
```

**Output:** Binary PNG data to stdout

**Error Handling:**
- Check FFmpeg exit code
- Parse error messages
- Verify image data is valid

### Image Storage

Frames are saved to temporary directory:

**Naming Convention:** `slide_<session_id>_<frame_number>.png`

**Storage Location:** Temporary directory during processing

**Cleanup:**
- Frames deleted after processing
- Unique slide images preserved
- Cleanup runs even on failure

### Hash Computation

Uses difference hash algorithm:

**Advantages:**
- Fast computation
- Good for detecting similar images
- Resistant to minor changes

**Limitations:**
- May miss differences in similar slides
- Sensitive to cropping
- Not rotation-invariant

### Performance Considerations

**Frame Extraction Speed:**
- Depends on video resolution
- ~0.1-0.5 seconds per frame

**Hash Computation Speed:**
- ~0.01-0.05 seconds per frame
- Negligible compared to extraction

**Memory Usage:**
- ~5-10MB per frame in memory
- One frame processed at a time

---

## Error Handling

### Frame Extraction Failures

**Causes:**
- FFmpeg not installed
- Video file corrupted
- Timestamp beyond video duration
- Insufficient disk space
- Permission errors

**Handling:**
- Log error with details
- Return FrameExtractionFailed error
- Skip frame and continue processing
- Mark session as failed if too many failures

### Hash Computation Failures

**Causes:**
- Image data corrupted
- Memory allocation failure
- Hash calculation error

**Handling:**
- Log error with details
- Return HashComputationFailed error
- Skip frame and continue processing

### Storage Failures

**Causes:**
- Insufficient disk space
- Permission errors
- Filesystem full

**Handling:**
- Log error with details
- Return StorageFailed error
- Mark session as failed
- Cleanup partial files

---

## Optimization Opportunities

### Parallel Frame Extraction

Extract multiple frames concurrently:
- Use Rayon for parallel processing
- Extract frames in batches
- Limited by CPU and I/O

### Progressive Hash Computation

Compute hashes during extraction:
- Pipe FFmpeg output directly to hash calculator
- Avoid storing intermediate images
- Reduce memory usage

### GPU Acceleration

Use GPU for frame extraction:
- FFmpeg with CUDA/Metal support
- Faster extraction for high-resolution videos
- Requires GPU-compatible FFmpeg build
