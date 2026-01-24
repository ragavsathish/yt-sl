import os
import re
import yaml
from collections import defaultdict
from typing import Any, Dict, List, Optional
from pathlib import Path

# Paths
SCRIPT_DIR = Path(__file__).parent.resolve()
BACKLOG_DIR = SCRIPT_DIR / "backlog/tasks"
REQUIREMENTS_DIR = SCRIPT_DIR / "requirements"
FEATURES_DIR = REQUIREMENTS_DIR / "features"

# Mappings
STATUS_MAP = {
    "Done": "implemented",
    "In Progress": "in_progress",
    "To Do": "proposed",
}

FEATURE_MAP = {
    "CLI": "CLI Interface",
    "VIDEO": "Video Management",
    "FRAME": "Frame Extraction",
    "DEDUP": "Deduplication",
    "OCR": "OCR Processing",
    "DOC": "Document Generation",
    "ERR": "Error Handling",
    "SESSION": "Session Orchestration",
}


def parse_markdown_file(file_path: Path) -> Dict[str, Any]:
    """Parse a markdown file with YAML frontmatter."""
    content = file_path.read_text()

    # Split frontmatter
    parts = content.split("---", 2)
    if len(parts) < 3:
        return {}

    frontmatter = yaml.safe_load(parts[1])
    body = parts[2].strip()

    return {
        "frontmatter": frontmatter,
        "body": body
    }

def extract_ac(body: str) -> List[str]:
    """Extract acceptance criteria checklists."""
    acs = []
    # Look for the Acceptance Criteria section
    ac_match = re.search(r"## Acceptance Criteria(.*?)(?:---|<!-- AC:END -->|$)", body, re.DOTALL)
    if ac_match:
        ac_section = ac_match.group(1)
        # Find unchecked or checked items
        # Matches: - [ ] #1 Some text
        matches = re.findall(r"- \[[ x]\] (?:#\d+ )?(.*)", ac_section)
        acs = [m.strip() for m in matches]
    return acs

def extract_story_narrative(body: str) -> Dict[str, str]:
    """Extract As a/I want/So that."""
    narrative = {"as_a": "", "i_want": "Start", "so_that": ""}

    as_a = re.search(r"\*\*As a\*\* (.*)", body)
    i_want = re.search(r"\*\*I want\*\* (.*)", body)
    so_that = re.search(r"\*\*So that\*\* (.*)", body)

    if as_a: narrative["as_a"] = as_a.group(1).strip()
    if i_want: narrative["i_want"] = i_want.group(1).strip()
    if so_that: narrative["so_that"] = so_that.group(1).strip()

    return narrative

def main():
    print(f"Reading from {BACKLOG_DIR.absolute()}")

    if not BACKLOG_DIR.exists():
        print(f"Error: {BACKLOG_DIR} does not exist.")
        return

    # Use default loader/dumper to avoid PyYAML tags
    yaml.SafeDumper.add_representer(
        type(None),
        lambda dumper, value: dumper.represent_scalar(u'tag:yaml.org,2002:null', '')
    )

    features: Dict[str, List[Dict]] = defaultdict(list)
    epics: List[Dict] = []

    # 1. Parse all files
    for file_path in BACKLOG_DIR.glob("*.md"):
        data = parse_markdown_file(file_path)
        if not data:
            continue

        fm = data["frontmatter"]
        title = fm.get("title", "")

        # Check if Epic
        if "Epic" in title or "Epic" in fm.get("labels", []):
            epic_id = f"EP-{len(epics) + 1:03d}"
            epics.append({
                "id": epic_id,
                "title": title.replace(" Epic", ""),
                "status": STATUS_MAP.get(fm.get("status"), "unknown"),
                "phases": ["phase_1"],
                "features": [], # To be filled
                "note": f"Imported from {file_path.name}"
            })
            print(f"Found Epic: {title} -> {epic_id}")
            continue

        # Check if User Story (US-XXX-YYY)
        us_match = re.match(r"(US-([A-Z]+)-\d+)\s+(.*)", title)
        if us_match:
            us_id = us_match.group(1)
            group_code = us_match.group(2) # e.g., VIDEO
            story_title = us_match.group(3)

            narrative = extract_story_narrative(data["body"])
            acs = extract_ac(data["body"])

            story = {
                "id": us_id,
                "as_a": narrative["as_a"],
                "i_want": narrative["i_want"],
                "so_that": narrative["so_that"],
                "acceptance_criteria": acs,
                "priority": "medium",
                "story_quality": "unknown",
                "status": STATUS_MAP.get(fm.get("status"), "unknown"),
                "note": story_title
            }

            features[group_code].append(story)
            print(f"Found Story: {us_id} -> Group {group_code}")

    # 2. Create Directory Structure
    FEATURES_DIR.mkdir(parents=True, exist_ok=True)

    feature_list = []

    # 3. Write Feature Files
    for i, (group_code, stories) in enumerate(features.items(), start=1):
        ft_id = f"FT-{i:03d}"
        ft_title = FEATURE_MAP.get(group_code, f"{group_code} Features")

        # Calculate stats for summary
        quality_summary = {"core": 0, "acceptable": 0, "weak": 0} # All unknown currently

        feature_obj = {
            "id": ft_id,
            "title": ft_title,
            "epic_id": "EP-001",
            "phase": "phase_1",
            "priority": "medium",
            "status": "in_progress",
            "description": f"Features related to {ft_title} (formerly {group_code})",
            "business_value": "Migration from backlog",
            "user_stories": sorted(stories, key=lambda x: x['id']),
        }

        output_path = FEATURES_DIR / f"{ft_id}.yaml"
        with open(output_path, "w") as f:
            yaml.safe_dump(feature_obj, f, sort_keys=False, width=1000)

        print(f"Created {output_path} ({group_code})")

        feature_list.append({
            "id": ft_id,
            "title": ft_title,
            "phase": "phase_1",
            "epic": "EP-001",
            "status": "in_progress"
        })

    # 4. Write Index File
    index_obj = {
        "project": {
            "name": "YouTube Slide Extractor",
            "description": "Extract slides from educational videos",
            "scope": "MVP"
        },
        "phases": {
            "phase_1": {
                "description": "Initial Migration",
                "features": [f["id"] for f in feature_list]
            }
        },
        "epics": [
            {
                "id": "EP-001",
                "title": "Core Functionality",
                "status": "in_progress",
                "phases": ["phase_1"],
                "features": [f["id"] for f in feature_list],
                "note": "Consolidated epic for migration"
            }
        ],
        "features": feature_list
    }

    with open(REQUIREMENTS_DIR / "_index.yaml", "w") as f:
        yaml.safe_dump(index_obj, f, sort_keys=False)

    print("Created _index.yaml")

if __name__ == "__main__":
    main()
