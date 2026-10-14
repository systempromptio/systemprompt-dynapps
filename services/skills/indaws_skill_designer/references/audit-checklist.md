# Skill Audit Checklist

Rate each item: PASS / WARN / FAIL. A good skill should have 0 FAIL and <=2 WARN.

## Frontmatter
- [ ] `description` is trigger-oriented (action verbs + specific nouns + the phrases users actually type)
- [ ] Activation phrases live INSIDE `description` (not in a separate `activation` field)
- [ ] Only recognized fields present — `name`, `description`, `version`, `allowed-tools`, `compatibility`; custom data nested under `metadata:`. No top-level `tags`/`activation`/`globs`/`requires` (no effect — remove)
- [ ] `version` present (semver)

## Structure
- [ ] Fits ONE category cleanly (see categories.md)
- [ ] SKILL.md is under 150 lines (index, not encyclopedia)
- [ ] Deep content is in references/ (not inline in SKILL.md)
- [ ] Has at least one reference file OR one script
- [ ] No redundant files (same content in multiple places)
- [ ] No `README.md` inside the skill folder (docs go in SKILL.md/references; repo-level README is fine)

## Content Quality
- [ ] Has a GOTCHAS section with 3+ real failure patterns
- [ ] Doesn't state the obvious (no "use Write to create files")
- [ ] Doesn't railroad (gives info + constraints, not rigid steps)
- [ ] Workflow has [STOP] points where user input is needed
- [ ] NEVER/ALWAYS rules are justified (not arbitrary restrictions)
- [ ] Critical instructions are at the TOP (under `## Important`/`## Critical`), not buried — so a loaded skill is actually followed
- [ ] Deterministic critical validations are scripted, not described in prose

## Scripts & Code
- [ ] Scripts are self-contained (minimal external deps)
- [ ] Scripts have clear names (filename = documentation)
- [ ] Scripts are composable (Claude can chain them)
- [ ] No hardcoded credentials/IDs/URLs (use config.json or env vars)

## Token Efficiency
- [ ] JSON data uses compact format (not pretty-printed with 4-space indent)
- [ ] No repeated defaults in data files
- [ ] CSS/HTML uses short class names if inline
- [ ] Progressive disclosure is used (Claude reads references on demand)
- [ ] No information Claude already knows (general coding, popular libraries)

## Output & Reporting
- [ ] Skill produces structured output (not just console text)
- [ ] Output is actionable (tells user what to do next)
- [ ] Error states are classified (not just "something went wrong")

## Cross-Skill Integrity
- [ ] No content duplicated across skills — if knowledge exists in a hub skill (e.g., `odoo-version-api-reference`), reference it instead of duplicating
- [ ] All referenced skills exist in the repo — grep for `/skill-name` in delegation tables and verify each one is a real skill
- [ ] Gotchas are not duplicated — before adding gotchas, check hub skill's gotchas.md for existing entries
- [ ] Content not lost on reduction — when moving content out of SKILL.md to references/, verify ALL removed content lands in a reference file (examples, success criteria, etc.)
- [ ] Frontmatter uses only valid fields — Claude Code recognizes top-level `name`, `description`, `version`, `allowed-tools`, `compatibility`; custom data nests under `metadata:`. Fields like `activation`, `tags`, `globs`, `requires` at top level have NO effect — remove or nest them

## Subagent Prompts (if skill uses Agent() delegations)
- [ ] Every subagent prompt is self-contained — does not reference orchestrator variables (SCAN_RESULT, API_RESULT) that the subagent can't see
- [ ] Every subagent prompt injects the Odoo version when version-dependent behavior is expected
- [ ] Post-action subagents (Pattern C) receive: module purpose, architecture decisions, and original requirements — not just a path
- [ ] Subagent descriptions are specific (not "scan module" but "scan {tech_name} for inventory")

## Distribution
- [ ] Works standalone (doesn't silently depend on uninstalled skills)
- [ ] Config setup is guided (asks user on first run)
- [ ] State persists correctly (uses stable storage path)
- [ ] Description alone is sufficient for the model to trigger it (no reliance on a separate activation field)

## Ecosystem (multi-repo)
- [ ] Skill is in the correct suite (see ecosystem-governance.md routing tree)
- [ ] No duplicate exists in another suite (run `detect_duplicates.py`)
- [ ] Cross-suite references point to real skills (no phantom references)
- [ ] If in common: used by 2+ roles. If role-specific: in the role's suite
- [ ] DEPENDENCIES.md of parent suite is up to date with this skill's needs
- [ ] Version field present in frontmatter (semver)

## Scoring
| Range | Rating |
|-------|--------|
| 0 FAIL, 0-1 WARN | Excellent — ready for marketplace |
| 0 FAIL, 2-3 WARN | Good — minor improvements needed |
| 1 FAIL | Needs work — fix before distributing |
| 2+ FAIL | Redesign needed |
