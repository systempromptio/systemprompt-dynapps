# Skill Categories

A skill should fit ONE category. If it straddles several, split it.

## 1. Library & API Reference
**Purpose:** Teach Claude how to correctly use a library, CLI, or SDK.
**Key content:** Function signatures, edge cases, footguns, code snippets.
**Structure:** SKILL.md (overview + gotchas) + references/ (API docs, examples).
**Signal:** Someone keeps correcting Claude about how to use a specific tool.
**Example skills:** billing-lib, internal-cli, frontend-design-system.
**Indaws example:** odoo-version-api-reference (API changes between Odoo 14-19).

## 2. Product Verification
**Purpose:** Test or verify that code/system is working correctly.
**Key content:** Test scripts, assertion patterns, verification workflows.
**Structure:** SKILL.md (workflow) + scripts/ (test runners, verification tools) + templates/ (report format).
**Signal:** You need to prove something works, not just build it.
**Techniques:** Record video of output, programmatic assertions at each step, structured reports.
**Example skills:** signup-flow-driver, checkout-verifier, tmux-cli-driver.
**Indaws example:** odoo-flow-tester/migration_tester (195 tests, 4 types, branded PDF report).

## 3. Data Fetching & Analysis
**Purpose:** Connect to data/monitoring stacks and analyze.
**Key content:** Credentials, query patterns, dashboard IDs, common workflows.
**Structure:** SKILL.md (workflows) + scripts/ (query helpers, fetchers) + references/ (schema docs).
**Signal:** Someone repeatedly asks "how do I get data about X?"
**Example skills:** funnel-query, cohort-compare, grafana-lookup.
**Indaws example:** odoo-pilot (CRUD operations on any Odoo model via API).

## 4. Business Process Automation
**Purpose:** Automate repetitive workflows into one command.
**Key content:** Instructions + dependencies on other skills/MCPs.
**Structure:** SKILL.md (workflow) + config.json (user settings) + logs/ (execution history).
**Signal:** Someone does the same multi-step task weekly.
**Tip:** Store previous results in log files for consistency between executions.
**Example skills:** standup-post, create-ticket, weekly-recap.
**Indaws example:** dev-assistant (daily planning, timesheet report), timesheet-report.

## 5. Code Scaffolding & Templates
**Purpose:** Generate boilerplate for specific codebase patterns.
**Key content:** Templates, naming conventions, configuration defaults.
**Structure:** SKILL.md (when to use) + templates/ (file templates) + scripts/ (generators).
**Signal:** New code always follows the same pattern with small variations.
**Example skills:** new-workflow, new-migration, create-app.
**Indaws example:** odoo-module-creator, odoo-carrier-creator.

## 6. Code Quality & Review
**Purpose:** Enforce code quality and review standards.
**Key content:** Style rules, testing practices, review checklists.
**Structure:** SKILL.md (rules) + scripts/ (linters, analyzers) + references/ (examples).
**Signal:** Reviews keep catching the same issues.
**Tip:** Can run automatically via hooks or GitHub Actions.
**Example skills:** adversarial-review, code-style, testing-practices.
**Indaws example:** quality-gate (5 gates), odoo-code-auditor.

## 7. CI/CD & Deployment
**Purpose:** Fetch, push, deploy, and monitor releases.
**Key content:** Deployment procedures, rollback protocols, smoke tests.
**Structure:** SKILL.md (procedures) + scripts/ (deploy, rollback, monitor).
**Signal:** Deployments involve manual steps that could go wrong.
**Example skills:** babysit-pr, deploy-service, cherry-pick-prod.

## 8. Runbooks
**Purpose:** Given a symptom, investigate and produce structured report.
**Key content:** Symptom→tool→query mapping, investigation workflows.
**Structure:** SKILL.md (symptom catalog) + scripts/ (diagnostic tools) + templates/ (report format).
**Signal:** Oncall keeps following the same investigation steps.
**Example skills:** service-debugging, oncall-runner, log-correlator.
**Indaws example:** odoo-database-analyzer (migration complexity assessment).

## 9. Infrastructure Operations
**Purpose:** Routine maintenance with guardrails for destructive actions.
**Key content:** Cleanup procedures, approval workflows, cost analysis.
**Structure:** SKILL.md (procedures) + scripts/ (cleanup, audit) + references/ (thresholds).
**Signal:** Operations require careful steps that benefit from automation + safety.
**Example skills:** orphan-cleanup, dependency-management, cost-investigation.
**Indaws example:** odoo-database-upgrader (6 phases, iterative SQL fixes).
