# In-Skill Workflow Patterns

How to STRUCTURE the workflow *inside* a skill's SKILL.md. This is a different axis from `tool-design-patterns.md` (which is about designing the tools/skills themselves) and from `agent-routing-patterns.md` (which is about sonnet/opus subagent orchestration). These five are the shapes a skill's body takes, drawn from Anthropic's "Complete Guide to Building Skills" + observed Indaws skills. Pick the one that matches the task; compose when needed.

First decide the framing (see best-practices.md → Problem-First vs Tool-First), then pick a pattern.

## Pattern 1: Sequential workflow orchestration
**Use when:** users need a multi-step process in a specific order.
**Shape:** numbered `### Step N` blocks, each naming the call + parameters, with explicit dependencies between steps and a validation gate at each stage. Include rollback instructions for failures.
**Key techniques:** explicit step ordering · dependencies between steps · validation at each stage · rollback on failure.
**Indaws fit:** onboarding flows, migration playbooks (the 5-gate migration template), order-to-cash style processes.

## Pattern 2: Multi-MCP / multi-service coordination
**Use when:** the workflow spans multiple services/connectors.
**Shape:** `### Phase N (Service)` blocks. Each phase completes against one service, passes data forward, and validates before the next phase starts. Centralize error handling.
**Key techniques:** clear phase separation · data passing between services · validate before advancing · centralized error handling.
**Indaws fit:** design-to-dev handoffs, anything chaining Odoo + an external API + a notification.

## Pattern 3: Iterative refinement
**Use when:** output quality improves with iteration (reports, generated docs, designs).
**Shape:** Initial draft → Quality check (run a validation script, list concrete issues) → Refinement loop (fix each issue, re-validate) → Finalize. State explicit quality criteria AND a stop condition.
**Key techniques:** explicit quality criteria · validation scripts · iterate · know when to stop.
**Indaws fit:** report generation, docx-template output, anything graded against a rubric.

## Pattern 4: Context-aware tool selection
**Use when:** same outcome, different tool depending on context.
**Shape:** a `### Decision Tree` mapping conditions → tool, then `### Execute` using the chosen tool, then a `### Explain` step that tells the user WHY that path was chosen (transparency). Always provide a fallback branch.
**Key techniques:** clear decision criteria · fallback options · transparency about the choice.
**Indaws fit:** "store this file" (size/type → location), "classify this request" (Standard vs Studio vs Custom in dev-assessment).

## Pattern 5: Domain-specific intelligence
**Use when:** the skill's value is specialized knowledge layered on top of plain tool access.
**Shape:** embed the domain rules in the logic — checks/compliance BEFORE the action, the action itself, then an audit trail. Document the governance clearly.
**Key techniques:** domain expertise in the logic · checks before action · comprehensive documentation · clear governance.
**Indaws fit:** accounting-auditor, indaws-core-auditor, payroll-accounting — where the rules, not the API call, are the product.

---

These are starting shapes, not a taxonomy to force-fit. A real skill often combines them — e.g. a migration skill is Sequential (Pattern 1) with an Iterative validation gate (Pattern 3) at each step. Keep the body lean: the pattern is scaffolding, the gotchas and domain rules are the value.
