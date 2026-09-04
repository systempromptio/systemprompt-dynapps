# 2026-09-04 — Mapping AI usage to outcomes

**Status:** open. The measurement side is built; the outcome side has no agreed
definition and no connected source system. Related: DEC-002 in the requirements
register (identity source of truth and external platform mapping), still
`Awaiting Astound`; REQ-007 and REQ-008, both `Blocked` on it.

## Context

The platform measures consumption thoroughly. Every gateway request records input
and output tokens, cache-read and cache-creation tokens, computed cost, latency,
model, provider, and the user, session and trace it belongs to; those roll up per
user per day and drive `/admin/analytics` and the per-user page.

Two things are worth saying plainly about that data. Cost is **computed from a
price catalogue**, not billed by the provider — an accurate estimate, not an
invoice. And the customer-facing report deliberately never selects cost, so a
report shared with a team shows usage without exposing budget; spend lives in the
internal report only.

What the platform cannot do is say whether any of that usage was **worth it**.

## The gap

The only outcome-shaped data captured today is self-reported by the model: at the
end of a Claude Code session a summariser records whether the stated goal was
achieved, a quality score, and a goal-to-outcome mapping. This is a good coaching
signal and a reasonable way to find sessions that went badly.

It is not a business metric. It is a model's opinion of its own work, and
presenting it as an observed result would misrepresent it. Nothing today joins AI
usage to a ticket closing, a deal advancing, an incident resolving, or a release
shipping.

## The decision required

Before any engineering, we need agreement on what an outcome *is*. Candidates, in
rough order of how directly observable they are:

1. **Jira issue transitions** — cycle time in-progress to done, reopen rate,
   throughput. The most concrete option for engineering work, and the easiest to
   distort once it becomes a target.
2. **Source control** — pull requests merged, review turnaround, change failure
   rate. Closest to delivery, but only meaningful once commits made outside Claude
   Code are visible, which is exactly what DEC-002 blocks.
3. **Salesforce records** — opportunities advanced, cases resolved. The right
   measure for commercial rather than engineering usage.
4. **Ratified self-report** — keep the session goal signal but have a human
   confirm or reject it, converting an opinion into a label. The cheapest option
   and the only one that needs no new connector.

## Dependencies that follow from any choice

1. **An API key per source system.** Jira, Confluence and Salesforce each need
   credentials before their data can be correlated with usage. Without them there
   is no outcome side of the join, whatever definition is chosen.

2. **An identity mapping rule** from the external account to a platform user — a
   Jira account id, a Salesforce username, a commit-author email. This is the
   harder half, and it is already the substance of DEC-002 for source control.
   Whatever rule is agreed there should be reused rather than re-invented per
   system, with a manual override table for the mismatches that always exist.

## Position until it is decided

The platform reports what AI cost and what it produced in code, labels the code
metrics as proxies in the interface, and claims nothing further. That restraint is
deliberate and should be preserved: a fabricated productivity number is worse for
the client relationship than an absent one.

## Consequences

- `services/content/documentation/enterprise-analytics.md` gains a "From usage to
  outcomes" section stating the above, so the published account matches this one.
- DEC-010, drafted in `register-rows-2026-09-04.md`, puts the outcome definition
  to the Customer in the register's own format, cross-referencing DEC-002 rather
  than duplicating it.
