---
id: TASK-47
title: 'Optimization: Skip Video Download and Audio Extraction if Files Exist'
status: To Do
assignee: []
created_date: '2026-01-24 22:27'
labels:
  - optimization
  - video
  - transcription
dependencies: []
priority: medium
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Optimize the workflow to check for existing video and audio files before initiating download or extraction. If the target video file or extracted audio file already exists in the expected location, skip the respective operation to save time and bandwidth.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Verify if video file exists for the given YouTube ID before downloading
- [x] #2 Skip yt-dlp execution if valid video file is found
- [x] #3 Verify if extracted audio file exists before running extraction command
- [x] #4 Skip FFmpeg audio extraction if valid audio file is found
- [x] #5 Ensure logging indicates when steps are skipped due to existing files
<!-- AC:END -->
