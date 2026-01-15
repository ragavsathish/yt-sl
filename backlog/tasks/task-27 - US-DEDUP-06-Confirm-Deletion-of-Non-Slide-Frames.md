---
id: TASK-27
title: US-DEDUP-06 Confirm Deletion of Non-Slide Frames
status: To Do
assignee: []
created_date: '2026-01-15 22:02'
updated_date: '2026-01-15 20:22'
labels: []
dependencies: []
ordinal: 1000
---

**As a** Researcher, Student, Content Creator, or Educator
**I want** to be asked whether to delete frames tagged for human review at the end of the process
**So that** I can easily clean up the output directory

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 After generation completes, if any slides were tagged for human review, the CLI prompts the user
- [ ] #2 The prompt asks: "Some slides were tagged as potentially not containing presentation content. Would you like to delete them? (y/N)"
- [ ] #3 If user confirms (y/Y), tagged slide images are deleted from the output directory
- [ ] #4 If user denies or provides no input, tagged slides are kept for manual review
- [ ] #5 Summary report indicates how many slides were deleted or kept for review

---
<!-- AC:END -->
