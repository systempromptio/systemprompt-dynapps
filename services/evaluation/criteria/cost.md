# Cost

The score for this criterion is computed, not judged. It compares the session's spend per user prompt against the budget in `services/evaluation/config.yaml` and rewards cache reuse; you will be told the score.

Your job is the rationale: explain, from the metrics and the user messages, what drove the cost. Look at

- the model mix (an expensive model used for routine turns),
- tokens per request (a large context re-sent every turn without cache reads),
- the number of requests per prompt (loops, retries, subagents), and
- whether the spend bought a result the person kept.

Name the single biggest driver and, if there is one, the one change that would have cut it. Quote a user message only when it explains why the spend was necessary or shows it was wasted.
