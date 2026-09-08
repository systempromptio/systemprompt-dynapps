# Running a research

The caller's procedure, from a decided-yes to a registered source. The execution contract — folder
layout, report template, exhibit rules, tools — belongs to
[openspec-research](mdc:.cursor/agents/openspec-research.md) and is not repeated here.

## Scoping

- One research per **topic**, not per question. Several questions about the same surface belong in
  one run; two unrelated surfaces are two runs.
- Slug is kebab-case, named after the topic (`legacy-datalayer-audit`, `checkout-consent-audit`).
- **You assign the slugs**, which is what keeps parallel runs from colliding.
- Independent topics may run as parallel subagents.

## Launch packet

The subagent starts with no memory of this conversation and hardcodes no paths. Pass all of it:

| Item | Value |
|---|---|
| Working directory | `openspec/changes/<change>/research/<slug>` — you compute it |
| Header metadata | change name, phase (`discovery` or `design`) |
| Questions | `Q1..Qn`, each with the artifact consumer that depends on the answer |
| Targets | live URL, repo path, instance, API |
| Access constraints | credentials or environments you already know are unavailable |

A question with no named consumer is not sent — that is the warranted-test applied one question at
a time.

## After the run

The subagent returns one line — `RESEARCH COMPLETE · <path>` or `RESEARCH FAILED · <reason>` — and
nothing else by design. Open `report.md` yourself; it is the deliverable.

**Register it in the Source Inventory of the current phase.** The two artifacts use different column
shapes:

- discovery — `| Source | Reference / Anchor | Purpose |`
- design — `| Input | Reference | Used for |`

A design artifact re-lists a discovery-era research only when it cites it directly; otherwise it is
already inherited through design's pointer to `discovery.md`.

**Cite a specific fact as `openspec/changes/<change>/research/<slug>/report.md · Q2`**, never the
bare file — the finding number is what preserves the link to that finding's status and basis.

**Carry status and basis honestly.** A `partial`, `not-observed`, or `blocked` finding may not be
hardened into a settled statement downstream, and a `documented` finding is what the vendor claims,
not what the system was seen doing. Both distinctions exist precisely because they get lost on the
hop between artifacts.

## Re-running a research

Same slug, report overwritten; git carries the history. Do not mint `-recheck` slugs and do not
append re-check sections — either one forks the citable truth into two places.

If a re-check **contradicts a finding you already registered, raise it with the user.** That finding
may already sit inside a Gap Analysis cell, a decision's rationale, or a requirement disposition, and
a silent overwrite leaves the artifact asserting something its own source no longer supports.
