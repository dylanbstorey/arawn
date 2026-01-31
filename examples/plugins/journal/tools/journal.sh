#!/usr/bin/env bash
# Journal CLI Tool - JSON stdin/stdout protocol
#
# Actions:
#   create - Create a new journal entry
#   list   - List entries by date range
#   search - Search entries by text
#   tag    - Add or remove tags from an entry
#
# Input: JSON object with "action" and action-specific fields
# Output: JSON object with "success", "content", and optional "error"

set -euo pipefail

JOURNAL_DIR="${HOME}/.local/share/arawn/journal"
mkdir -p "$JOURNAL_DIR"

# Read JSON input from stdin
INPUT=$(cat)

# Parse action from input
ACTION=$(echo "$INPUT" | jq -r '.action // "list"')

case "$ACTION" in
    create)
        # Extract fields
        DATE=$(echo "$INPUT" | jq -r '.date // ""')
        MOOD=$(echo "$INPUT" | jq -r '.mood // ""')
        ACCOMPLISHMENTS=$(echo "$INPUT" | jq -r '.accomplishments // ""')
        GOALS=$(echo "$INPUT" | jq -r '.goals // ""')
        NOTES=$(echo "$INPUT" | jq -r '.notes // ""')
        TAGS=$(echo "$INPUT" | jq -c '.tags // []')

        # Default date to today
        if [ -z "$DATE" ] || [ "$DATE" = "null" ]; then
            DATE=$(date +%Y-%m-%d)
        fi

        ENTRY_FILE="$JOURNAL_DIR/${DATE}.json"

        # Create entry JSON
        ENTRY=$(jq -n \
            --arg date "$DATE" \
            --arg mood "$MOOD" \
            --arg accomplishments "$ACCOMPLISHMENTS" \
            --arg goals "$GOALS" \
            --arg notes "$NOTES" \
            --argjson tags "$TAGS" \
            --arg created_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
            '{
                date: $date,
                mood: $mood,
                accomplishments: $accomplishments,
                goals: $goals,
                notes: $notes,
                tags: $tags,
                created_at: $created_at
            }')

        echo "$ENTRY" > "$ENTRY_FILE"

        echo "{\"success\": true, \"content\": \"Journal entry created for $DATE\"}"
        ;;

    list)
        # Extract date range
        START_DATE=$(echo "$INPUT" | jq -r '.start_date // ""')
        END_DATE=$(echo "$INPUT" | jq -r '.end_date // ""')
        LIMIT=$(echo "$INPUT" | jq -r '.limit // "10"')

        # Default to last 7 days if no dates specified
        if [ -z "$START_DATE" ] || [ "$START_DATE" = "null" ]; then
            START_DATE=$(date -v-7d +%Y-%m-%d 2>/dev/null || date -d "7 days ago" +%Y-%m-%d)
        fi
        if [ -z "$END_DATE" ] || [ "$END_DATE" = "null" ]; then
            END_DATE=$(date +%Y-%m-%d)
        fi

        # Find and collect entries
        ENTRIES="[]"
        for FILE in "$JOURNAL_DIR"/*.json; do
            [ -f "$FILE" ] || continue
            FILENAME=$(basename "$FILE" .json)
            if [[ "$FILENAME" > "$START_DATE" || "$FILENAME" == "$START_DATE" ]] && [[ "$FILENAME" < "$END_DATE" || "$FILENAME" == "$END_DATE" ]]; then
                ENTRY=$(cat "$FILE")
                ENTRIES=$(echo "$ENTRIES" | jq --argjson entry "$ENTRY" '. + [$entry]')
            fi
        done

        # Sort by date descending and limit
        ENTRIES=$(echo "$ENTRIES" | jq "sort_by(.date) | reverse | .[:$LIMIT]")
        COUNT=$(echo "$ENTRIES" | jq 'length')

        if [ "$COUNT" -eq 0 ]; then
            echo "{\"success\": true, \"content\": \"No journal entries found from $START_DATE to $END_DATE\"}"
        else
            CONTENT=$(echo "$ENTRIES" | jq -r --arg start "$START_DATE" --arg end "$END_DATE" '
                "Found \(length) journal entries from \($start) to \($end):\n\n" +
                (map("## \(.date)\nMood: \(.mood)\nAccomplishments: \(.accomplishments)\nGoals: \(.goals)\nNotes: \(.notes)\nTags: \(.tags | join(", "))\n") | join("\n"))
            ')
            echo "{\"success\": true, \"content\": $(echo "$CONTENT" | jq -Rs .)}"
        fi
        ;;

    search)
        QUERY=$(echo "$INPUT" | jq -r '.query // ""')

        if [ -z "$QUERY" ] || [ "$QUERY" = "null" ]; then
            echo '{"success": false, "error": "search requires a query parameter"}'
            exit 0
        fi

        RESULTS="[]"
        for FILE in "$JOURNAL_DIR"/*.json; do
            [ -f "$FILE" ] || continue
            if grep -qi "$QUERY" "$FILE"; then
                ENTRY=$(cat "$FILE")
                RESULTS=$(echo "$RESULTS" | jq --argjson entry "$ENTRY" '. + [$entry]')
            fi
        done

        COUNT=$(echo "$RESULTS" | jq 'length')

        if [ "$COUNT" -eq 0 ]; then
            echo "{\"success\": true, \"content\": \"No journal entries found matching '$QUERY'\"}"
        else
            CONTENT=$(echo "$RESULTS" | jq -r --arg query "$QUERY" '
                "Found \(length) entries matching \"\($query)\":\n\n" +
                (map("## \(.date)\nMood: \(.mood)\nNotes: \(.notes)\n") | join("\n"))
            ')
            echo "{\"success\": true, \"content\": $(echo "$CONTENT" | jq -Rs .)}"
        fi
        ;;

    tag)
        DATE=$(echo "$INPUT" | jq -r '.date // ""')
        ADD_TAGS=$(echo "$INPUT" | jq -c '.add // []')
        REMOVE_TAGS=$(echo "$INPUT" | jq -c '.remove // []')

        if [ -z "$DATE" ] || [ "$DATE" = "null" ]; then
            DATE=$(date +%Y-%m-%d)
        fi

        ENTRY_FILE="$JOURNAL_DIR/${DATE}.json"

        if [ ! -f "$ENTRY_FILE" ]; then
            echo "{\"success\": false, \"error\": \"No journal entry found for $DATE\"}"
            exit 0
        fi

        # Read existing entry
        ENTRY=$(cat "$ENTRY_FILE")

        # Add new tags
        ENTRY=$(echo "$ENTRY" | jq --argjson add "$ADD_TAGS" '.tags = (.tags + $add | unique)')

        # Remove tags
        ENTRY=$(echo "$ENTRY" | jq --argjson remove "$REMOVE_TAGS" '.tags = (.tags - $remove)')

        # Save
        echo "$ENTRY" > "$ENTRY_FILE"

        NEW_TAGS=$(echo "$ENTRY" | jq -r '.tags | join(", ")')
        echo "{\"success\": true, \"content\": \"Tags updated for $DATE. Current tags: $NEW_TAGS\"}"
        ;;

    *)
        echo "{\"success\": false, \"error\": \"Unknown action: $ACTION. Valid actions: create, list, search, tag\"}"
        ;;
esac
