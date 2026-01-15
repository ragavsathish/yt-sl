---
id: TASK-25
title: US-DEDUP-04 Handle No Unique Slides
status: Done
assignee: []
created_date: '2026-01-15 22:02'
updated_date: '2026-01-15 20:19'
labels: []
dependencies: []
---

**As a** System
**I want** to handle the case where no unique slides are found
**So that** the user receives a clear error message

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 If no unique slides are found, session fails with clear error message
- [ ] #2 Error message indicates possible reasons (video has no slides, threshold too high)
- [ ] #3 Error message suggests lowering similarity threshold
- [ ] #4 Temporary files are cleaned up before reporting error
- [ ] #5 Session state is set to Failed with reason "NoUniqueSlidesFound"

---
<!-- AC:END -->
