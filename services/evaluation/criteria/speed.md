# Speed

The score for this criterion is computed, not judged. It compares the session's wall-clock minutes per user prompt and the average model latency against the targets in `services/evaluation/config.yaml`; you will be told the score.

Your job is the rationale: explain, from the metrics and the user messages, where the time went. Distinguish

- time the model spent (latency, long generations, many tool calls per prompt),
- time the person spent (long gaps between prompts, which are not the assistant's fault), and
- time lost to rework (a prompt that repeats or corrects an earlier one).

Name the single biggest contributor. If the session was fast, say what made it fast. Quote a user message only when it shows a wait or a repeat.
