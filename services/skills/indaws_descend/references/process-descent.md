# Process mode — descending on a way of working

A process is not a text, so it cannot be scored by reading. It is scored by **running it in
simulation against real past cases**. Everything below is what changes versus artifact mode; the
loop itself (freeze → gradient → decaying step → convergence → held-out judge) is unchanged.

## 1. The artifact of record

You never edit "the process". You edit its **spec** — a written document containing, for each step:
trigger, owner (one named role), input, action, output, decision rule, exception path, and how long
it should take. The spec is what you patch with `Edit`; the real world is what the change plan
proposes to touch afterwards.

If no spec exists → write one in a single pass first (that is initialization, not descent). If a
spec exists but nobody follows it → **stop**. The delta between documented and observed is the whole
finding; descending on fiction produces a better fiction.

How to capture the observed process cheaply: walk 3 recent real cases backwards from outcome to
trigger, recording every touch, wait and handoff that actually happened.

## 2. The case set (this replaces "reading the artifact")

Freeze **5–10 real past cases** with the rubric, chosen *before* studying the spec:

| Class | Min | Why |
|---|---|---|
| Typical | 3 | the volume that pays for the process |
| Exception | 2 | where processes actually fail — missing input, wrong order, angry customer, holiday |
| Pathological | 1 | the one everyone still tells the story about |

Each round, the evaluator **dry-runs the current spec against every case** and reports, per case:
did it complete, how many touches and handoffs, where it stalled, what the spec left undefined.
Scores are computed from that, not from how the document reads.

Freeze the case set in `rubric.md`. Cases picked after reading the spec are cases the spec already
handles — the classic way a process run scores 9 and changes nothing.

## 3. Starting rubric

Adapt and re-anchor, weights sum to 1.00. **At least two dimensions must be countable from the spec
or the dry-run** so the trajectory is not purely a judge's mood.

| Dim | W | Countable? | Knobs |
|---|---|---|---|
| Throughput at the constraint — does a case clear the bottleneck sooner | 0.30 | touches/waits per case | `step-count`, `batch-size`, `wip-limit`, `parallelization`, `automation-boundary` |
| Exception coverage — share of real cases with a defined path | 0.25 | yes — cases covered / total | `exception-path`, `escalation-trigger`, `default-action` |
| Ownership clarity — every step and decision has exactly one named owner | 0.20 | yes — ownerless steps | `decision-rights`, `handoff-count`, `entry-criteria`, `exit-criteria` |
| Executability — the owner can run it on a bad day without asking anyone | 0.15 | no — judged | `instruction-specificity`, `tool-in-hand`, `preconditions` |
| Observability — you can tell it is working or broken without asking people | 0.10 | yes — instrumented steps | `instrumentation`, `sla-definition`, `exception-visibility` |

Anchor examples: exception coverage 3 = happy path only; 6 = the two common exceptions named;
9 = every case class in the set has a defined path with an owner. Ownership 3 = "the team" owns
steps; 9 = one named role per step and per decision, escalation included.

## 4. Knobs

`step-count` · `handoff-count` · `decision-rights` · `automation-boundary` (human vs system) ·
`batch-size` · `wip-limit` · `entry-criteria` · `exit-criteria` · `exception-path` ·
`escalation-trigger` · `default-action` (what happens when nobody acts) · `instrumentation` ·
`parallelization` · `rework-loop` · `preconditions` · `sla-definition`.

`default-action` is the most under-used lever: most process pain is cases sitting still because
inaction has no consequence. Giving inaction a defined outcome (auto-assign, auto-close, auto-escalate)
often outperforms adding a step.

## 5. Lenses (rotate 2 per round, same contract as `lenses.md`)

1. **Constraint** — find the one step where work queues. Everything else is secondary.
2. **Handoff audit** — every handoff is a queue plus information loss. Which could be removed by
   moving the decision to where the information already is?
3. **Failure mode (FMEA)** — for each step: how does it fail, who notices, how long until anyone
   does? Undetected failure scores worse than frequent visible failure.
4. **Exception load** — what fraction of the real case set takes the happy path? A process whose
   happy path covers 40% of reality is an exception process wearing a costume.
5. **Monday morning** — role-play the actual owner, understaffed, one input missing. Where do they
   stop and ask someone?
6. **Blast radius** — when a step is done wrong, what does it cost and can it be undone? Irreversible
   steps deserve a check the reversible ones do not.
7. **Instrumentation** — could you tell from data alone that this ran correctly last week?
8. **10× load** — does it hold at ten times the volume, or is it a person's memory in disguise?
9. **Incentive alignment** — does the person carrying a step's cost get any of its benefit? Steps
   that tax one team for another team's gain decay back within a quarter regardless of the spec.

Pair one flow lens (1, 2, 4, 8) with one robustness lens (3, 5, 6, 7, 9) in early rounds. From
round 4 (low η, no structural edits) use two robustness lenses — flow findings you are not allowed
to act on waste the round.

## 6. The bottleneck discount (process-specific gradient rule)

Multiply the magnitude of any finding whose knob is **not at the constraint** by **0.3** before
ranking. A gradient that ignores this reliably produces a spec that is tidier everywhere and faster
nowhere. If the constraint moves during the run (it often does after round 1 or 2), re-identify it
before ranking — the constraint is a property of the current spec, not of the rubric, so tracking it
is not label leakage.

## 7. Exit — a change plan, never an enactment

A converged process run produces the improved spec **plus**:

| Field | Content |
|---|---|
| Changes | each patch as one line: knob, what changes, which step |
| Affected | roles and teams touched, and who has to agree |
| Pilot | narrowest slice that would show the effect, and for how long |
| Metric | the ONE number that proves it worked, and its current value |
| Rollback | how to undo, and the signal that says undo now |
| Not done | knobs that did not move, and constraint-boundary findings escalated |

**Never enact during the run**: no cron, automation rule, CRM stage, permission, assignment or
template edits, and no messages to the affected people. Descent optimizes the spec; changing how
people work is the user's decision, taken once, with the plan in hand.

## 8. Gotchas specific to process runs

- **Deleting a step is not always progress** — check whether the work moved into someone's head, a
  spreadsheet, or a chat thread. Off-spec work still costs; it just stops being measured.
- **The spec gets better because it gets longer.** Length is not coverage. Watch `step-count`
  climbing while executability falls — that trade is usually a loss.
- **Consensus is not convergence.** Everyone agreeing the new spec reads well predicts nothing about
  the next real case; the case-set dry-run does.
- **A constraint you cannot turn** (headcount, a system you do not own, someone else's approval) is
  not a low score to grind at. Two rounds pointing there = escalate and stop.
- **Automating before converging** cements whatever shape the process had that week.
