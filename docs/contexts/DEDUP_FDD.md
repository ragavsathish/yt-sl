# Deduplication Context

> Identifies unique slides from extracted frames using perceptual hashing and optional LLM verification.

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

The Deduplication Context is responsible for:

- **Identifying** unique slides from extracted frames
- **Comparing** frames using perceptual hash similarity
- **Tagging** frames as unique or duplicate
- **Verifying** slide content using Cloud LLM (optional)
- **Managing** slide storage and retrieval

---

## Domain Types

### Slide

Represents a unique slide.

**Fields:**
- id - Unique slide identifier
- session_id - Associated session identifier
- source_frame_id - Frame this slide was extracted from
- image_path - Path to saved slide image
- extracted_text - Optional text from OCR
- ocr_confidence - Optional OCR confidence score
- language - Optional detected language
- requires_human_review - True if LLM flagged as non-slide

---

## Handlers

### Identify Unique Slide

Compares frame hashes to identify unique slides.

**Input:** IdentifyUniqueSlideCommand
- session_id - Session identifier
- frame_hashes - List of frame IDs with hashes

**Output:** Vec<UniqueSlideIdentified>
- slide_id - New slide identifier
- frame_id - Source frame identifier
- image_path - Saved slide image path

**Behavior:**
- Compares each frame hash against existing hashes
- Creates slide for frames above similarity threshold
- Saves slide image to storage
- Returns list of new unique slides

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
- Sends image to LLM API
- LLM classifies as slide or not
- Tags slides as requiring review if not slide
- Returns error on LLM failure (non-blocking)

---

## Policies

### Similarity Threshold

Frames must exceed threshold to be considered unique.

**Default Threshold:** 0.9 (90% match)

**Configurable Range:** 0.0-1.0

**Behavior:**
- Lower threshold = more slides (less filtering)
- Higher threshold = fewer slides (more filtering)

**Guidelines:**
- 0.95: Aggressive filtering (fewer duplicates)
- 0.90: Balanced filtering (recommended)
- 0.80: Conservative filtering (more slides, risk of duplicates)

### Hash Comparison Strategy

Compares new hashes against existing hashes.

**Algorithm:**
- For each new frame hash:
  - Compare against all existing slide hashes
  - Calculate similarity score for each
  - If any similarity >= threshold, mark as duplicate
  - Otherwise, create new slide

**Optimization:**
- Early exit when match found
- Can use locality-sensitive hashing (LSH) for large datasets

---

### LLM Verification Strategy

Uses Cloud LLM to verify slide content.

**Configuration:**
- API key for LLM service
- Base URL for API endpoint
- Model name (e.g., gpt-4-vision, claude-3-opus)

**Prompt Strategy:**
```
You are a slide detection expert. Analyze this image and determine if it contains a presentation slide.

Consider:
- Text content and structure
- Visual layout (bullet points, diagrams, charts)
- Professional presentation style
- Presence of speaker/people (indicates not a slide)

Respond with:
1. Is this a presentation slide? (yes/no)
2. Brief explanation

Format: {"is_slide": true/false, "reason": "..."}
```

**Decision:**
- is_slide = true: Keep slide
- is_slide = false: Tag for human review
- requires_human_review = !is_slide

**Fallback:**
- If LLM fails, skip verification
- Keep slide without tagging
- Log warning about LLM failure

---

## External Dependencies

### SlideRepository

Manages slide storage and retrieval.

**Methods:**
- `create_slide(session_id, frame_id, hash) -> Result<slide_id, error>`
- `is_duplicate_hash(&hash) -> Result<bool, error>`
- `get_image_path(&slide_id) -> Result<path, error>`
- `get_slides_for_session(&session_id) -> Result<Vec<Slide>, error>`
- `update_slide(&slide) -> Result<(), error>`

**Implementation:** FilesystemSlideRepository
- Stores slides as PNG images
- Maintains index of hashes
- Uses JSON for metadata

### LlmVerifier

Verifies slide content with Cloud LLM.

**Signature:**
- `verify(&image_path, &config) -> Result<(is_slide, reason), error>`

**Implementation:**
- Sends image to LLM API
- Parses response to extract classification
- Returns result to caller

**Supported APIs:**
- OpenAI Vision API (gpt-4-vision-preview)
- Anthropic Claude (claude-3-opus)
- Custom OpenAI-compatible APIs

---

## Testing

### Unit Tests

Test similarity threshold:
- Verify frames with similarity >= threshold are duplicates
- Verify frames with similarity < threshold are unique
- Test edge cases (threshold = 0.0, threshold = 1.0)

Test hash comparison:
- Verify comparison against all existing hashes
- Verify early exit on match found
- Test with empty existing hashes

Test LLM verification parsing:
- Parse yes/no responses correctly
- Extract reason from response
- Handle malformed responses

### Integration Tests

Test slide identification:
- Process frames with known duplicates
- Verify only unique slides identified
- Verify correct number of slides

Test LLM verification:
- Verify slides with known classifications
- Test with sample slide images
- Test with non-slide images

### Mock Dependencies

**MockSlideRepository:**
- In-memory storage
- Returns mock slide data
- Can simulate failures

**MockLlmVerifier:**
- Returns mock classifications
- Simulates API delays
- Can simulate API failures

---

## Implementation Notes

### Hash Storage

Stores hashes in efficient data structure:

**Data Structure:** HashSet<HashValue>

**Advantages:**
- O(1) lookup time
- Automatic deduplication
- Memory efficient

**Persistence:**
- Save to JSON file after each update
- Load on session start
- File format: `[{"hash": "...", "slide_id": "..."}]`

### Slide Image Storage

Stores slide images as PNG files:

**Naming Convention:** `slide_<session_id>_<slide_id>.png`

**Storage Location:**
- Temporary directory during processing
- Moved to output directory on completion
- Tagged for review kept for manual inspection

### LLM API Integration

Uses HTTP client for LLM API:

**Request Format:**
- Content-Type: multipart/form-data
- Image data as file upload
- Prompt as text field
- Model name as parameter

**Response Format:**
- JSON with classification result
- Reason field for explanation

**Error Handling:**
- Retry with exponential backoff
- Timeout after 30 seconds
- Log error and continue on final failure

---

## Error Handling

### Hash Comparison Failures

**Causes:**
- Memory allocation failure
- Hash calculation error
- Data structure corruption

**Handling:**
- Log error with details
- Return HashComputationFailed error
- Skip frame and continue processing
- Mark session as failed if too many failures

### LLM Verification Failures

**Causes:**
- API authentication failure
- Network timeout
- Rate limiting
- Malformed response
- API unavailable

**Handling:**
- Log error with details
- Return LlmVerificationFailed error
- Continue without verification (non-blocking)
- Keep slide without tagging

### Storage Failures

**Causes:**
- Insufficient disk space
- Permission errors
- Filesystem full
- I/O errors

**Handling:**
- Log error with details
- Return StorageFailed error
- Mark session as failed
- Cleanup partial files

---

## Optimization Opportunities

### Locality-Sensitive Hashing (LSH)

Use LSH for faster hash comparison:

**Benefits:**
- Approximate nearest neighbor search
- O(log n) vs O(n) lookup
- Better for large slide sets

**Trade-offs:**
- Slightly higher memory usage
- Small accuracy loss
- More complex implementation

### Parallel Hash Comparison

Compare hashes in parallel:

**Implementation:**
- Use Rayon for parallel iteration
- Divide hashes into chunks
- Process chunks concurrently

**Benefits:**
- Faster comparison for large datasets
- Scales with CPU cores

### LLM Batch Processing

Verify multiple slides in single API call:

**Benefits:**
- Fewer API calls
- Lower latency
- Better rate limit utilization

**Limitations:**
- Not all APIs support batch
- Requires base64 encoding

---

## User Experience

### Progress Reporting

Reports deduplication progress:

**Metrics:**
- Total frames processed
- Unique slides found
- Duplicates filtered out
- LLM verifications completed

**Display:**
- Progress bar for frame comparison
- Counter for unique slides
- Spinner for LLM verification

### Human Review Workflow

Workflow for tagged slides:

1. Generate Markdown document
2. Count slides requiring review
3. Prompt user: "X slides were tagged as potentially not containing presentation content. Would you like to delete them? (y/N)"
4. If user confirms (y/Y):
   - Delete tagged slide images
   - Update document (remove references)
5. If user denies or provides no input:
   - Keep tagged slides
   - Note in document summary

**Summary Report:**
- Number of slides deleted
- Number of slides kept for review
- List of deleted slide IDs
- List of kept slide IDs
