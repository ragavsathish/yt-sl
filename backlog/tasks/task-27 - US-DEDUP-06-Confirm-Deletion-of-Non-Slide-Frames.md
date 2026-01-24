---
id: TASK-27
title: US-DEDUP-06 Confirm Deletion of Non-Slide Frames
status: Completed
assignee: [opencode]
created_date: '2026-01-15 22:02'
updated_date: '2026-01-24 19:54'
labels: []
dependencies: [TASK-26]
ordinal: 3000
---

**As a** Researcher, Student, Content Creator, or Educator
**I want** to be asked whether to delete frames tagged for human review at the end of the process
**So that** I can easily clean up the output directory

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 After generation completes, if any slides were tagged for human review, the CLI prompts the user
- [x] #2 The prompt asks: "Some slides were tagged as potentially not containing presentation content. Would you like to delete them? (y/N)"
- [x] #3 If user confirms (y/Y), tagged slide images are deleted from the output directory
- [x] #4 If user denies or provides no input, tagged slides are kept for manual review
- [x] #5 Summary report indicates how many slides were deleted or kept for review
- [x] #6 Generate dual reports: `report.md` (full with warnings) and `report_cleaned.md` (verified slides only)

---
<!-- AC:END -->
