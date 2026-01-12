Implement Wave 3 Frame Context stories (US-FRAME-01 through US-FRAME-04) for the YouTube Video Slide Extractor project.

**Context:**
- This is a Rust project located at /Users/sathish.narayanan/Documents/git/yt-sl-ci
- Wave 1 (Error Handling, CLI basics, Session creation) has been completed
- Wave 2 (Video Context) has been completed, including video download functionality
- Error handling is available in `poc/src/shared/domain/error.rs`
- Logging infrastructure is available in `poc/src/shared/infrastructure/logging.rs`
- Memory monitoring is available in `poc/src/shared/infrastructure/memory.rs`
- Dependency analysis document (plans/dependency_analysis.md) provides detailed requirements

**User Stories to Implement:**

1. **US-FRAME-01: Extract Frames at Intervals** - Frame extraction from video
2. **US-FRAME-02: Compute Perceptual Hash** - Perceptual hash computation
3. **US-FRAME-03: Handle Frame Extraction Errors** - Frame extraction error handling
4. **US-FRAME-04: Optimize Frame Storage** - Frame storage optimization

**Requirements:**

Based on the user stories in plans/user_stories.md, implement the following:

**US-FRAME-01: Extract Frames at Intervals**
- Extract frames from downloaded video at configurable intervals
- Use FFmpeg for frame extraction
- Store frames with metadata (timestamp, frame number)
- Track extraction progress
- Handle memory constraints during extraction
- Support different output formats (JPEG, PNG)

**US-FRAME-02: Compute Perceptual Hash**
- Compute perceptual hash for each frame (e.g., using image_hash crate)
- Use ahash algorithm for efficient similarity comparison
- Store hash with frame metadata
- Support multiple hash algorithms (average, difference, dhash)
- Provide hash comparison functions

**US-FRAME-03: Handle Frame Extraction Errors**
- Detect corrupt or invalid frames
- Skip frames that cannot be extracted
- Log errors for problematic frames
- Continue extraction despite individual frame failures
- Provide summary of extraction success/failure

**US-FRAME-04: Optimize Frame Storage**
- Compress frames to reduce disk usage
- Use efficient image formats (JPEG with quality settings)
- Implement frame caching for faster access
- Clean up temporary frames after processing
- Optimize storage for large numbers of frames

**Implementation Guidelines:**

1. Create a new frame context module structure:
   - `poc/src/contexts/frame/mod.rs` - Module exports
   - `poc/src/contexts/frame/domain/commands.rs` - Frame extraction commands
   - `poc/src/contexts/frame/domain/events.rs` - Frame-related events
   - `poc/src/contexts/frame/domain/handlers.rs` - Frame extraction handlers
   - `poc/src/contexts/frame/domain/state.rs` - Frame state management
   - `poc/src/contexts/frame/infrastructure/extractor.rs` - FFmpeg frame extraction
   - `poc/src/contexts/frame/infrastructure/perceptual_hash.rs` - Hash computation
   - `poc/src/contexts/frame/infrastructure/frame_storage.rs` - Frame storage optimization

2. Update `poc/src/contexts/mod.rs` to export the new frame context

3. Update `poc/Cargo.toml` to add necessary dependencies:
   - `image` crate for image processing
   - `imageproc` or `image_hash` for perceptual hashing
   - `ffmpeg-next` or similar for FFmpeg integration

4. Use existing error types from `poc/src/shared/domain/error.rs`

5. Use existing logging infrastructure from `poc/src/shared/infrastructure/logging.rs`

6. Use existing memory monitoring from `poc/src/shared/infrastructure/memory.rs`

7. Write unit tests for:
   - Frame extraction at intervals
   - Perceptual hash computation
   - Error handling for corrupt frames
   - Frame storage optimization

**Important Notes:**
- US-FRAME-01 depends on US-VIDEO-02 (completed in Wave 2)
- US-FRAME-02 depends on US-FRAME-01
- US-FRAME-03 and US-FRAME-04 have no hard dependencies
- US-FRAME-03 and US-FRAME-04 can be done in parallel with US-FRAME-01
- Use FFmpeg for frame extraction (external dependency)
- Follow Rust async/await patterns for long-running operations
- Consider memory efficiency when processing many frames

**Scope:**
- ONLY implement the 4 Frame Context stories listed above (US-FRAME-01 through US-FRAME-04)
- DO NOT implement any other user stories
- DO NOT deviate from the requirements specified
- Signal completion by using attempt_completion with a concise summary of what was implemented
