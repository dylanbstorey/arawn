# Journal Plugin

A personal journaling and note-taking plugin for Arawn demonstrating all plugin component types. This plugin uses the Claude Code plugin format for maximum compatibility.

## Features

- **CLI Tool**: `journal` - Create, list, search, and tag journal entries
- **Skills**: `/journal-entry` and `/journal-review` for guided journaling
- **Hook**: Session-end reminder to journal if you haven't today
- **Agent**: `journal-assistant` focused on reflection tasks

## Installation

### Subscribe via CLI

```bash
# From GitHub (if published)
arawn plugin add owner/journal-plugin

# From local path
arawn plugin add --project ./examples/plugins/journal
```

### Manual Installation

Copy this directory to your plugins folder:

```bash
# User plugins
cp -r examples/plugins/journal ~/.config/arawn/plugins/

# Or project plugins
cp -r examples/plugins/journal ./plugins/
```

## Usage

### Creating a Journal Entry

Use the `/journal-entry` skill for a guided experience:

```
/journal-entry
```

Or specify a date:

```
/journal-entry 2024-01-15
```

### Reviewing Entries

Review your entries for a time period:

```
/journal-review week
/journal-review month
```

### Direct Tool Usage

The `journal` tool supports these actions:

**Create an entry:**
```json
{
  "action": "create",
  "date": "2024-01-15",
  "mood": "good",
  "accomplishments": "Finished the plugin system",
  "goals": "Write documentation",
  "notes": "Productive day overall",
  "tags": ["work", "coding"]
}
```

**List entries:**
```json
{
  "action": "list",
  "start_date": "2024-01-01",
  "end_date": "2024-01-31",
  "limit": 10
}
```

**Search entries:**
```json
{
  "action": "search",
  "query": "productive"
}
```

**Tag an entry:**
```json
{
  "action": "tag",
  "date": "2024-01-15",
  "add": ["important"],
  "remove": ["draft"]
}
```

## Storage

Journal entries are stored as JSON files in:
```
~/.local/share/arawn/journal/YYYY-MM-DD.json
```

Each entry contains:
- `date`: Entry date
- `mood`: How you're feeling
- `accomplishments`: What went well
- `goals`: What you want to focus on
- `notes`: Additional thoughts
- `tags`: Organization tags
- `created_at`: Timestamp

## Plugin Structure (Claude Code Format)

```
journal/
├── .claude-plugin/
│   └── plugin.json           # Plugin manifest
├── tools/
│   └── journal.sh            # CLI tool script
├── skills/
│   ├── journal-entry/
│   │   └── SKILL.md          # Guided entry skill
│   └── journal-review/
│       └── SKILL.md          # Review skill
├── hooks/
│   ├── hooks.json            # Hook declarations
│   └── session-end.sh        # Session end hook script
├── agents/
│   └── journal-assistant.md  # Agent definition
└── README.md
```

## License

MIT
