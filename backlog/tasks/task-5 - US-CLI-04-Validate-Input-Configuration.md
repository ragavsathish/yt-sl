---
id: TASK-5
title: US-CLI-04 Validate Input Configuration
status: Done
assignee: []
created_date: '2026-01-15 22:02'
updated_date: '2026-01-15 20:07'
labels: []
dependencies: []
---

**As a** Researcher, Student, Content Creator, or Educator
**I want** to receive immediate feedback if my configuration is invalid
**So that** I can correct errors before starting the extraction process

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Configuration is validated before starting extraction
- [ ] #2 Invalid frame intervals (< 0.1 seconds or > 60 seconds) are rejected with clear error message
- [ ] #3 Invalid similarity thresholds (< 0.0 or > 1.0) are rejected with clear error message
- [ ] #4 Invalid language codes are rejected with list of supported languages
- [ ] #5 Non-existent or non-writable output directories are rejected with clear error message
- [ ] #6 Validation errors are displayed before any processing begins
- [ ] #7 All validation errors are displayed together if multiple issues exist

---
<!-- AC:END -->
