# Modes of Use and the Escalation Gate

> Instructions are English. **Everything this skill emits goes out in the reader's language** —
> including the `BLOCKED — needs a dev` line and the chatter note. See SKILL.md `## Critical`.

## Two modes of use

| Mode | When | What it does |
|------|------|--------------|
| **A — New request** | BEFORE anything is sketched | The full ladder Standard > Core > Studio > Custom. The verdict decides whether anything gets sketched, and in what. **This is the default mode.** |
| **B — Promotion** | AFTER sketching, before pushing to production | The consultant already prototyped in Studio and the client validated it. It audits what was built (STEP 3.5, promotion mode) and decides whether it ships as Studio or is rewritten as a governed module. |

**Detect the mode in STEP 0; do not ask if it can be deduced:** if a Studio change exists in
staging and not in production, it is mode B.

**Mode B warning — the sketch bias.** What is already sketched biases the verdict: it is hard to
knock down something the client has seen working. In mode B the question is NOT "is this good
enough?", it is **"would this survive STEP 3.5 if it did not exist yet?"**. If the answer is no,
the sketch is thrown away however much it hurts: it gets rewritten as a module.

**If the two environments differ, that is two assessments, not one.** Never extrapolate a staging
verdict to production without auditing production. A green verdict in one environment says
nothing about the other (see `output-standard/`: two branches, two sibling documents, explicitly
not interchangeable).

---

## Escalation gate to a dev (non-negotiable)

There are cases where the skill emits **NO verdict** and instead returns
**`BLOCKED — needs a dev`**. Any one of these fires the gate:

- A **zone collision** detected in STEP 3.5 and the consultant wants to carry on with Studio.
- The request touches **shared Core** (`indaws_*` installed at more than one client). The code
  says it breaks; only the environment inventory says **how many clients**. Without that
  measurement there is no verdict.
- It changes the **semantics of an existing `store` field** that already holds real data.
- The change goes **straight to production** without passing through staging.
- Someone asks to **skip the verdict** without providing a new technical argument.

### What the skill does at the gate

1. It emits `BLOCKED — needs a dev`: **the reason in one line**, the affected zone/piece, and the
   **exact technical identifier** (module, model, field, xmlid).
2. It does **NOT offer a laxer alternative.** It does not negotiate downwards.
3. To unblock it demands **TWO concrete data points**, not an assertion:
   - **the name of the dev** taking the decision, and
   - **the override written in the chatter** of the source record (`message_post`,
     `subtype_xmlid: mail.mt_note`) with the technical reason.
4. If pressed **without those two data points**, the answer is always the same sentence, neither
   reworded nor softened: **"Still BLOCKED. Missing {datum}."** Repeating the pressure is not a
   new argument.
5. Once they are provided, the skill continues **and records** who overrode it and why.

### Why an artefact and not persistence

A guardrail that amounts to "being annoying" caves on the third refusal, and then gives false
safety: the consultant believes they passed a control. Demanding an external artefact (a name +
a written note) cannot be argued away, only supplied.

It is the same principle as the mechanical gate on phases: *a door that cannot be closed is not
a door*.

### What the skill does not decide

**What is a business decision is declared as such and is NOT decided here**: what gets charged,
what gets written off, what the client sees, and any case where being wrong by excess is not
symmetrical to being wrong by omission (payroll, treasury, tax). A person signs that.

**The gate does not protect against the technical error: it protects against nobody signing it.**
