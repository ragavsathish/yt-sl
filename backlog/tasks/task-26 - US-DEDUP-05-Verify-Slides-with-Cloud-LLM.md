---
id: TASK-26
title: US-DEDUP-05 Verify Slides with Cloud LLM
status: Done
assignee: []
created_date: '2026-01-15 22:02'
updated_date: '2026-01-24 21:04'
labels: []
dependencies: []
ordinal: 44000
---

**As a** Researcher, Student, Content Creator, or Educator
**I want** to use a Cloud LLM to verify if identified unique frames actually contain slides
**So that** non-slide frames (like speaker-only views) are tagged for review

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Each identified unique slide is sent to a Cloud LLM (OpenAI compatible API) for verification
- [x] #2 LLM is prompted to identify if the frame contains a presentation slide or just people/faces
- [x] #3 Slides identified as "not a slide" are tagged with `requires_human_review = true`
- [x] #4 LLM verification is only performed if `llm` configuration is provided
- [x] #5 LLM verification failures are logged but do not stop the process

---
<!-- AC:END -->
