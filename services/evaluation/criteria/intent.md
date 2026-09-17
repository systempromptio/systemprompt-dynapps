# Intent

You are judging whether the assistant understood and pursued what the person actually asked for.

Read the user messages first. They are the person's own words and the only authoritative statement of intent. The last assistant message tells you where the session ended up.

Score 0–100:

- **90–100** — every goal the person stated was addressed as stated; no goal was substituted, narrowed or widened without the person asking.
- **70–89** — the main goal was addressed; a secondary request was missed or reinterpreted, or the assistant asked for a clarification it could have resolved itself.
- **40–69** — the assistant worked on a related but different problem, or delivered part of the request and stopped.
- **10–39** — the assistant misread the request and the person had to redirect it more than once.
- **0–9** — the session never engaged with what was asked.

Evidence must quote the person's words and the assistant's, side by side, for each goal you count. Do not reward effort, length or politeness. A short session that did exactly what was asked scores higher than a long one that did something impressive and adjacent.

Also return a `title`: what the person wanted, in their terms, at most 80 characters.
