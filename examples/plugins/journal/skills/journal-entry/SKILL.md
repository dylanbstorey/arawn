---
name: journal-entry
description: Create a guided daily journal entry
user_invocable: true
arguments:
  - name: date
    description: Date for the entry (YYYY-MM-DD format, defaults to today)
    required: false
---

# Daily Journal Entry for {date}

Guide the user through creating a structured journal entry.

## Steps

1. **Ask about mood**: How are you feeling today? (1-10 scale or descriptive words)

2. **Reflect on accomplishments**: What did you accomplish today? What went well?

3. **Set tomorrow's goals**: What do you want to focus on tomorrow? Any important tasks?

4. **Additional notes**: Any other thoughts, observations, or things you want to remember?

5. **Tags**: Suggest relevant tags based on the content (work, personal, health, learning, etc.)

## Creating the Entry

Once you have gathered the information, use the `journal` tool to create the entry:

```json
{
  "action": "create",
  "date": "{date}",
  "mood": "<mood from step 1>",
  "accomplishments": "<accomplishments from step 2>",
  "goals": "<goals from step 3>",
  "notes": "<notes from step 4>",
  "tags": ["<suggested tags>"]
}
```

Be encouraging and supportive. Journaling is a practice — the goal is reflection, not perfection.
