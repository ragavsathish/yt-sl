---
id: TASK-13
title: US-VIDEO-02 Download Video
status: Done
assignee: []
created_date: '2026-01-15 22:02'
updated_date: '2026-01-24 21:04'
labels: []
dependencies: []
ordinal: 28000
---

**As a** System
**I want** to download the YouTube video to a temporary location
**So that** frames can be extracted from the video

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Video is downloaded to a temporary directory (cache dir /tmp/yt-sl-cache)
- [x] #2 Video is downloaded in the highest available quality up to 1080p
- [x] #3 Download progress is reported to the session
- [x] #4 Video duration is captured after download completes (via metadata fetch)
- [x] #5 Video resolution is captured after download completes (via metadata fetch)
- [x] #6 Download is retried up to 3 times with exponential backoff on failure
- [x] #7 Download fails with clear error message if video is unavailable or private
- [x] #8 Download fails with clear error message if video exceeds 4 hours (via availability checker)
- [x] #9 Download timeout is set to 30 minutes per hour of video duration

---
<!-- AC:END -->
