---
id: TASK-8
title: US-SESSION-02 Orchestrate Extraction Pipeline
status: Done
assignee: []
created_date: '2026-01-15 22:02'
updated_date: '2026-01-24 21:04'
labels: []
dependencies: []
ordinal: 26000
---

**As a** System
**I want** to coordinate the execution of all extraction steps in the correct order
**So that** slides are extracted efficiently and reliably

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Pipeline executes steps in order: URL validation → Video download → Frame extraction → Deduplication → OCR → Markdown generation
- [x] #2 Each step is triggered by completion of the previous step
- [x] #3 Session state transitions correctly through: Created → Downloading → Extracting → Processing → Generating → Completed
- [x] #4 Events are published after each step completes
- [x] #5 Failed steps transition session to Failed state with error reason
- [x] #6 Session records completion timestamp when finished
- [x] #7 Session records total duration of extraction

---
<!-- AC:END -->
