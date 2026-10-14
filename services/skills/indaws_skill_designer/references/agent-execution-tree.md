# Agent Execution Tree

How to design the real agent tree for any skill. Every skill must have a concrete execution plan showing which Agent() calls happen, in what order, with what model.

## CRITICAL: Portable Agent() Prompts

**NEVER hardcode absolute paths** in Agent() prompts. Claude already knows all installed skills — just reference them by name.

```markdown
# ❌ WRONG — breaks on other machines
"Read $SKILLS_ROOT/ai-claude-common/skills/odoo-module-scanner/SKILL.md. Follow its workflow..."

# ✅ CORRECT — portable, works everywhere
"Invoke the skill 'odoo-module-scanner' on the module at {path}. Return JSON with..."
```

**Why:** Skill paths differ per user *and per agent* (`$SKILLS_ROOT/` vs `~/.claude/skills/` under Claude Code vs `~/.config/opencode/skills/` under opencode vs the marketplace cache). Using names makes prompts machine- and agent-independent; the agent discovers installed skills automatically. When a script genuinely needs a path, resolve it via `references/shared/skill-path-resolution.md` instead of hardcoding one.

**Also:** `effort` is NOT a real Agent() parameter — it's frontmatter metadata. To hint at effort level for a subagent, encode it in the prompt text.

## CRITICAL: Context-Rich Prompts

**Subagents start with ZERO context** from the orchestrator conversation. They don't see prior steps, stored variables, or user requirements. The prompt is ALL they get.

### The #1 mistake: "invoke skill X on path"

```markdown
# ❌ WRONG — subagent has no context
"Invoke the skill 'quality-gate' on the module at {path}. Run 5 gates. Return PASS/WARN/FAIL."

# ✅ CORRECT — subagent has everything it needs
"Run a quality gate review on the Odoo {version} module at {path}.
Module purpose: {description from STEP 0}.
Architecture decisions: {summary from STEP 1c — models, multi-company classification, deps}.
Original requirements: {user requirements from STEP 0}.
Run all 5 gates. Return PASS/WARN/FAIL per gate with evidence."
```

### What context to inject per pattern

| Pattern | Minimum context the subagent needs |
|---------|-----------------------------------|
| A (scan) | Path + Odoo version + what to focus on (fields? deprecated APIs? complexity?) |
| B (API ref) | Version + specific topics + whether modules have JS components |
| C (post-action) | Path + version + what was created/modified + architecture decisions + requirements |

### Rule: if the orchestrator knows it, and the subagent needs it, inject it

Ask yourself for each `{variable}` in the prompt: "Does the subagent have access to this?" If the answer is "only if the orchestrator passes it" → it MUST be in the prompt.

**Common failures:**
- "Use SCAN_RESULT" → subagent can't see orchestrator variables. Inline the relevant data.
- "Check correct tags for version" → subagent doesn't know the version. Pass `{version}`.
- "Validate against requirements" → subagent doesn't know what was required. Pass `{requirements summary}`.
- "Use SCAN inventory" → subagent 3c can't access subagent 3a's output. The orchestrator must extract and pass the relevant parts.

## The 3 Agent() Patterns

### Pattern A: "Scan First" — module-scanner
```
Agent(model: sonnet, subagent_type: general-purpose, description: "scan {tech_name} for inventory")
  Input: module path + Odoo version + focus areas
  Invokes: skill 'odoo-module-scanner' by name
  Output: {models, fields, views, security, controllers, JS}
  Cost: ~2-3K tokens sonnet
```
**Used by:** 12 skills. Always before: creating, extending, migrating, auditing, testing, documenting a module.

### Pattern B: "Read API Ref" — version-api-reference
```
Agent(model: sonnet, subagent_type: general-purpose, description: "load API ref v{version}")
  Input: version + topics + whether modules have JS/OWL components
  Invokes: skill 'odoo-version-api-reference' by name
  Output: filtered API summary for target version
  Cost: ~3-5K tokens sonnet
```
**Used by:** 5 skills. Always before: writing Odoo code for a specific version.

### Pattern C: "Post-action" — quality-gate / test-generator / flow-tester
```
Agent(model: sonnet, subagent_type: general-purpose, description: "run quality gate on {tech_name}")
  Input: path + version + module purpose + architecture decisions + requirements
  Invokes: skill 'quality-gate' or 'odoo-test-generator' by name
  Output: structured report (PASS/WARN/FAIL) or test files
  Cost: ~4-6K tokens sonnet
```
**Used by:** after code generation. Usually optional (suggest to user).
**CRITICAL:** Post-action subagents need the MOST context because they must validate against decisions made in earlier steps that only the orchestrator knows.

## NOT an Agent() — inline operations

| Operation | How to execute | Example |
|-----------|---------------|---------|
| Read a reference file | `Read references/file.md` | effort-estimator tables |
| Run odoo-pilot script | `Bash odoo-pilot/scripts/auth.sh` | API connection |
| Read brand tokens | `Read dynapps-brand/references/brand-system.md` | Colors, fonts |
| Ask user question | `AskUserQuestion` tool | Collect requirements |
| Read shared patterns | `Read references/shared/odoo-connection.md` | Connection pattern |

**Rule:** If the delegation just reads files or runs a shell script, do it inline. Only use Agent() when the delegation requires reading another skill's SKILL.md and following its workflow.

## How to build the tree for a new skill

### Step 1: List all delegations from "Delegates to" table

```
module-creator delegates to:
  → version-api-reference (common)
  → module-scanner (common)
  → test-generator (developer)
  → quality-gate (common)
```

### Step 2: Classify each delegation

| Delegation | Read/Bash? | Follows a workflow? | Agent()? |
|-----------|-----------|-------------------|----------|
| version-api-reference | Reads 29 files, needs filtering | Yes (select by version+topic) | **Pattern B** |
| module-scanner | Scans module structure | Yes (full scan workflow) | **Pattern A** |
| test-generator | Generates test files | Yes (full gen workflow) | **Pattern C** |
| quality-gate | Evaluates 5 gates | Yes (review workflow) | **Pattern C** |

### Step 3: Place each Agent() in the workflow

```
Step 0 — Gather: inline (Read registry, AskUser)
Step 1 — Analyze:
  → Agent 1: Pattern A (scan module to extend)
  → Agent 2: Pattern B (load API for target version)
  → Opus: design architecture using Agent 1+2 output
Step 2 — Generate: opus writes code
Step 3 — Validate:
  → Agent 3: Pattern A (scan created module)
  → Opus: review scan results
  → Agent 4: Pattern C quality-gate (optional)
  → Agent 5: Pattern C test-generator (optional)
```

### Step 4: Estimate cost

| Phase | Agents | Sonnet tokens | Opus tokens |
|-------|--------|--------------|-------------|
| Gather | 0 | 0 | ~1K |
| Analyze | 2 sonnet + opus | ~8K | ~8K |
| Generate | 0 | 0 | ~12K |
| Validate | 1-3 sonnet + opus | ~2-12K | ~3K |
| **Total** | **3-5** | **~10-20K** | **~24K** |

## Effort → typical agent count

| Effort | Agents | Example |
|--------|--------|---------|
| low | 0 | brand skills, effort-estimator, classification-guide |
| low (connector) | 0 | odoo-pilot (runs scripts inline) |
| medium | 1-2 | module-documenter (1 scan), code-auditor (1 scan + 1 API) |
| high | 3-5 | module-creator (2 scan + 1 API + 2 optional post-action) |

## Template for SKILL.md workflow with Agent() calls

**IMPORTANT:** All Agent() prompts MUST use skill names, never absolute paths. All prompts MUST inject the context the subagent needs — it sees NOTHING from the orchestrator conversation.

```markdown
## Workflow

### Step 0 — Gather [STOP]
Read PROJECT_REGISTRY.md. Use AskUserQuestion for missing info.

### Step 1 — Analyze [STOP]

**NEVER** scan modules or read API references yourself. Always delegate to subagents.

**1a. Scan module [MANDATORY SUBAGENT]**

Launch a subagent:
- **model:** sonnet
- **subagent_type:** general-purpose
- **description:** "scan {tech_name} for inventory"
- **prompt:** "Scan the Odoo {version} module at {path}. Return a JSON inventory
  with keys: models (name, _inherit, fields with type/relation/required/compute),
  views (name, model, type, inherit_id), security (ACL lines + ir.rule records),
  controllers (routes), js_components (names).
  Only scan and report — do NOT modify files."

Store result as `SCAN_RESULT`. Do NOT design architecture without it.

**1b. Load API reference [MANDATORY SUBAGENT]**

Launch a subagent:
- **model:** sonnet
- **subagent_type:** general-purpose
- **description:** "load API ref for v{version}"
- **prompt:** "Load the Odoo API reference for version {version}. Return a summary of:
  field parameters, engine patterns (precompute, indexes), and gotchas relevant to
  {topics}. {If modules have JS components: 'Include OWL/JS framework changes.'}"

Store result as `API_RESULT`. Do NOT write code without it.

**1c. Design architecture [OPUS — this agent]**
Using SCAN_RESULT + API_RESULT + local references, propose plan.
[STOP — wait for user confirmation]

### Step 2 — Execute [OPUS]
Generate/modify code using plan + references.

### Step 3 — Validate

**3a. Scan result [MANDATORY SUBAGENT]**

Launch a subagent:
- **model:** sonnet
- **subagent_type:** general-purpose
- **description:** "validate {tech_name}"
- **prompt:** "Scan the Odoo {version} module at {created_path} and validate it.
  Return a JSON inventory (models, fields, views, security, controllers, js_components).
  Then verify:
  1. All models are imported in __init__.py (expected: {list of model names from STEP 1c})
  2. Every model with company_id has a matching ir.rule (multi-company: {classification per model})
  3. Views use correct XML tags for v{version}
  4. Manifest depends list matches actual imports
  Report: inventory + PASS/FAIL per check with failure details."

Store result as `VALIDATION_RESULT`. Review for completeness.

**3b. Quality gate [OPTIONAL SUBAGENT]** — suggest to user

Launch a subagent:
- **model:** sonnet
- **subagent_type:** general-purpose
- **description:** "run quality gate on {tech_name}"
- **prompt:** "Run a quality gate review on the Odoo {version} module at {created_path}.
  Module purpose: {description from STEP 0}.
  Architecture: {summary from STEP 1c — models, multi-company, deps}.
  Requirements: {user requirements from STEP 0}.
  Run all 5 gates. Return PASS/WARN/FAIL per gate with evidence."
```
