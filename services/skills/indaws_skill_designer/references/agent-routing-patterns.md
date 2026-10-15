# Agent Routing Patterns

How to structure skills to use the right model at each phase — sonnet for mechanical work, opus for creative/complex work.

## The Problem

A skill with `model: opus` runs EVERYTHING in opus — including reading files, asking questions, running validations. This wastes tokens and money on tasks where sonnet performs equally well.

A skill with `model: sonnet` in frontmatter does NOT automatically route to sonnet. The `model` field only works in marketplace plugins. For slash commands, **the skill must explicitly instruct to launch an Agent with the desired model**.

## The Pattern: Orchestrator + Subagents

The SKILL.md acts as an **orchestrator** that decides which model handles each phase.

```
SKILL.md (model: opus)
  │
  ├─ Phase 0: Gather info ──→ Agent(model: sonnet)   ← reads, asks, collects
  ├─ Phase 1: Design       ──→ runs in principal      ← opus thinks
  ├─ Phase 2: Generate     ──→ runs in principal      ← opus creates
  ├─ Phase 3: Validate     ──→ Agent(model: sonnet)   ← checks, lints
  └─ Phase 4: Delegate     ──→ Agent(model: sonnet)   ← calls other skills
```

## Model Routing Rules

| Work type | Model | Why |
|-----------|-------|-----|
| Read files, registry, config | sonnet | Mechanical extraction |
| Ask user questions, collect input | sonnet | No reasoning needed |
| Consult API references, summarize | sonnet | Read + filter |
| Design architecture, make trade-offs | **opus** | Complex reasoning |
| Generate code (models, views, security) | **opus** | Correctness matters |
| Plan mode, collaborative design | **opus** | Creativity + judgment |
| Validate naming, imports, structure | sonnet | Rule-based checks |
| Run scripts (audit, catalog, lint) | sonnet | Execute + report |
| Git operations (commit, push, branch) | sonnet | Mechanical |
| Docker operations | sonnet | Mechanical |
| Delegate to other skills | sonnet | Orchestration only |

**Rule of thumb:** If the task is "follow rules and report", use sonnet. If the task is "decide what's right", use opus.

## How to Write the SKILL.md

### Step 1: Classify each phase

In your workflow, tag each step with its model:

```markdown
### Step 0 — Gather context [SONNET]
Launch Agent(model: sonnet) to:
- Read PROJECT_REGISTRY.md if exists
- Collect Odoo version, module name, description from user
- Return structured JSON with all collected info

### Step 1 — Design architecture [OPUS — runs in principal]
Using the context from Step 0:
- Analyze requirements against API reference
- Propose models, fields, views, security
- [STOP] — present plan to user

### Step 2 — Generate module [OPUS — runs in principal]
Write all files following the approved plan.

### Step 3 — Validate [SONNET]
Launch Agent(model: sonnet) to:
- Check naming conventions
- Verify security rules exist for all company_id models
- Validate imports order
- Report issues found
```

### Step 2: Write explicit Agent instructions

Don't just say "use sonnet". Write the exact Agent call pattern:

```markdown
### Step 0 — Gather context [SONNET]

Launch a subagent:
- **model:** sonnet
- **subagent_type:** general-purpose
- **description:** "gather module requirements"
- **prompt must include:**
  1. The working directory (cwd)
  2. What to read: PROJECT_REGISTRY.md, __manifest__.py of related modules
  3. What to ask the user (using AskUserQuestion)
  4. Expected output format (structured data the principal agent will use)

The subagent returns a structured summary. The principal agent continues from Step 1.
```

### Step 3: Define the handoff contract

Each subagent must know:
- **Input:** what context it receives (cwd, user request, prior step output)
- **Output:** what it must return (structured data, not prose)
- **Boundaries:** what it must NOT do (no code generation in sonnet steps)

```markdown
#### Subagent contract — Step 0
**Input:** cwd, user's original request
**Output:** JSON with keys: odoo_version, module_name, description, approach (A/B/C), paths, multi_company
**Must NOT:** generate code, make architecture decisions, skip AskUserQuestion
```

## Effort Classification

Skills declare `effort` in frontmatter. This maps to the default model:

| Effort | Default model | Examples |
|--------|--------------|----------|
| low | sonnet (full skill) | /git, /docker, module-scanner |
| medium | opus orchestrator + sonnet subagents | module-creator, code-auditor |
| high | opus (full skill) | database-upgrader, flow-tester |

For **low effort** skills, the ENTIRE skill runs as a sonnet subagent:

```markdown
# Git Mode

**OBLIGATORIO:** Launch Agent(model: sonnet) with the user's request.
Do NOT execute git commands in the principal conversation.
```

For **medium effort** skills, use the orchestrator pattern described above.

For **high effort** skills, opus runs everything because every step requires deep reasoning.

## Description Field and Model Routing

The `description` field determines WHEN the skill triggers. It must also hint at the routing:

```yaml
# Low effort — entire skill is sonnet
description: >
  Gestiona operaciones git con Sonnet: commits, push, pull...
model: sonnet

# Medium effort — orchestrator with mixed models  
description: >
  Create new Odoo modules (v14-19). Gathers requirements (sonnet),
  designs architecture and generates code (opus), validates output (sonnet).
model: opus

# High effort — all opus
description: >
  Migrate Odoo databases between versions via Upgrade Service.
  Multi-phase process requiring deep analysis at every step.
model: opus
```

## Standard Delegation Patterns (real Agent() calls)

Most delegations are just Read/Bash inline — they don't need Agent(). Only **execution delegations** need real Agent() calls. Three patterns cover 90% of cases:

**CRITICAL:** All Agent() prompts MUST use skill names, NEVER absolute paths. Claude discovers installed skills automatically. This makes prompts portable across machines.

### Pattern A: "Scan First" — invoke module-scanner before acting

Used by 12+ skills. Launch sonnet subagent to scan a module and return structured inventory.

**Context to inject:** path + Odoo version + specific focus areas for the consuming skill's needs.

```markdown
### Step N — Scan module [MANDATORY SUBAGENT]

Launch a subagent:
- **model:** sonnet
- **subagent_type:** general-purpose
- **description:** "scan {tech_name} for inventory"
- **prompt:** "Scan the Odoo {version} module at {path}. Return a JSON inventory
  with keys: models (name, _inherit, fields with type/relation/required/compute),
  views (name, model, type, inherit_id), security (ACL lines + ir.rule records),
  controllers (routes), js_components (names).
  {Add skill-specific focus: e.g. 'Flag raw SQL usage and deprecated API calls' for migration,
  'Read i18n/{lang}.po and return translated labels' for documentation.}
  Only scan and report — do NOT modify files."

Store result as `SCAN_RESULT`. Do NOT proceed without it.
```

### Pattern B: "Read API Ref" — fetch version-specific API summary

Used by 5+ skills. Launch sonnet subagent to load version-api-reference and return only the relevant parts.

**Context to inject:** version + specific topics + whether modules have JS/OWL components.

```markdown
### Step N — Load API reference [MANDATORY SUBAGENT]

Launch a subagent:
- **model:** sonnet
- **subagent_type:** general-purpose
- **description:** "load API ref for v{version}"
- **prompt:** "Load the Odoo API reference for version {version}. Return a summary of:
  field parameters, engine patterns (precompute, indexes), and gotchas relevant to
  {topics}. {If source version known: 'Focus on breaking changes from v{source}.'} 
  {If modules have JS: 'Include OWL/JS framework changes.'}"

Store result as `API_RESULT`. Do NOT write code without it.
```

### Pattern C: "Post-action validation" — invoke quality-gate or test-generator

Used after code generation. Launch sonnet subagent to validate or generate tests.

**Context to inject:** path + version + module purpose + architecture decisions from earlier steps + original requirements. Post-action subagents need the MOST context because they validate against decisions only the orchestrator knows.

**CRITICAL:** Never reference orchestrator variables (SCAN_RESULT, API_RESULT) in the subagent prompt — the subagent can't see them. Extract and inline the relevant data.

```markdown
### Step N — Validate [OPTIONAL SUBAGENT] — suggest to user

Launch a subagent:
- **model:** sonnet
- **subagent_type:** general-purpose
- **description:** "run quality gate on {tech_name}"
- **prompt:** "Run a quality gate review on the Odoo {version} module at {path}.
  Module purpose: {description from STEP 0}.
  Architecture: {summary from design step — models, multi-company classification, deps}.
  Requirements: {user requirements}.
  Run all 5 gates. Return PASS/WARN/FAIL per gate with evidence."
```

### When NOT to use Agent()

| Delegation type | Example | Use Agent()? |
|----------------|---------|-------------|
| Read a reference file | → effort-estimator tables | **No** — just Read the file inline |
| Execute a shell script | → odoo-pilot/scripts/auth.sh | **No** — just Bash inline |
| Read brand tokens | → dynapps-brand/references/brand-system.md | **No** — just Read inline |
| Ask user a question | → onboarding credentials | **No** — AskUserQuestion inline |
| Scan a module | → module-scanner | **Yes** — Pattern A |
| Summarize 29 API files | → version-api-reference | **Yes** — Pattern B |
| Run quality review | → quality-gate | **Yes** — Pattern C |
| Generate tests | → test-generator | **Yes** — Pattern C |

## Persistent Subagents (`.claude/agents/`)

Beyond inline `Agent()` calls in workflows, Claude Code supports **persistent subagent definitions** as markdown files in `.claude/agents/`. These are reusable across skills and sessions.

### When to use persistent agents vs inline Agent()

| Scenario | Use |
|----------|-----|
| Skill-specific delegation (scan, validate) | Inline `Agent()` in SKILL.md workflow |
| Reusable specialist across multiple skills | Persistent `.claude/agents/specialist.md` |
| Security reviewer, performance auditor | Persistent — shared concern, not skill-specific |
| One-off data gathering | Inline `Agent()` |

### Format
```markdown
# .claude/agents/security-reviewer.md
---
name: security-reviewer
description: Reviews code for security vulnerabilities
tools: Read, Grep, Glob, Bash
model: opus
---
You are a senior security engineer. Review code for:
- Injection vulnerabilities (SQL, XSS, command injection)
- Authentication and authorization flaws
- Secrets or credentials in code
```

### Design consideration for skill-designer
When designing a new skill, check if the validation/review step would benefit OTHER skills too. If yes → create a persistent agent instead of an inline Agent() call. If it's skill-specific → keep it inline.

## Anti-Patterns

### 1. The All-Opus Skill
**Problem:** Every step runs in opus, including file reads and validation.
**Fix:** Identify mechanical phases and delegate to sonnet subagents.

### 2. The Fake Sonnet Skill
**Problem:** `model: sonnet` in frontmatter but no Agent instruction. Runs in whatever model the session uses.
**Fix:** Add explicit `Agent(model: sonnet)` instruction in the workflow.

### 3. The Blind Handoff
**Problem:** Subagent prompt says "invoke skill X on path" without injecting the context the subagent needs. The subagent starts with ZERO context — it can't see orchestrator variables, prior step results, or user requirements.
**Fix:** Every subagent prompt must be self-contained. Inject: version, module purpose, architecture decisions, and any prior-step output the subagent needs. Never reference orchestrator variables (SCAN_RESULT, API_RESULT) — inline the data instead. Define input/output contract. Subagent returns structured data, not a narrative.

### 4. The Opus Validator
**Problem:** Opus runs mechanical validation (naming, imports, structure checks).
**Fix:** Launch sonnet subagent for validation. Opus reviews only if issues require judgment.

## Cost Impact

Typical savings from proper routing on a medium-effort skill:

| Phase | Tokens | Opus cost | With routing |
|-------|--------|-----------|-------------|
| Gather info | ~3,000 | opus | → sonnet (70% cheaper) |
| Design | ~8,000 | opus | opus (no change) |
| Generate | ~12,000 | opus | opus (no change) |
| Validate | ~4,000 | opus | → sonnet (70% cheaper) |
| **Total** | ~27,000 | 100% opus | **~26% savings** |

For low-effort skills (/git, /docker), routing to sonnet saves **~70% per invocation**.
