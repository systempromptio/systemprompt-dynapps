# Revamp Playbook

Complete recipe for transforming a skill suite into the federated platform standard. Tested on common (30/30 PASS).

## 8 Phases (execute in order)

### Phase 1 — Frontmatter (batch, mechanical)
- Add `version: 1.0.0` where missing (keep existing versions)
- Add `effort: low|medium|high`
- Add `model: sonnet|opus` (see mapping below)
- Remove ghost FM: activation, tags, globs, requires, references, triggers, tools

### Phase 2 — Progressive disclosure (per skill)
- Every SKILL.md > 150 lines → move content to references/
- SKILL.md keeps only: frontmatter, short description, workflow summary, rules, gotchas, references index
- Nothing gets lost — every removed line must land in a reference file

### Phase 3 — Domain content (requires domain knowledge)
- Add `## GOTCHAS` (5-7 per skill, from real failures)
- Add `## RULES` with NEVER/ALWAYS
- Add `## Workflow` with numbered steps

### Phase 4 — Delegation (the core)
- Add `## Delegates to` — who this skill invokes, which suite, when
- Add `## Consumed by` — who invokes this skill, which suite, for what
- Reference `common:references/shared/` for connection, batch, fields, protocol
- Remove content duplicated from common (version diffs, estimation, scanning, auth, brand colors)

### Phase 5 — Agent routing (per skill)
- Sonnet skills: `## Agent Routing\nRuns entirely in sonnet — {reason}.`
- Opus skills: table `| Phase | Model | Why |` per step
- For effort:low slash commands: explicit `Agent(model: sonnet)` instruction
- CRITICAL: `model:` in frontmatter does NOT route in slash commands

### Phase 6 — Suite infrastructure
- `CATALOG.md` — run `generate_catalog.py`
- `DEPENDENCIES.md` — 3 sections: Vertical (common), Horizontal (peers), Optional
- `STRUCTURE.md` — file tree of the suite

### Phase 7 — Deduplication
- Search for content duplicated from common (version diffs, estimation, scanning, auth, brand)
- Replace with references to source of truth
- Run `detect_duplicates.py`

### Phase 8 — Validation
- Run `verify_suite.py` on the suite → target: N/N PASS
- Run `validate_ecosystem.py` → 0 errors
- Run `detect_duplicates.py` → 0 duplicates

## Checklist per skill (11 checks)

```
□ Frontmatter: name, version, description, model, effort
□ No ghost FM
□ SKILL.md < 150 lines
□ GOTCHAS (5+)
□ NEVER/ALWAYS rules
□ Workflow with steps
□ Delegates to (with suite)
□ Consumed by (with suite)
□ Agent routing (tags per step or one-liner)
□ Reference to common:references/shared/
□ No duplicated content from common
```

## Effort → Model mapping

| Effort | Model | When |
|--------|-------|------|
| low | sonnet | Lookups, scripts, data, references, brand |
| medium, no code generation | sonnet | Reports, templates, content, diagrams |
| medium, code/architecture | opus | Code gen, complex mapping, judgment |
| high | opus | Deep analysis, migration, multi-phase |

## Phase execution tips

- **Phases 1-2**: Can be batched with a single subagent processing all skills
- **Phase 3**: Requires domain knowledge — do per skill or use subagents with detailed gotcha lists
- **Phase 4**: Build delegation map from DEPENDENCIES.md first, then apply to each skill
- **Phase 5**: Sonnet skills get a one-liner (fast). Opus skills get a table (need to understand the workflow)
- **Phase 6**: CATALOG.md is auto-generated. DEPENDENCIES.md requires analyzing real cross-suite references
- **Phase 7**: Run exploration agent to find duplicates before fixing
- **Phase 8**: Run `verify_suite.py` — it checks all 11 items automatically
