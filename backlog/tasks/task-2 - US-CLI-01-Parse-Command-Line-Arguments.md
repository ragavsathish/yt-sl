---
id: TASK-2
title: US-CLI-01 Parse Command Line Arguments
status: Done
assignee: []
created_date: '2026-01-15 22:02'
updated_date: '2026-01-15 20:19'
labels: []
dependencies: []
---

**As a** Researcher, Student, Content Creator, or Educator
**I want** to provide a YouTube URL and optional configuration parameters via command line arguments
**So that** I can customize the slide extraction process for my specific needs

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 CLI accepts a required YouTube URL argument
- [ ] #2 CLI accepts optional `--interval` parameter for frame extraction interval (default: 5 seconds)
- [ ] #3 CLI accepts optional `--threshold` parameter for slide similarity threshold (default: 0.85)
- [ ] #4 CLI accepts optional `--output` parameter for output directory (default: current directory)
- [ ] #5 CLI accepts optional `--languages` parameter for OCR languages (default: English)
- [ ] #6 CLI accepts optional `--timestamps` flag to include timestamps in output
- [ ] #7 CLI accepts optional `--llm-api-key` parameter for Cloud LLM verification
- [ ] #8 CLI accepts optional `--llm-base-url` parameter for OpenAI-compatible API base URL
- [ ] #9 CLI accepts optional `--llm-model` parameter for the LLM model name
- [ ] #10 CLI displays help message with all available options when `--help` is provided
- [ ] #11 CLI displays version information when `--version` is provided
- [ ] #12 Invalid arguments result in a clear error message with usage instructions

---
<!-- AC:END -->
