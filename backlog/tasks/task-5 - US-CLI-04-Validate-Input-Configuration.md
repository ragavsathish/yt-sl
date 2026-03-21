---
id: TASK-5
title: US-CLI-04 Validate Input Configuration
status: Done
assignee: []
created_date: '2026-01-15 22:02'
updated_date: '2026-01-24 21:04'
labels: []
dependencies: []
ordinal: 37000
---

**As a** Researcher, Student, Content Creator, or Educator
**I want** to receive immediate feedback if my configuration is invalid
**So that** I can correct errors before starting the extraction process

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Configuration is validated before starting extraction
- [x] #2 Invalid frame intervals (< 0.1 seconds or > 60 seconds) are rejected with clear error message
- [x] #3 Invalid similarity thresholds (< 0.0 or > 1.0) are rejected with clear error message
- [x] #4 Invalid language codes are rejected with list of supported languages
- [x] #5 Non-existent or non-writable output directories are rejected with clear error message
- [x] #6 Validation errors are displayed before any processing begins
- [x] #7 All validation errors are displayed together if multiple issues exist

---
<!-- AC:END -->
