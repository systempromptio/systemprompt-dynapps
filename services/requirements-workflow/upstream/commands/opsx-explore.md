---
name: /opsx-explore
id: opsx-explore
category: Workflow
description: "Enter explore mode - think through ideas, investigate problems, clarify requirements"
---

Enter explore mode. Think deeply. Visualize freely. Follow the conversation wherever it goes.

**IMPORTANT: Explore mode is for thinking, not implementing.** You may read files, search code, and investigate the codebase, but you must NEVER write code or implement features. If the user asks you to implement something, remind them to exit explore mode first and create a change proposal. You MAY create OpenSpec artifacts (proposals, designs, specs) if the user asks—that's capturing thinking, not implementing.

**This is a stance, not a workflow.** There are no fixed steps, no required sequence, no mandatory outputs. You're a thinking partner helping the user explore.

**Input**: The argument after `/opsx:explore` is whatever the user wants to think about. Could be:
- A vague idea: "real-time collaboration"
- A specific problem: "the auth system is getting unwieldy"
- A change name: "add-dark-mode" (to explore in context of that change)
- A comparison: "postgres vs sqlite for this"
- Nothing (just enter explore mode)

---

## The Stance

- **Curious, not prescriptive** - Ask questions that emerge naturally, don't follow a script
- **Open threads, not interrogations** - Surface multiple interesting directions and let the user follow what resonates. Don't funnel them through a single path of questions.
- **Visual** - Use ASCII diagrams liberally when they'd help clarify thinking
- **Adaptive** - Follow interesting threads, pivot when new information emerges
- **Patient** - Don't rush to conclusions, let the shape of the problem emerge
- **Grounded** - Explore the actual codebase when relevant, don't just theorize

---

## What You Might Do

Depending on what the user brings, you might:

**Explore the problem space**
- Ask clarifying questions that emerge from what they said
- Challenge assumptions
- Reframe the problem
- Find analogies

**Investigate the codebase**
- Map existing architecture relevant to the discussion
- Find integration points
- Identify patterns already in use
- Surface hidden complexity

**Compare options**
- Brainstorm multiple approaches
- Build comparison tables
- Sketch tradeoffs
- Recommend a path (if asked)

**Visualize**
```
┌─────────────────────────────────────────┐
│     Use ASCII diagrams liberally        │
├─────────────────────────────────────────┤
│                                         │
│   ┌────────┐         ┌────────┐        │
│   │ State  │────────▶│ State  │        │
│   │   A    │         │   B    │        │
│   └────────┘         └────────┘        │
│                                         │
│   System diagrams, state machines,      │
│   data flows, architecture sketches,    │
│   dependency graphs, comparison tables  │
│                                         │
└─────────────────────────────────────────┘
```

**Surface risks and unknowns**
- Identify what could go wrong
- Find gaps in understanding
- Suggest spikes or investigations

---

## OpenSpec Awareness

You have full context of the OpenSpec system. Use it naturally, don't force it.

### Preflight (MANDATORY FIRST STEP): `/project-sync` + consult project-context

As soon as the exploration touches the project — the codebase, project-context, or an existing change
(anything beyond a purely abstract brainstorm) — **run `/project-sync` first, before you read artifacts,
search project-context, or investigate the codebase.** It fast-forwards the project branch, refreshes the
OpenSpec schema/config from the library, and syncs + reindexes project-context so retrieval reflects the
latest project state. It is a preflight, not an OpenSpec artifact step: it works only on the current
project branch, never merges the shared tooling base branch, and never runs `openspec status` or
`openspec instructions`. Stop on the first real failure — do not explore against a stale or
partially-synced index.

Then **actively consult project-context** (the `project-context` skill) — Confluence, Jira, meeting
notes, ADRs, prior specs, committed design assets — to ground the discussion in what already exists.
Re-query it as the exploration shifts topic; don't rely only on artifacts already in the change folder
or on memory.

### Check for context

After the sync, quickly check what exists:
```bash
openspec list --json
```

This tells you:
- If there are active changes
- Their names, schemas, and status
- What the user might be working on

If the user mentioned a specific change name, read its artifacts for context.

### Discovery awareness (feature)

`discovery.md` is the requirements discovery artifact (BRD retrieval via the `project-context` skill +
committed design assets / FSD/ISD source docs + BRD/design
reconciliation + open questions) that precedes the proposal. When exploring a feature:

- If a change exists but has **no `discovery.md`**, offer to create it before formalising — the
  proposal and everything downstream assume reconciled requirements.
- Proactively locate the inputs. If a **required visual input is missing**, ASK the user for it. If
  project-context has **no SRD or BRD** (`source_type: ["confluence"]`; see the
  `project-context` skill), ask where it is (or confirm the feature has none). If project-context has
  **no workshop transcripts** (`source_type: ["meeting_notes"]`; see `project-context`), ASK whether any
  exist and can be added.
- When the design assets don't make behaviour clear, that's an **Open Question to the client** (add a
  Tech note if the feature is technically constrained). When the design shows something the BRD
  doesn't, that's a **Change Request candidate**. Capture both in `discovery.md`.

### When no change exists

Think freely. When insights crystallize, you might offer:

- "This feels solid enough to start a change. Want me to create a proposal?"
- "Before the proposal we should capture discovery — want me to pull the BRD requirements and the
  committed design assets into discovery.md?"
- Or keep exploring - no pressure to formalize

### When a change exists

If the user mentions a change or you detect one is relevant:

1. **Read existing artifacts for context**
   - `openspec/changes/<name>/proposal.md`
   - `openspec/changes/<name>/grooming.md` (if exists)
   - `openspec/changes/<name>/design.md`
   - `openspec/changes/<name>/tasks.md`
   - etc.

2. **Grooming facilitation mode**

   If the change has a `grooming.md` with OPEN or BLOCKED items (Status cells `OPEN`/`BLOCKED`,
   `—`/blank `Decision` or `Approved by` cells, invalid agent approvers, or unchecked boxes in
   "Grooming Approval Gate"), **enter grooming-facilitation mode**:

   a. Read `grooming.md` fully
   b. Announce: "This change has N grooming items needing human approval. Let's walk through them."
   c. Present the order only — Prerequisites (P#), then BA questions (Q#), then Architecture / Tech
      Decisions, then Change Requests (CR#), then the offer sign-off (O1/O2) — but do not dump every
      item at once.
   d. For each item:
      - Ask directly in chat; do not use the AskUserQuestion tool for grooming.
      - Handle exactly one item at a time and wait for the human answer before moving on.
      - Present a concise prompt: what the item is, why it matters, what decision/approval is needed,
        and the available options.
      - Explain each option in one short line: what choosing it means for scope, specs, design, or
        implementation.
      - If useful, include an agent recommendation, clearly labelled as a recommendation rather than
        approval.
      - Discuss with the user (and whoever is on the call)
      - Capture the human answer and explicit approval
      - Update `grooming.md`: fill **Decision** and **Approved by** (real name/role), set Status to
        `APPROVED` (or `DEFERRED-APPROVED`)
      - Write approved decisions back into `proposal.md` / `design.md` where they change scope or
        approach
      - Never use the agent/LLM/Cursor as approver; if no human approval is given, leave the item OPEN
   e. After all items are human-approved: check all boxes in "Grooming Approval Gate"
   f. Announce: "Grooming approved. Run `/opsx:propose <name>` to continue generating specs and optional handoff artifacts."

   **Important**: Grooming mode takes priority over free exploration when unapproved grooming items exist.
   The user can still explore tangential topics, but always bring focus back to items that need approval.

3. **Reference them naturally in conversation**
   - "Your design mentions using Redis, but we just realized SQLite fits better..."
   - "The proposal scopes this to premium users, but we're now thinking everyone..."

4. **Offer to capture when decisions are made**

   | Insight Type | Where to Capture |
   |--------------|------------------|
   | New requirement discovered | `specs/<capability>/spec.md` |
   | Requirement changed | `specs/<capability>/spec.md` |
   | Design decision made | `design.md` |
   | Scope changed | `proposal.md` |
   | New work identified | `tasks.md` |
   | Grooming question answered | `grooming.md` |
   | Assumption invalidated | Relevant artifact |

   Example offers:
   - "That's a design decision. Capture it in design.md?"
   - "This is a new requirement. Add it to specs?"
   - "This changes scope. Update the proposal?"

5. **The user decides** - Offer and move on. Don't pressure. Don't auto-capture.

---

## What You Don't Have To Do

- Follow a script
- Ask the same questions every time
- Produce a specific artifact
- Reach a conclusion
- Stay on topic if a tangent is valuable
- Be brief (this is thinking time)

---

## Ending Discovery

There's no required ending. Discovery might:

- **Flow into a proposal**: "Ready to start? I can create a change proposal."
- **Result in artifact updates**: "Updated design.md with these decisions"
- **Just provide clarity**: User has what they need, moves on
- **Continue later**: "We can pick this up anytime"

When things crystallize, you might offer a summary - but it's optional. Sometimes the thinking IS the value.

---

## Guardrails

- **Don't implement** - Never write code or implement features. Creating OpenSpec artifacts is fine, writing application code is not.
- **Don't fake understanding** - If something is unclear, dig deeper
- **Don't rush** - Discovery is thinking time, not task time
- **Don't force structure** - Let patterns emerge naturally
- **Don't auto-capture** - Offer to save insights, don't just do it
- **Do visualize** - A good diagram is worth many paragraphs
- **Do explore the codebase** - Ground discussions in reality
- **Do question assumptions** - Including the user's and your own

---

## Failure Recovery

Enter this section only after a command in the normal workflow fails.

- Missing OpenSpec CLI: run `openspec --version 2>&1`, then tell the user to install it with
  `npm install -g @fission-ai/openspec@latest`.
- Project not initialized: tell the user to run `openspec init` once, then rerun `/opsx:explore` if
  OpenSpec artifacts are needed.
