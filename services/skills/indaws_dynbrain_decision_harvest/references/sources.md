# Sources for the harvest

Four sources, most distilled to noisiest. Walk them in that order: what you already found in the
worklog does not need deducing again from mail.

Load tools in **as few calls as possible**, grouping by server.

---

## 1 · Memory OS worklog — the best signal

The daily worklog already comes distilled, scoped to work and separated from the personal. It is
the only source where somebody already did the work of summarising.

- Run or read the output of `memory:worklog` for the days in the window.
- Read `memory/decisions.md` too if it exists: it serves to **deduplicate** against what is already
  registered, and to spot contradictions with earlier novelties.
- Active **overlays** (`memory:overlays`) give context on which projects and customers the owner
  has been on that week. Use them to understand scope, **not** as a source of novelties: an overlay
  describes a context, not a doctrine.

**If there is no Memory OS**, say so on the page. The harvest continues, but precision drops
noticeably and the owner should know that while reading it.

---

## 2 · Meeting transcripts — where decisions actually happen

```
ToolSearch: "select:mcp__Granola__list_meetings,mcp__Granola__get_meeting_transcript,mcp__Granola__query_granola_meetings"
```

1. `list_meetings` over the window. Prioritise the internal ones: committees, department syncs,
   product meetings. **Customer meetings rarely produce doctrine** — they produce project
   agreements, which are not this.
2. For the candidates, `get_meeting_transcript`, and look for the closing moments: where the
   conversation stops arguing and somebody summarises.
3. `query_granola_meetings` is for searching closing patterns directly ("a partir de ahora",
   "dejamos de", "queda decidido") without reading whole transcripts.

**Always quote the literal sentence and who said it.** It is what lets the owner confirm at a
glance — and what stops them confirming something they never said.

---

## 3 · Teams — quick decisions, plenty of noise

```
ToolSearch: "select:mcp__Dynapps_365__d365_list_channels,mcp__Dynapps_365__d365_list_channel_messages,mcp__Dynapps_365__d365_search_messages,mcp__Dynapps_365__d365_list_chats,mcp__Dynapps_365__d365_list_chat_messages"
```

- Prioritise **team channels** over one-to-one chats: what is decided in a channel already has
  witnesses, and that is half the authority condition met.
- Among chats, keep the ones including area leads.
- A single message is almost never a decision. Look for the **closing pattern**: proposal →
  discussion → "pues hacemos X" → the conversation stops.
- Ignore threads still open at the end of the window. If it is still being argued, it is not
  decided.

Typical noise to discard without thinking: calendar coordination, "¿lo tienes?", deployment
notices, birthdays.

---

## 4 · Mail — confirmations and formality

```
ToolSearch: "select:mcp__Dynapps_365__d365_search_mail,mcp__Dynapps_365__d365_list_mail"
```

- **Internal mail only.** Customer mail contains project commitments, not doctrine.
- Look for mail that **announces** something to several people: those are already-closed decisions
  somebody bothered to communicate. Strong signal.
- A long thread with debate is usually the opposite: still open.
- Never quote a whole email. The minimum sentence and the subject.

---

## Consolidation and cross-source reinforcement

**The same novelty in two sources raises confidence.** The most reliable pattern:

> discussed in a meeting → somebody writes it in Teams or by mail → nobody pushes back

When you spot that, quote **both pieces of evidence**. It is the easiest candidate to confirm and
the least likely to be a misunderstanding.

When consolidating:

- One novelty, one record. If it shows up in three places, it is still one.
- Pick as primary evidence the **most explicit** one, not the most recent.
- As `decided_on`, the date it was closed, not the last time it was mentioned.
- As `decided_by`, whoever closed it — and if it was a group, the group.

---

## What never enters the harvest

- Nothing belonging to another person: this skill runs in the owner's session and only sees theirs.
- Nothing personal, even if it appears in a work source.
- Nothing about specific people: performance, salary, hiring, health, conflicts.
- Nothing from an identifiable customer that is a commercial agreement rather than doctrine.

When in doubt, out. A lost candidate comes back next week; an inappropriate candidate on the page
burns trust in the whole system.
