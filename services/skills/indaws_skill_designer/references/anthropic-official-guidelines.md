# Anthropic Official Skill Guidelines (2025)

Source: https://platform.claude.com/docs/en/agents-and-tools/agent-skills/best-practices
Source: https://platform.claude.com/docs/en/agents-and-tools/agent-skills/enterprise

Read this BEFORE designing, auditing, or optimizing any skill. These are Anthropic's official recommendations.

## Contents
- Core principles (conciseness, degrees of freedom, testing)
- Naming and description rules
- Progressive disclosure patterns
- Anti-patterns to avoid
- Enterprise governance
- Checklist

---

## 1. Core Principles

### Concise is key
The context window is a shared resource. Challenge each piece of information:
- "Does Claude really need this explanation?"
- "Can I assume Claude knows this?"
- "Does this paragraph justify its token cost?"

**Default assumption**: Claude is already very smart. Only add context Claude doesn't already have.

### Degrees of freedom
Match specificity to task fragility:

| Level | Use when | Example |
|-------|----------|---------|
| **High** (text instructions) | Multiple approaches valid, context-dependent | Code review guidelines |
| **Medium** (pseudocode/scripts with params) | Preferred pattern exists, some variation OK | Report generator with format param |
| **Low** (exact scripts, no params) | Fragile operations, consistency critical | Database migration command |

**Analogy**: Narrow bridge = low freedom. Open field = high freedom.

### Test with all target models
- **Haiku**: Does the skill provide enough guidance?
- **Sonnet**: Is it clear and efficient?
- **Opus**: Does it avoid over-explaining?

---

## 2. Naming Rules

### Format
- Max 64 chars, lowercase + numbers + hyphens only
- No XML tags, no "anthropic" or "claude" reserved words

### Prefer gerund form
Good: `processing-pdfs`, `analyzing-spreadsheets`, `managing-databases`
Acceptable: `pdf-processing`, `process-pdfs`
Avoid: `helper`, `utils`, `tools`, `documents`

---

## 3. Description Rules (CRITICAL for triggering)

### Always third person
- Good: "Processes Excel files and generates reports"
- Bad: "I can help you process Excel files"
- Bad: "You can use this to process Excel files"

### Be specific with key terms
Include WHAT + WHEN to use it. The description is what Claude uses to choose from 100+ skills.

Good:
```yaml
description: "Extract text and tables from PDF files, fill forms, merge documents. Use when working with PDF files or when the user mentions PDFs, forms, or document extraction."
```

Bad:
```yaml
description: "Helps with documents"
```

### Max 1024 characters, non-empty, no XML tags

---

## 4. Progressive Disclosure Patterns

### Three levels
1. **Metadata** (name + description) — always in context (~100 words)
2. **SKILL.md body** — loaded when triggered (<500 lines)
3. **Bundled resources** — loaded on demand (unlimited)

### Pattern 1: High-level guide with references
```markdown
# PDF Processing
## Quick start
[inline code example]
## Advanced features
**Form filling**: See [FORMS.md](FORMS.md)
**API reference**: See [REFERENCE.md](REFERENCE.md)
```

### Pattern 2: Domain-specific organization
```
bigquery-skill/
├── SKILL.md (overview + navigation)
└── reference/
    ├── finance.md
    ├── sales.md
    └── product.md
```

### Pattern 3: Conditional details
```markdown
## Basic: modify XML directly
## For tracked changes: See [REDLINING.md](REDLINING.md)
```

### CRITICAL: Max 1 level deep references
Bad: SKILL.md → advanced.md → details.md (Claude may partially read)
Good: SKILL.md → advanced.md, SKILL.md → reference.md (all direct)

### TOC for files >100 lines
```markdown
## Contents
- Authentication and setup
- Core methods (CRUD)
- Error handling
```

---

## 5. Workflows and Feedback Loops

### Use checklists for complex tasks
```markdown
Task Progress:
- [ ] Step 1: Analyze
- [ ] Step 2: Create plan
- [ ] Step 3: Validate plan
- [ ] Step 4: Execute
- [ ] Step 5: Verify
```

### Feedback loops (high value pattern)
Run validator → fix errors → repeat. Catches errors early.

### Plan-validate-execute pattern
For complex/destructive operations: create intermediate plan file → validate with script → execute only if valid.

---

## 6. Frontmatter: Real vs Ghost Fields

### `disable-model-invocation: true`
Prevents the skill from auto-activating via model matching. The skill can ONLY be invoked explicitly via `/skill-name`. Use for:
- Workflow skills with side-effects (e.g., `/fix-issue`, `/revamp`)
- Skills that modify external systems (git push, API writes, deployments)
- Skills where accidental invocation could be destructive

```yaml
---
name: fix-issue
description: Analyze and fix a GitHub issue end-to-end
disable-model-invocation: true
---
```

### Ghost fields (NO effect — do NOT use)
Parsed and silently dropped: `activation`, `tags`, `globs`, `triggers`, `requires`, `references`,
`tools` (that one is an **agent** field), `compatibility` (an MCPB/DXT manifest field). Two of them
have a real counterpart and that is the point of naming them: `globs` → **`paths`**, `triggers` →
**`when_to_use`**. The loader's own validator says it plainly: *"Claude Code ignores unrecognized
fields at load time."*

### Real fields summary — SKILL.md

Not folklore: this is the zod schema the Claude Code skill loader validates against, read out of
the binary. **Verified against 2.1.252** — it is versioned, so re-derive before trusting it on a
newer build. `scripts/frontmatter_keys.py` holds the machine-readable copy, the re-derivation
command, and is the single list every gate imports.

| Field | Effect | Required |
|-------|--------|----------|
| `name` | Display name. Defaults to the filename | Yes |
| `description` | Trigger matching (the model reads this to decide) | Yes |
| `allowed-tools` / `disallowed-tools` | Tools added/removed while the skill is active | No |
| `model` | `haiku`/`sonnet`/`opus`/`fable`/full ID, or `inherit` | No |
| `effort` | **Real runtime parameter**: `low`/`medium`/`high`/`xhigh`/`max` or an integer. An invalid value warns at load | No |
| `version` | Read, but marked `@internal` — bookkeeping, never shown to users | No |
| `argument-hint` | Placeholder shown after the slash-command name | No |
| `disable-model-invocation` | Only the user may type `/name`; the model cannot invoke it | No |
| `user-invocable` | `false` hides the slash command; only the model may invoke it | No |
| `when_to_use` | **Becomes part of the tool description.** Not one of the 212 ecosystem skills uses it | No |
| `paths` | Globs; the skill loads only when the model touches matching files | No |
| `context` | `inline` (expand here) or `fork` (spawn a subagent) | No |
| `agent` / `background` | Agent type for `context: fork`, and whether the fork blocks the turn | No |
| `hooks` | Hooks registered while the skill is active (settings.json shape) | No |
| `shell` | `bash` or `powershell` for `!`-command blocks | No |
| `metadata` | Free-form map for the author, preserved on the loaded skill | No |
| `license` | Read, `@internal` | No |

## 6. Anti-Patterns

- **Windows paths**: Always use forward slashes, even on Windows
- **Too many options**: Provide ONE default approach + escape hatch, not 5 alternatives
- **Time-sensitive info**: Use "old patterns" collapsed section instead of dates
- **Inconsistent terminology**: Pick one term and use it everywhere
- **Deeply nested references**: Max 1 level from SKILL.md
- **Assuming tools installed**: Always include install instructions
- **Vague descriptions**: "Helps with documents" tells Claude nothing

---

## 7. Enterprise Governance

### Risk tier assessment for Skills with code

| Indicator | Concern |
|-----------|---------|
| Scripts (*.py, *.sh) | HIGH: full environment access |
| Instruction manipulation | HIGH: bypass safety |
| MCP server references | HIGH: extends access |
| Network access (URLs, fetch, curl) | HIGH: data exfiltration |
| Hardcoded credentials | HIGH: secrets exposed |
| File system scope (../, broad globs) | MEDIUM |
| Tool invocations (bash, file ops) | MEDIUM |

### Security review checklist
1. Read ALL skill directory content
2. Verify script behavior matches stated purpose
3. Check for adversarial instructions
4. Check for network calls
5. Verify no hardcoded credentials
6. List all tool invocations
7. Confirm redirect destinations
8. Verify no data exfiltration patterns

### Evaluation dimensions

| Dimension | What it measures |
|-----------|-----------------|
| Triggering accuracy | Activates for right queries, stays inactive for wrong ones |
| Isolation behavior | Works correctly alone |
| Coexistence | Doesn't degrade other skills |
| Instruction following | Claude follows the skill accurately |
| Output quality | Correct, useful results |

### Recall limits
- Max ~8 skills per API request
- Too many active skills = Claude fails to select the right one
- Measure recall accuracy as you add skills, stop when it degrades

### Role-based bundles
Group skills by role: Sales team, Engineering, Finance, etc.

### Lifecycle: Plan → Create → Test → Deploy → Monitor → Iterate/Deprecate

---

## 8. MCP Tool References

Always use fully qualified names: `ServerName:tool_name`
```markdown
Use the BigQuery:bigquery_schema tool to retrieve schemas.
```

---

## 9. Final Checklist

### Core quality
- [ ] Description specific, includes key terms, third person
- [ ] SKILL.md body under 500 lines
- [ ] Additional details in separate files
- [ ] No time-sensitive information
- [ ] Consistent terminology
- [ ] Concrete examples (not abstract)
- [ ] File references max 1 level deep
- [ ] Progressive disclosure used
- [ ] Workflows have clear steps

### Code and scripts
- [ ] Scripts handle errors (don't punt to Claude)
- [ ] No magic constants (all values justified)
- [ ] Required packages listed
- [ ] No Windows-style paths
- [ ] Validation/verification for critical ops
- [ ] Feedback loops for quality-critical tasks

### Testing
- [ ] 3-5 representative eval queries
- [ ] Tested with target models (Haiku/Sonnet/Opus)
- [ ] Tested with real scenarios
- [ ] Team feedback incorporated
