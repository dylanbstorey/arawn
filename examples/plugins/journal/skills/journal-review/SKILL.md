---
name: journal-review
description: Review journal entries for a time period
user_invocable: true
arguments:
  - name: period
    description: "Time period to review: today, week, or month"
    required: true
---

# Journal Review: {period}

Review the user's journal entries for the specified time period and provide insights.

## Steps

1. **Fetch entries**: Use the `journal` tool to list entries for the {period}:

   - **today**: Just today's entry
   - **week**: Last 7 days
   - **month**: Last 30 days

   ```json
   {
     "action": "list",
     "start_date": "<calculated start date>",
     "end_date": "<today's date>",
     "limit": 31
   }
   ```

2. **Summarize patterns**: Look for:
   - Mood trends over the period
   - Common themes in accomplishments
   - Recurring goals or challenges
   - Frequently used tags

3. **Highlight insights**:
   - What went particularly well?
   - What challenges came up repeatedly?
   - Are goals being achieved?
   - Any noticeable mood patterns?

4. **Offer encouragement**: Note positive progress and suggest areas for focus.

Be thoughtful and supportive. The goal is to help the user see the bigger picture of their journey.
