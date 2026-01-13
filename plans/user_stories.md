# User Stories: YouTube Video Slide Extractor

> Agile user stories derived from the Functional Domain-Driven Design specification for the CLI tool that extracts unique slides from YouTube videos with OCR and Markdown output.

---

## Table of Contents

1. [CLI Interface Epic](#cli-interface-epic)
2. [Session Orchestration Epic](#session-orchestration-epic)
3. [Video Context Epic](#video-context-epic)
4. [Frame Context Epic](#frame-context-epic)
5. [Deduplication Context Epic](#deduplication-context-epic)
6. [OCR Context Epic](#ocr-context-epic)
7. [Document Context Epic](#document-context-epic)
8. [Error Handling Epic](#error-handling-epic)

---

## CLI Interface Epic

### US-CLI-01: Parse Command Line Arguments

**As a** Researcher, Student, Content Creator, or Educator
**I want** to provide a YouTube URL and optional configuration parameters via command line arguments
**So that** I can customize the slide extraction process for my specific needs

**Acceptance Criteria:**
- CLI accepts a required YouTube URL argument
- CLI accepts optional `--interval` parameter for frame extraction interval (default: 5 seconds)
- CLI accepts optional `--threshold` parameter for slide similarity threshold (default: 0.85)
- CLI accepts optional `--output` parameter for output directory (default: current directory)
- CLI accepts optional `--languages` parameter for OCR languages (default: English)
- CLI accepts optional `--timestamps` flag to include timestamps in output
- CLI accepts optional `--llm-api-key` parameter for Cloud LLM verification
- CLI accepts optional `--llm-base-url` parameter for OpenAI-compatible API base URL
- CLI accepts optional `--llm-model` parameter for the LLM model name
- CLI displays help message with all available options when `--help` is provided
- CLI displays version information when `--version` is provided
- Invalid arguments result in a clear error message with usage instructions

---

### US-CLI-02: Display Real-Time Progress

**As a** Researcher, Student, Content Creator, or Educator
**I want** to see real-time progress updates during slide extraction
**So that** I can monitor the extraction process and estimate completion time

**Acceptance Criteria:**
- Progress bar displays current processing stage (Validating, Downloading, Extracting, Processing, Generating)
- Progress bar shows percentage complete for each stage
- Progress bar displays estimated time remaining
- Current operation is displayed with descriptive text (e.g., "Downloading video...", "Extracting frames...")
- Number of unique slides found is displayed as they are identified
- Total frames processed is displayed
- Progress updates occur at least once per second during active processing
- Progress bar is cleared upon completion or error

---

### US-CLI-03: Display Summary Report

**As a** Researcher, Student, Content Creator, or Educator
**I want** to see a summary report after extraction completes
**So that** I can quickly understand the results and locate output files

**Acceptance Criteria:**
- Summary displays total unique slides extracted
- Summary displays total frames processed
- Summary displays total processing duration
- Summary displays path to generated Markdown file
- Summary displays path to extracted slide images directory
- Summary displays OCR confidence statistics (average, min, max)
- Summary displays any warnings or issues encountered
- Summary is displayed in a clean, readable format
- Summary includes session ID for reference

---

### US-CLI-04: Validate Input Configuration

**As a** Researcher, Student, Content Creator, or Educator
**I want** to receive immediate feedback if my configuration is invalid
**So that** I can correct errors before starting the extraction process

**Acceptance Criteria:**
- Configuration is validated before starting extraction
- Invalid frame intervals (< 0.1 seconds or > 60 seconds) are rejected with clear error message
- Invalid similarity thresholds (< 0.0 or > 1.0) are rejected with clear error message
- Invalid language codes are rejected with list of supported languages
- Non-existent or non-writable output directories are rejected with clear error message
- Validation errors are displayed before any processing begins
- All validation errors are displayed together if multiple issues exist

---

## Session Orchestration Epic

### US-SESSION-01: Create Processing Session

**As a** System
**I want** to create a unique processing session for each extraction request
**So that** I can track the lifecycle of the extraction process and maintain state

**Acceptance Criteria:**
- A unique session ID is generated for each extraction request
- Session stores the YouTube video ID
- Session stores the extraction configuration
- Session stores the initial state as "Created"
- Session records creation timestamp
- Session events are stored in the event store
- Session state can be reconstructed from stored events

---

### US-SESSION-02: Orchestrate Extraction Pipeline

**As a** System
**I want** to coordinate the execution of all extraction steps in the correct order
**So that** slides are extracted efficiently and reliably

**Acceptance Criteria:**
- Pipeline executes steps in order: URL validation → Video download → Frame extraction → Deduplication → OCR → Markdown generation
- Each step is triggered by completion of the previous step
- Session state transitions correctly through: Created → Downloading → Extracting → Processing → Generating → Completed
- Events are published after each step completes
- Failed steps transition session to Failed state with error reason
- Session records completion timestamp when finished
- Session records total duration of extraction

---

### US-SESSION-03: Handle Session Recovery

**As a** Researcher, Student, Content Creator, or Educator
**I want** to be able to resume an interrupted extraction session
**So that** I don't have to restart the entire process if it fails partway through

**Acceptance Criteria:**
- Session state is persisted after each completed step
- Session can be resumed from the last successful step
- User can provide a session ID to resume an interrupted session
- Resumed session continues from the last completed step
- Resumed session uses the same configuration as original
- Resumed session generates the same output as non-interrupted session
- Completed steps are not re-executed when resuming

---

### US-SESSION-04: Clean Up Temporary Resources

**As a** System
**I want** to clean up temporary files and resources after session completion
**So that** disk space is not wasted and the system remains clean

**Acceptance Criteria:**
- Downloaded video file is deleted after frame extraction completes
- Temporary frame files are deleted after deduplication completes
- Only unique slide images are retained in output directory
- Cleanup occurs even if session fails
- Cleanup failures are logged but do not cause session to fail
- User can opt to keep temporary files with a flag
- Cleanup is performed in reverse order of resource creation

---

## Video Context Epic

### US-VIDEO-01: Validate YouTube URL

**As a** Researcher, Student, Content Creator, or Educator
**I want** the system to validate that I've provided a valid YouTube URL
**So that** I don't waste time attempting to process invalid URLs

**Acceptance Criteria:**
- URLs starting with `https://www.youtube.com/` are accepted
- URLs starting with `https://youtu.be/` are accepted
- Video ID is correctly extracted from standard YouTube URL format
- Video ID is correctly extracted from shortened YouTube URL format
- Invalid URLs (non-YouTube domains) are rejected with clear error message
- URLs without valid video IDs are rejected with clear error message
- Validation completes in under 1 second

---

### US-VIDEO-02: Download Video

**As a** System
**I want** to download the YouTube video to a temporary location
**So that** frames can be extracted from the video

**Acceptance Criteria:**
- Video is downloaded to a temporary directory
- Video is downloaded in the highest available quality up to 1080p
- Download progress is reported to the session
- Video duration is captured after download completes
- Video resolution is captured after download completes
- Download is retried up to 3 times with exponential backoff on failure
- Download fails with clear error message if video is unavailable or private
- Download fails with clear error message if video exceeds 4 hours
- Download timeout is set to 30 minutes per hour of video duration

---

### US-VIDEO-03: Verify Video Availability

**As a** System
**I want** to verify that the video is publicly available before downloading
**So that** I can fail fast and provide a clear error message

**Acceptance Criteria:**
- Video availability is checked before download begins
- Private videos are rejected with clear error message
- Age-restricted videos are rejected with clear error message
- Deleted videos are rejected with clear error message
- Region-locked videos are rejected with clear error message
- Video metadata (title, duration, resolution) is retrieved if available
- Availability check completes in under 5 seconds

---

### US-VIDEO-04: Handle Network Timeouts

**As a** System
**I want** to handle network timeouts gracefully during video download
**So that** temporary network issues don't cause permanent failure

**Acceptance Criteria:**
- Network timeout is set to 60 seconds per connection attempt
- Timeout triggers retry with exponential backoff
- Maximum of 3 retry attempts are made
- All retries exhausted results in clear error message with timeout duration
- Partial downloads are cleaned up before retry
- Timeout duration is configurable via environment variable

---

## Frame Context Epic

### US-FRAME-01: Extract Frames at Intervals

**As a** System
**I want** to extract frames from the video at regular intervals
**So that** I can capture all potential slides from the presentation

**Acceptance Criteria:**
- Frames are extracted starting from the beginning of the video
- Frames are extracted at the configured interval (default: 5 seconds)
- Frame number and timestamp are recorded for each extracted frame
- Frames are saved as PNG images in a temporary directory
- Frame extraction progress is reported to the session
- Frames are extracted in order from start to end of video
- Last frame of video is always extracted regardless of interval

---

### US-FRAME-02: Compute Perceptual Hash

**As a** System
**I want** to compute a perceptual hash for each extracted frame
**So that** I can identify similar frames for deduplication

**Acceptance Criteria:**
- Perceptual hash is computed using image hashing algorithm
- Hash is deterministic (same image always produces same hash)
- Hash is resistant to minor image variations (compression artifacts, slight shifts)
- Hash is stored with the frame metadata
- Hash computation completes in under 100ms per frame
- Hash computation failure results in clear error message with frame ID

---

### US-FRAME-03: Handle Frame Extraction Errors

**As a** System
**I want** to handle errors during frame extraction gracefully
**So that** a single corrupt frame doesn't cause the entire extraction to fail

**Acceptance Criteria:**
- Corrupt frames are skipped with warning logged
- Skipped frames do not affect subsequent frame extraction
- Maximum of 10% of frames can be skipped before session fails
- Skipped frame count is included in summary report
- Frame extraction errors include frame number and timestamp in error message

---

### US-FRAME-04: Optimize Frame Storage

**As a** System
**I want** to optimize frame storage to minimize disk usage
**So that** the extraction process doesn't consume excessive disk space

**Acceptance Criteria:**
- Frames are stored with compression
- Temporary frames are deleted as soon as they are no longer needed
- Frame filenames include session ID and frame number for easy identification
- Frame storage directory is cleaned up after deduplication completes
- Maximum disk usage for frames is limited to 10GB

---

## Deduplication Context Epic

### US-DEDUP-01: Identify Unique Slides

**As a** System
**I want** to identify which frames represent unique slides
**So that** duplicate slides are not included in the final output

**Acceptance Criteria:**
- Frames are compared using perceptual hash similarity
- Frames with similarity above the threshold are considered duplicates
- First occurrence of each unique slide is retained
- Subsequent duplicates are discarded
- Similarity threshold is configurable (default: 0.85)
- Uniqueness check is performed in O(n) time using hash set
- Number of unique slides is reported to the session

---

### US-DEDUP-02: Calculate Similarity Score

**As a** System
**I want** to calculate a similarity score between frame hashes
**So that** I can determine if frames represent the same slide

**Acceptance Criteria:**
- Similarity score is between 0.0 (completely different) and 1.0 (identical)
- Similarity calculation is symmetric (similarity(A,B) == similarity(B,A))
- Similarity calculation is deterministic
- Similarity calculation completes in under 1ms per comparison
- Similarity threshold can be adjusted for different use cases

---

### US-DEDUP-03: Preserve Unique Slide Images

**As a** System
**I want** to preserve images of unique slides in the output directory
**So that** users can view the extracted slides

**Acceptance Criteria:**
- Unique slide images are saved to the output directory
- Slide images are named sequentially (slide-001.png, slide-002.png, etc.)
- Slide images are saved in PNG format
- Slide images retain original resolution
- Slide images include metadata in filename if timestamps are enabled
- Slide images are organized in a subdirectory named after the video ID

---

### US-DEDUP-04: Handle No Unique Slides

**As a** System
**I want** to handle the case where no unique slides are found
**So that** the user receives a clear error message

**Acceptance Criteria:**
- If no unique slides are found, session fails with clear error message
- Error message indicates possible reasons (video has no slides, threshold too high)
- Error message suggests lowering similarity threshold
- Temporary files are cleaned up before reporting error
- Session state is set to Failed with reason "NoUniqueSlidesFound"

---

### US-DEDUP-05: Verify Slides with Cloud LLM

**As a** Researcher, Student, Content Creator, or Educator
**I want** to use a Cloud LLM to verify if identified unique frames actually contain slides
**So that** non-slide frames (like speaker-only views) are tagged for review

**Acceptance Criteria:**
- Each identified unique slide is sent to a Cloud LLM (OpenAI compatible API) for verification
- LLM is prompted to identify if the frame contains a presentation slide or just people/faces
- Slides identified as "not a slide" are tagged with `requires_human_review = true`
- LLM verification is only performed if `llm` configuration is provided
- LLM verification failures are logged but do not stop the process

---

### US-DEDUP-06: Confirm Deletion of Non-Slide Frames

**As a** Researcher, Student, Content Creator, or Educator
**I want** to be asked whether to delete frames tagged for human review at the end of the process
**So that** I can easily clean up the output directory

**Acceptance Criteria:**
- After generation completes, if any slides were tagged for human review, the CLI prompts the user
- The prompt asks: "Some slides were tagged as potentially not containing presentation content. Would you like to delete them? (y/N)"
- If user confirms (y/Y), tagged slide images are deleted from the output directory
- If user denies or provides no input, tagged slides are kept for manual review
- Summary report indicates how many slides were deleted or kept for review

---

## OCR Context Epic

### US-OCR-01: Extract Text from Slides

**As a** System
**I want** to extract text content from each unique slide using OCR
**So that** the Markdown output includes searchable text from the slides

**Acceptance Criteria:**
- OCR is performed on each unique slide image
- Extracted text is stored with the slide metadata
- OCR confidence score is calculated and stored
- OCR language is detected and stored
- OCR progress is reported to the session
- OCR completes in under 5 seconds per slide
- OCR failure for a slide is logged but does not stop processing

---

### US-OCR-02: Support Multiple Languages

**As a** Researcher, Student, Content Creator, or Educator
**I want** to specify which languages the OCR should recognize
**So that** text from non-English slides is extracted correctly

**Acceptance Criteria:**
- User can specify one or more OCR languages via command line
- Supported languages include: English, Spanish, French, German, Japanese, Chinese, Korean
- Language codes are validated against supported languages
- OCR uses specified languages in priority order
- Default language is English if not specified
- Language detection can be enabled to auto-detect slide language

---

### US-OCR-03: Filter Low Confidence Results

**As a** System
**I want** to flag OCR results with low confidence
**So that** users are aware of potentially inaccurate text extractions

**Acceptance Criteria:**
- OCR confidence threshold is set to 0.5
- Text with confidence below threshold is flagged in output
- Flagged text includes confidence score in output
- Low confidence does not prevent text from being included
- Confidence statistics are included in summary report
- Confidence threshold is configurable via command line

---

### US-OCR-04: Handle OCR Failures

**As a** System
**I want** to handle OCR failures gracefully
**So that** a single failed OCR doesn't prevent the entire process from completing

**Acceptance Criteria:**
- OCR failures are logged with slide ID and error reason
- Failed OCR does not prevent other slides from being processed
- Failed OCR is indicated in output with placeholder text
- Maximum of 20% of slides can fail OCR before session fails
- OCR failure count is included in summary report
- Common OCR failure reasons are explained to user

---

## Document Context Epic

### US-DOC-01: Generate Markdown Document

**As a** Researcher, Student, Content Creator, or Educator
**I want** to receive a Markdown document containing all extracted slides
**So that** I can easily view, edit, and share the slide content

**Acceptance Criteria:**
- Markdown document is generated in the output directory
- Document filename is based on video title and session ID
- Each slide is represented as a heading with slide number
- Slide images are embedded as Markdown image references
- Extracted text is included below each slide image
- Timestamp is included if timestamps are enabled
- Document includes a title section with video information
- Document includes a summary section with extraction statistics

---

### US-DOC-02: Format Extracted Text

**As a** System
**I want** to format extracted text in a readable way
**So that** the Markdown document is easy to read and understand

**Acceptance Criteria:**
- Text is formatted as Markdown blockquotes under each slide
- Low confidence text is marked with a warning indicator
- Text is cleaned of common OCR artifacts (extra whitespace, random characters)
- Text preserves paragraph structure where detected
- Text preserves list structure where detected
- Empty text extractions are indicated with placeholder

---

### US-DOC-03: Include Video Metadata

**As a** Researcher, Student, Content Creator, or Educator
**I want** the Markdown document to include video metadata
**So that** I can reference the original source

**Acceptance Criteria:**
- Document includes video title as main heading
- Document includes YouTube URL as a link
- Document includes video duration
- Document includes extraction date
- Document includes session ID for reference
- Document includes configuration used for extraction

---

### US-DOC-04: Customize Output Template

**As a** Researcher, Student, Content Creator, or Educator
**I want** to be able to customize the Markdown output template
**So that** the output matches my specific formatting needs

**Acceptance Criteria:**
- User can provide a custom Tera template file
- Default template is used if no custom template is provided
- Template has access to all session, video, and slide data
- Template syntax errors are reported with clear error messages
- Template variables are documented in user guide
- Example custom templates are provided

---

## Error Handling Epic

### US-ERR-01: Display User-Friendly Error Messages

**As a** Researcher, Student, Content Creator, or Educator
**I want** to receive clear, actionable error messages when something goes wrong
**So that** I can understand what went wrong and how to fix it

**Acceptance Criteria:**
- Error messages are written in plain language
- Error messages include the specific problem
- Error messages suggest possible solutions
- Error messages include relevant context (URL, file path, session ID)
- Technical errors are logged separately with full details
- Error messages are consistent in format and tone
- Error messages are displayed in a readable format

---

### US-ERR-02: Validate Output Directory

**As a** System
**I want** to validate that the output directory is writable before processing
**So that** the user is notified early if output cannot be written

**Acceptance Criteria:**
- Output directory is checked for write permissions before processing
- Non-existent directories are created if parent exists
- Non-existent parent directories result in clear error message
- Non-writable directories result in clear error message
- Disk space is checked before processing
- Insufficient disk space results in clear error message with required space

---

### US-ERR-03: Handle Insufficient Memory

**As a** System
**I want** to handle insufficient memory situations gracefully
**So that** the system doesn't crash and provides helpful feedback

**Acceptance Criteria:**
- Memory usage is monitored during processing
- Memory threshold is set to 500MB
- Approaching memory threshold triggers warning
- Exceeding memory threshold results in graceful failure
- Error message indicates memory requirement and available memory
- User is suggested to process shorter videos or reduce frame interval

---

### US-ERR-04: Handle External Dependency Failures

**As a** System
**I want** to handle failures of external dependencies (yt-dlp, FFmpeg, Tesseract)
**So that** the user is informed about missing or failed dependencies

**Acceptance Criteria:**
- External dependencies are checked before processing
- Missing dependencies result in clear error message with installation instructions
- Dependency execution failures are logged with full error output
- Dependency failures result in clear error message indicating which dependency failed
- User is provided with troubleshooting steps for dependency issues
- Dependency versions are logged for debugging

---

### US-ERR-05: Validate Configuration

**As a** System
**I want** to validate all configuration parameters before processing
**So that** invalid configuration doesn't cause unexpected behavior

**Acceptance Criteria:**
- All configuration parameters are validated before processing
- Invalid parameters result in clear error message with valid range
- Conflicting parameters are detected and reported
- Default values are used for optional parameters not provided
- Configuration validation occurs before any external operations
- All validation errors are reported together

---

### US-ERR-06: Log Technical Errors

**As a** System
**I want** to log technical errors with full details for debugging
**So that** developers can diagnose and fix issues

**Acceptance Criteria:**
- Technical errors are logged with full stack traces
- Logs include session ID, timestamp, and error context
- Logs are written to a file in the output directory
- Log level is configurable (error, warn, info, debug, trace)
- Log rotation is implemented to prevent excessive log files
- Logs include system information (OS, version, dependencies)

---

## Appendix: Actors

| Actor | Description |
|-------|-------------|
| **Researcher** | Extracts slides for analysis and research purposes |
| **Student** | Captures lecture content for study and review |
| **Content Creator** | Extracts source material for creating new content |
| **Educator** | Prepares teaching materials from educational videos |

---

## Appendix: Bounded Contexts

| Context | Responsibility |
|---------|----------------|
| **CLI Interface** | User interaction, command parsing, progress display |
| **Session** | Orchestration, command handlers, event store, state projections |
| **Video** | YouTube interaction, URL validation, downloading |
| **Frame** | Frame extraction, hash calculation |
| **Deduplication** | Slide identification, uniqueness checking |
| **OCR** | Text recognition, language detection |
| **Document** | Output formatting, Markdown generation |

---

## Appendix: Success Metrics

**Functional Requirements:**
- Extract unique slides from YouTube videos (95% accuracy)
- Generate Markdown with embedded images
- Extract text via OCR (80%+ accuracy on clear text)
- Handle videos up to 4 hours length

**Quality Metrics:**
- Test coverage > 80%
- Zero unsafe code in domain layer
- All property tests passing

**Performance Metrics:**
- < 5 minutes processing for 30-minute video
- Memory usage < 500MB during processing
- Zero-copy operations where possible
