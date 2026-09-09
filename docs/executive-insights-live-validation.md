# Executive and UK sales skill validation — 9 September 2026

The local rollout uses group `uk` and attribution project
`business-implementation`. Migration 052 corrects the earlier spelling while
preserving memberships, defaults and project references. It also updates the
manual-role database constraint to admit `super_admin`.

## Method

`scripts/executive-insights/live.py` exercises the running local gateway, not a
mock model or a text-only prompt. It exchanges a temporary caller PAT, verifies
the signed bridge manifest and caller identity, and loads the delivered skill
instructions. It discovers actual MCP schemas and exposes selected read-only
Atlassian tools plus knowledge-bank reads. The model selects and executes reads
through the caller's MCP session. The research case additionally enables the
provider's public web-search tool; confidential context is not a search input.

Each run saves the instruction hash, identity, status, full tool transcript and
Markdown report in a private output directory. A completed model response is
only an execution result: factual review against the returned sources is required.
`--resume` reuses earlier tool evidence and reports token usage for the resumed
segment only; use a fresh output directory for a clean acceptance run.

Example (supply your own short-lived caller PAT and scoped prompt):

```bash
python3 scripts/executive-insights/live.py \
  --credential-file /private/caller-pat \
  --skill executive_portfolio_health \
  --prompt-file /private/portfolio-prompt.txt \
  --output /private/portfolio-evaluation
```

Add `--web` for external client research. The harness deliberately refuses a
connected Salesforce server until read schemas are explicitly selected; it cannot
be used to claim that untested CRM reads passed. It exposes no write MCP tools.
Revoke only the temporary test PAT afterward; do not revoke the user's entire
access or disconnect their existing connector.

## Local scope and limitations

Testing used the owner's existing local user and existing Atlassian connection.
The caller was explicitly granted the new Super Admin role locally, and the
signed manifest delivered both plugins and all six skills. The UK group and
correctly named attribution project exist. Marketplace permissions do not widen
Atlassian permissions.

The permission-visible scope was one Jira project and one Confluence space.
Jira returned 76 issues across two labelled demonstration datasets; project
metadata advertised 88 issues. The cause of that difference is unknown. This is
a connector and reporting test using real authenticated access to synthetic
content, not an assessment of Astound's production UK portfolio. Source inventory
may change during concurrent local work.

Salesforce was absent from the caller's manifest. Sales tests therefore cover
Atlassian/knowledge-bank discovery, explicit CRM gaps and public research, not
live Salesforce records. The Larkspur discovery/account examples are fictional;
the public research example treats Marks & Spencer as a prospective company,
with no established Astound relationship.

## Findings corrected during testing

Initial and intermediate completed reports were rejected for factual quality: they sometimes
promoted knowledge-bank fixtures to real account intelligence, extrapolated one
issue's empty fields to the entire portfolio, compared one seed dataset's
expected inventory with the combined project, or misread yesterday as necessarily
inside the last 24 hours. Skills now require explicit dataset boundaries,
field/sample coverage, exact rolling-window timestamps, and internal verification
of synthetic or unavailable records. Portfolio RAG is explicitly Red/Amber/Green;
a retrieval-augmented-generation access probe is not a project status report.
Long-list counting also failed review: executive skills now require an explicit
count response or calculation from complete pages, otherwise omit the number.
Public research now explicitly searches primary company sources and excludes
uncorroborated automated financial commentary from headline recommendations.

## Results

All six skills completed authenticated gateway/MCP runs with the owner's local
identity. Each final instruction hash was checked against the signed manifest.
Final runs made 47 MCP reads with no MCP tool errors or write tool calls. Public research made
five native web searches, with no web-search error results.

| Skill | Coverage established | Remaining limitation |
|---|---|---|
| Portfolio health | Jira/Confluence reads, synthetic-scope fallback | No real portfolio health/trend evaluated |
| Blockers and decisions | Jira/Confluence reads, synthetic-scope fallback | No real blocker prioritisation/recovery decision evaluated |
| Emergency recovery | Jira/Confluence reads; final reported counts match returned records | No real incident/recovery trajectory evaluated |
| Client discovery | Atlassian + knowledge-bank; fixture identity and CRM gap identified | No real CRM account discovery evaluated |
| Account/meeting report | Atlassian + knowledge-bank; independent fixtures kept separate | No live opportunities, commitments or pipeline verified |
| Public client research | Five web searches; company-primary evidence and dated developments | CRM absent; some secondary findings remain uncorroborated |

The research headline was spot-checked against [the primary M&S announcement](https://corporate.marksandspencer.com/newsroom/press-releases/bringing-ms-closer-customers-across-europe-through-zeos)
and supported by it. That is not a verification of every secondary claim in the
research output. Reports remain drafts requiring source review; a completed model
response is not a guarantee of factual accuracy.

Validation: local build passed; all 815 integration tests passed (including the
four executive access/config/migration tests); all six skill files passed the skill
validator. Rust formatting, SQLx cache checks and all 33 static lint gates passed.
An earlier static run observed a missing unrelated `@REQ-001` tag during concurrent
work; the final rerun passed without changing those files.

Private reports and raw source evidence remain under the gitignored
`.build/executive-live/` directory, indexed by `evaluation.json` and `REPORT.md`.
No credentials or client-source transcripts are committed. No remote deployment
or Atlassian mutation was performed.
