---
id: TASK-14
title: US-VIDEO-03 Verify Video Availability
status: Done
assignee: []
created_date: '2026-01-15 22:02'
updated_date: '2026-01-15 20:19'
labels: []
dependencies: []
---

**As a** System
**I want** to verify that the video is publicly available before downloading
**So that** I can fail fast and provide a clear error message

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Video availability is checked before download begins
- [ ] #2 Private videos are rejected with clear error message
- [ ] #3 Age-restricted videos are rejected with clear error message
- [ ] #4 Deleted videos are rejected with clear error message
- [ ] #5 Region-locked videos are rejected with clear error message
- [ ] #6 Video metadata (title, duration, resolution) is retrieved if available
- [ ] #7 Availability check completes in under 5 seconds

---
<!-- AC:END -->
