
import re
import os
import datetime

SOURCE_FILE = 'plans/user_stories.md'
DEST_DIR = 'backlog/tasks'

def parse_user_stories(file_path):
    with open(file_path, 'r') as f:
        content = f.read()

    epics = []
    current_epic = None

    # Split by lines to process line by line
    lines = content.split('\n')

    i = 0
    while i < len(lines):
        line = lines[i]

        # New Epic detection
        # ## CLI Interface Epic
        epic_match = re.match(r'^##\s+(.*)', line)
        if epic_match and 'Table of Contents' not in line and 'Appendix' not in line:
            current_epic = {
                'name': epic_match.group(1).strip(),
                'stories': []
            }
            epics.append(current_epic)

        # New Story detection
        # ### US-CLI-01: Parse Command Line Arguments
        story_match = re.match(r'^###\s+((US-[\w-]+):\s+(.*))', line)
        if story_match and current_epic:
            story = {
                'id_code': story_match.group(2).strip(),
                'title': story_match.group(3).strip(),
                'content': []
            }
            # Capture story content until next header or end
            i += 1
            while i < len(lines):
                next_line = lines[i]
                if next_line.startswith('##') or next_line.startswith('###'):
                    i -= 1 # Backtrack
                    break
                story['content'].append(next_line)
                i += 1
            current_epic['stories'].append(story)

        i += 1

    return epics

def create_task_file(task_id, title, content, status='To Do', parent=None, is_epic=False):
    # Backlog.md seems to slugify titles in filenames, but let's try keep it simple
    # Format: task-{id} - {title}.md
    filename_title = re.sub(r'[\\/*?:"<>|]', "", title)
    filename = f"task-{task_id} - {filename_title}.md"
    filepath = os.path.join(DEST_DIR, filename)

    created_date = datetime.datetime.now().strftime("%Y-%m-%d %H:%M")

    with open(filepath, 'w') as f:
        f.write("---\n")
        f.write(f"id: TASK-{task_id}\n")
        # Escape title if needed, but yaml safe
        f.write(f"title: {title}\n")
        f.write(f"status: {status}\n")
        f.write("assignee: []\n")
        f.write(f"created_date: '{created_date}'\n")

        if is_epic:
             f.write("labels: ['Epic']\n")
        else:
             f.write("labels: []\n")

        if parent:
            # Parent might need to be the ID string e.g. TASK-1
            f.write(f"parent: TASK-{parent}\n")

        f.write("dependencies: []\n")
        f.write("---\n\n")
        f.write(content)

    return task_id

def main():
    if not os.path.exists(DEST_DIR):
        os.makedirs(DEST_DIR)

    # Clean existing tasks to avoid duplicates
    # Be careful not to delete things we didn't create
    # But for migration we can assume we want a fresh start of tasks
    for f in os.listdir(DEST_DIR):
        if f.startswith("task-") and f.endswith(".md"):
             os.remove(os.path.join(DEST_DIR, f))

    epics = parse_user_stories(SOURCE_FILE)

    task_counter = 1

    for epic in epics:
        epic_id = task_counter
        create_task_file(
            task_id=epic_id,
            title=epic['name'],
            content=f"Parent Epic for related user stories.",
            is_epic=True
        )
        print(f"Created Epic: {epic['name']} (ID: {epic_id})")
        task_counter += 1

        for story in epic['stories']:
            story_id = task_counter
            # Clean up content (remove leading/trailing newlines)
            story_text = '\n'.join(story['content']).strip()

            # Remove the ### Title line if present (it shouldn't be based on parsing logic but just in case)

            # Convert "Acceptance Criteria:" section lists to checklist
            # Look for lines starting with "- " after "**Acceptance Criteria:**"
            processed_lines = []
            in_ac_section = False
            for line in story_text.split('\n'):
                if "**Acceptance Criteria:**" in line:
                    in_ac_section = True
                    processed_lines.append("## Acceptance Criteria") # Use H2 for Backlog readability
                    continue

                if in_ac_section and line.strip().startswith("- "):
                    # Convert "- Item" to "- [ ] Item"
                    processed_lines.append(line.replace("- ", "- [ ] ", 1))
                else:
                    processed_lines.append(line)

            story_text = '\n'.join(processed_lines)

            # Format title to include the US code for easy reference
            full_title = f"{story['id_code']} {story['title']}"

            create_task_file(
                task_id=story_id,
                title=full_title,
                content=story_text,
                parent=epic_id,
                status='To Do'
            )
            print(f"  Created Story: {full_title} (ID: {story_id})")
            task_counter += 1

if __name__ == '__main__':
    main()
