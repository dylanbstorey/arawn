#!/usr/bin/env bash
# Session End Hook - Suggest journaling if no entry exists for today
#
# This hook runs at session end and checks if there's a journal entry for today.
# If not, it returns info output suggesting the user create one.
#
# Output: JSON with outcome (allow/block/info) per hook protocol

JOURNAL_DIR="${HOME}/.local/share/arawn/journal"
TODAY=$(date +%Y-%m-%d)
TODAY_FILE="$JOURNAL_DIR/${TODAY}.json"

if [ -f "$TODAY_FILE" ]; then
    # Entry exists, just allow
    echo '{"outcome": "allow"}'
else
    # No entry for today - suggest journaling
    echo '{
        "outcome": "info",
        "output": "You haven'\''t created a journal entry today. Consider using `/journal-entry` to reflect on your day before ending the session."
    }'
fi
