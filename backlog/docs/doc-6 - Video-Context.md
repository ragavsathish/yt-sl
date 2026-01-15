---
id: doc-6
title: Video-Context
type: specification
created_date: '2026-01-15 22:15'
---
# Video Context

> Handles YouTube video URL validation, metadata fetching, and video file downloading with retry logic.

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

The Video Context is responsible for:

- **Validating** YouTube video URLs and extracting video IDs
- **Fetching** video metadata (title, duration, resolution)
- **Downloading** video files to local storage
- **Managing** download retries with exponential backoff
- **Cleaning up** downloaded files after processing

---

## Domain Types

### YouTubeVideo

Represents a YouTube video.

**Fields:**
- id - Unique video identifier
- url - Original YouTube URL
- title - Video title from metadata
- duration - Video duration
- resolution - Video resolution (width x height)
- download_path - Optional path to downloaded file

### VideoFile

Represents a downloaded video file.

**Fields:**
- video_id - Associated video identifier
- path - File system path
- format - Video format (MP4/WebM/MKV)
- size_bytes - File size in bytes
- duration - Video duration

### VideoURL

Validated YouTube URL wrapper.

**Methods:**
- `new(url)` - Validates URL format
- `extract_video_id()` - Extracts video ID from URL

**Supported URL Formats:**
- `https://www.youtube.com/watch?v=VIDEO_ID`
- `https://youtu.be/VIDEO_ID`

---

## Handlers

### Validate URL

Validates YouTube URL format and extracts video ID.

**Input:** YouTube URL string

**Output:** (video_id, video_url) tuple

**Validation:**
- URL must start with `youtube.com` or `youtu.be`
- Video ID must be extractable from URL

**Error:** InvalidUrl for malformed URLs

---

### Fetch Metadata

Fetches video metadata from YouTube.

**Input:** video_id

**Output:** VideoMetadata
- video_id - Video identifier
- title - Video title
- duration - Video duration
- resolution - Video resolution

**Error:** MetadataFetchFailed for metadata fetch errors

---

### Download Video

Downloads video file from YouTube.

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
- Downloads to configured output directory

---

## Policies

### Maximum Video Duration

Videos must not exceed 4 hours.

**Limit:** 4 hours (14400 seconds)

**Error:** InvalidConfig if duration exceeds limit

### Minimum Video Resolution

Videos must be at least 720p.

**Minimum:** 1280x720 (720p)

**Rationale:** Lower resolutions may not have readable slide text

**Error:** InvalidConfig if resolution below minimum

### Download Retry Policy

Downloads retry with exponential backoff.

**Maximum Retries:** 3 attempts

**Retry Delays:**
- Attempt 1: 2 seconds
- Attempt 2: 4 seconds
- Attempt 3: 8 seconds

**Error:** DownloadFailed if all retries exhausted

---

## External Dependencies

### VideoDownloader

Downloads video from YouTube.

**Signature:**
- `download(&video_id) -> Result<(path, duration), error>`

**Implementation:** YtDlpDownloader
- Uses yt-dlp command-line tool
- Downloads best quality MP4
- Returns error on download failure

### MetadataFetcher

Fetches video metadata.

**Signature:**
- `fetch(&video_id) -> Result<VideoMetadata, error>`

**Implementation:** YtDlpMetadataFetcher
- Uses yt-dlp to fetch metadata
- Extracts title, duration, resolution
- Returns error on fetch failure

---

## Testing

### Unit Tests

Test URL validation:
- Valid YouTube.com URLs should pass
- Valid youtu.be URLs should pass
- Invalid URLs should return InvalidUrl error
- Extract video ID correctly from both URL formats

Test policy validation:
- Videos under 4 hours should pass duration check
- Videos over 4 hours should fail duration check
- Videos 720p+ should pass resolution check
- Videos below 720p should fail resolution check

### Integration Tests

Test download flow:
- Download video from YouTube
- Verify file exists at returned path
- Verify duration is correct
- Test retry logic with mock failures

Test metadata fetch:
- Fetch metadata for real video
- Verify title, duration, resolution

### Mock Dependencies

**MockVideoDownloader:**
- Simulates download delays
- Can simulate download failures for retry testing
- Returns mock file paths and durations

---

## Implementation Notes

### yt-dlp Integration

Uses yt-dlp command-line tool:

**Required Flags:**
- `--format best[ext=mp4]` - Download best quality MP4
- `--output <path>` - Output file path
- Video URL as argument

**Error Handling:**
- Check exit code for failures
- Parse error messages for user feedback

### Metadata Extraction

Extracts metadata using yt-dlp:

**Fields Extracted:**
- Video title
- Duration (from ffprobe)
- Resolution (from ffprobe)
- File size

### Temporary File Management

Downloaded files are stored in:
- Temporary directory during processing
- Configured output directory for user access

Cleanup:
- Temporary files deleted on completion
- Output files preserved for user
- Cleanup runs even on failure
