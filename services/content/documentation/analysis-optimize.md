---
title: "Improve a Skill: Campaign to Publication"
description: "Start from one skill, create a campaign with a baseline and a capped budget, preflight and launch an experiment, approve what needs a human, then read Impact and publish or roll back."
author: "Astound Digital"
slug: "analysis-optimize"
keywords: "campaign, baseline, objective, quality floor, paired cases, experiment, preflight, holdout, approval, impact, publication, rollback"
kind: "guide"
public: true
tags: ["enterprise", "admin", "analytics"]
published_at: "2026-09-16"
updated_at: "2026-09-16"
after_reading_this:
  - "Create a campaign from a skill and set every field on the form deliberately"
  - "Preflight a frozen matrix, read its price, and launch exactly that matrix"
  - "Approve or deny the exact operation an execution asked for"
  - "Read an Impact cohort without over-claiming, and publish or roll back on Publication"
related_docs:
  - title: "Analysis: Measure, Improve and Publish a Skill"
    url: "/documentation/analysis"
  - title: "Measure Which Skills Are Used"
    url: "/documentation/analysis-measure-skills"
  - title: "Versions: Sources, Generations and Compare"
    url: "/documentation/analysis-versions"
---

# Improve a skill: campaign to publication

**TL;DR:** A campaign is the policy, an experiment is the evidence, publication is the decision. Nothing in this chain spends money until you launch a priced matrix, and nothing reaches users until a person records a review.

## Prerequisites

- A skill with a captured revision. Only resources with a captured revision can be optimised; the inventory sync captures and publishes every configured skill within a minute.
- Retained usage on that skill, so the baseline is chosen against numbers rather than a hunch. See [Measure Which Skills Are Used](/documentation/analysis-measure-skills).
- Manage rights for the approval and publication steps.

## Step 1: Start from the skill

On `/admin/analysis/resources`, find the skill's row and press **Optimize** in the Actions column. Campaigns opens with that skill pre-selected as the baseline, and the hint under the field says so.

If the link is missing, the skill is not bound to a managed resource with a captured revision yet. Press **Sync inventory now** and try again.

## Step 2: Fill in the campaign form

Expand **New campaign** on `/admin/analysis/campaigns`. Creating a campaign does not start inference.

| Field | What to put in it |
|---|---|
| **Campaign name** | A name a colleague will recognise in the table. Up to 200 characters. |
| **Resource baseline** | The captured resource to improve. Each option carries its retained 30 day usage: invocations, users, cost, p95 and failed requests. |
| **Objective** | One of reduce tokens, reduce cost, reduce latency, improve quality. Failed requests are tracked as a quality signal on every campaign row regardless of which you pick. |
| **Total budget (USD)** | The ceiling for the whole campaign. Default 5.00, minimum 0.01. |
| **Quality floor (thousandths)** | The minimum quality a candidate must hold. 4000 means 4 out of 5. Range 1000 to 5000. |
| **Minimum paired cases per partition** | How many paired cases each partition needs before a comparison counts. Default 10, range 2 to 1000. |
| **Maximum iterations** | How many rounds the campaign may run. Default 3, range 1 to 100. |
| **Allow automatic follow-up development runs** | Leave unticked to drive each round by hand. Ticked, follow-up development runs proceed inside the budget and iteration limit already set. |

Press **Create campaign**. The campaign appears in the table with its state, baseline provenance, objective and budget account.

Watch the **Stale baselines** tile. A campaign goes stale when the served generation moves past the baseline it pinned, and the row carries a `stale` badge. Re-baseline before trusting a comparison from it.

## Step 3: Launch a comparison

In the campaign's row, expand **Run…** under **Launch a comparison**.

1. Choose a retained experiment under **Frozen environment and dataset**. This is the frozen matrix, named with its execution mode.
2. Choose a **Candidate revision**, listed by its rationale and creation time.
3. Press **Run comparison**.

To qualify a result against data it has never seen, use the second form in the same panel: pick the qualifying development experiment, give the independent **Holdout dataset revision**, and press **Prepare holdout**.

If a warning notice says execution is blocked, the reason is on the notice. Experiments cannot run until the retained environment admits them.

## Step 4: Preflight a matrix directly

You can also price a matrix without a campaign, on `/admin/analysis/evaluations` under **Preflight a frozen matrix**.

1. Enter the **Shared budget ID**, the capped account the matrix reserves against.
2. Enter an **Idempotency key**. Reusing a key returns the first experiment instead of launching a second.
3. Paste the **Frozen experiment specification (JSON)**: cases, variants and repetitions exactly as reviewed.
4. Press **Calculate complete matrix**.

Preflight is pure. It prices the complete matrix and dispatches nothing.

The estimate page then shows four numbers: **Executions** (cases × variants × repetitions), the **Conservative maximum** as an upper bound rather than an expected spend, the **Available** unreserved headroom, and **Admission**, which says whether the complete matrix fits the cap. It also prints the **Matrix digest**. A different digest is a different experiment.

If it fits, press **Launch exact frozen matrix**. That reserves the conservative maximum and dispatches every execution. If it does not fit, the matrix is never trimmed to fit the budget; raise the cap or preflight a smaller reviewed specification.

## Step 5: Watch the experiment and answer approvals

Open the experiment from the **Recorded experiments** table. Its page lists one row per case, variant and repetition, with a status, the recorded outcome and an **Evidence** link per execution.

Under **Approval waits**, an execution that requested an operation needing a human decision waits with its exact operation printed and an expiry. **Approve exact operation** authorises that operation and nothing else. **Deny** refuses it.

Execution completion establishes neither quality nor complete accounting. Read the evidence before judging a candidate. **Cancel pending work** stops the rest of the matrix, and the experiment, comparison and comparison Markdown are exportable as JSON from the same page.

## Step 6: Read Impact

Open `/admin/analysis/impact`. Filter by **Skill**, **Revision** and **Traffic class** (production, fixture, live evaluation, suggestion, judge), then press **Apply cohort**.

Cohorts are grouped by skill so revisions of the same skill sit side by side. Each row is one day, one revision, one traffic class, with invocations, attributed invocations, requests, failures, quality, latency and cost.

Two caveats the page states outright, and you should repeat when reporting a result. Before and after cohorts are associations, not causation. Related conversation cost overlaps when several skills run in one conversation, so cohort costs must never be summed across skills.

Below the cohorts, **Invocation evidence** lists the most recent 200 invocations with their attribution status, conversation link, accounted requests over total requests and tool count. **Capture a reviewed production failure** turns one of those invocations into a sanitised development case: paste the invocation id, a sanitised reproduction prompt, the expected behaviours one per line, and the deterministic assertion names. It never changes a holdout and never publishes a skill. Remove customer data yourself. The automatic redaction of obvious secrets and addresses is a backstop, not a review.

## Step 7: Publish, withdraw or roll back

Open `/admin/analysis/publications`. Adoption and improvement publish automatically when the inventory sync captures a new revision. Withdrawals and rollbacks are reviewed here.

**Withdrawal proposals** are raised when the upstream source no longer carries a resource. Approving one withdraws the published generation.

To record a decision by hand, expand **Record a review manually** and give the resource id, the revision id (leave empty to withdraw), the action, the expected generation you are publishing on top of, an idempotency key, the comparison evidence JSON and any limitations. The four actions are initial baseline adoption, publish evaluated improvement, withdraw, and rollback as a new generation. An improvement requires an experiment id inside the evidence. A mismatched expected generation is rejected, and replaying the same key never publishes twice.

**Rolling back** is the rollback action: it republishes an earlier revision as a new generation rather than deleting history.

Approval, marketplace distribution and verified installation are three separate states. Publication retains existing access policies and never grants tools or permissions. Track delivery under **Distribution deliveries**, and confirm devices actually installed it under **Installation receipts**, where verified means the consumer read every file back and the digests matched.

## Troubleshooting

### The campaign table is empty

**Symptom:** "No campaigns yet".
**Cause:** Nothing has been created in this authoring scope.
**Solution:** Pick a skill on Skills and press **Optimize**.

### Launching is refused as a duplicate

**Symptom:** Launching returns an existing experiment.
**Cause:** The idempotency key was used before.
**Solution:** That is the intended behaviour. Use a new key for a genuinely new experiment.

### A recommendation will not publish

**Symptom:** The review is rejected.
**Cause:** Either the expected generation does not match the current one, or an improvement was recorded without an experiment id in the evidence.
**Solution:** Re-read the current generation from the publication history and resubmit with matching values.

## Related pages

- [Analysis overview](/documentation/analysis)
- [Versions: Sources, Generations and Compare](/documentation/analysis-versions)
- [Measure Which Skills Are Used](/documentation/analysis-measure-skills)
