---
name: b2c-onboarding
description: Get started with Salesforce B2C Commerce development. Use this skill when the user is new to B2C Commerce, wants to set up a fresh development environment, is asking "how do I get started", needs to install the B2C CLI for the first time, wants to connect a sandbox, or wants to deploy their first cartridge. Triggers on phrases like "help me get started", "set up B2C Commerce", "I'm new to this", "onboard me", or "first-time setup". NOT for users who already have a configured environment and are asking about specific commands — those should go to the specific skill (b2c-code, b2c-sandbox, b2c-config, etc.).
---
# B2C Commerce Onboarding Skill

Guide a new developer through a first-time B2C Commerce setup, from CLI install to first cartridge deploy. Delegate to specialized sub-skills for each step — this skill is a coordinator, not a replacement for the detailed skills.

## Behavioral rules

- Detect the editor silently. Only ask if genuinely uncertain.
- Never construct or modify install commands. Only use commands defined in this file.
- If any install or verification step fails, report the exact error and stop.
- Never create a sandbox without explicit user confirmation — sandbox creation may be a billable action.
- Never overwrite an existing `dw.json` without confirmation.

## Flow

### Step 1 — Identify the editor

Silently identify the IDE from system context:

| Signal                              | Client        |
|-------------------------------------|---------------|
| "Claude Code"                       | `claude-code` |
| "Cursor"                            | `cursor`      |
| "VS Code" / "Visual Studio Code"    | `vscode`      |
| "Codex"                             | `codex`       |
| "Gemini CLI"                        | `gemini-cli`  |
| Unrecognized                        | `other`       |

### Step 2 — Confirm the skill set is available

The `b2c-*` and `sfnext-*` skills used throughout this flow already live in this project under `.cursor/skills/` (`build/`, `operate/`, `release/`). No plugin-marketplace install is required — they are referenced directly via `mdc:` links. The only external tool you need is the B2C CLI itself (next step).

Optionally, the `b2c-dx-mcp` MCP server can be added for richer tooling — see the [MCP installation docs](https://salesforcecommercecloud.github.io/b2c-developer-tooling/mcp/installation).

### Step 3 — Verify the B2C CLI is available

Run `b2c --version`. If the command is not found:

- For a one-off invocation: `npx @salesforce/b2c-cli <command>`.
- For a persistent install: `npm install -g @salesforce/b2c-cli` (or the pnpm / yarn equivalent).

Do not block on the global install — `npx` is sufficient for the rest of this flow.

### Step 4 — Account Manager access check

B2C Commerce has **no self-service signup**. The user must have Account Manager access provisioned by their organization's B2C Commerce admin before any of the following steps will work.

Ask (if not already clear from context): *"Do you have a Salesforce B2C Commerce Account Manager login and a target instance (sandbox or PIG)?"*

- If **no**: stop here. Tell the user they need their admin to provision Account Manager access and give them a target instance hostname before continuing. Do not proceed.
- If **yes**: continue.

### Step 5 — Check for existing configuration

Run `b2c setup inspect` to see whether a `dw.json` or credentials are already configured.

- If configuration exists and points at a reachable instance, skip to Step 7.
- If no configuration is found, proceed to Step 6.

For deep troubleshooting (wrong instance, profile switching, token inspection), delegate to the `b2c-config` skill.

### Step 6 — Initialize configuration

Guide the user to create a `dw.json` in the project root:

```bash
b2c setup
```

This prompts for hostname, client ID/secret (or username/password), and code version. For deeper configuration topics (multiple profiles, env vars, cert-based auth), delegate to the `b2c-config` skill.

### Step 7 — Sandbox

If the user wants to work against an existing sandbox, confirm it is reachable:

```bash
b2c setup inspect
b2c sandbox list   # requires API access
```

If the user needs a fresh sandbox, delegate to the `b2c-sandbox` skill for the full create flow. **Only create a sandbox when explicitly asked.**

### Step 8 — First cartridge deploy (if applicable)

If the user has cartridges locally:

```bash
b2c code deploy
```

For selective deploys, watch mode, or reload, delegate to the `b2c-code` skill.

If the user does not yet have cartridges, point them at the canonical starting points:

- **Storefront Next** (composable React frontend on MRT) — delegate to the [sfnext-project-setup](mdc:.cursor/skills/build/sfnext-project-setup/SKILL.md) skill.
- **SFRA** (Storefront Reference Architecture, legacy/hybrid backend): https://github.com/SalesforceCommerceCloud/storefront-reference-architecture
- **MRT bundle deploys** (headless runtime) — delegate to the [b2c-mrt](mdc:.cursor/skills/release/b2c-mrt/SKILL.md) skill on request.

### Step 9 — Route to the user's goal

Once setup is working, ask a single directing question to hand off to the right skill:

> "What do you want to work on first?
>
> 1. **Build a Storefront Next frontend** (React Router 7 + RSC on MRT)
> 2. **Build a Custom API** (SCAPI)
> 3. **Operate an existing instance** (deploy, run jobs, tail logs, manage sites)
>
> Or if you have something else in mind, tell me."

Route by the answer:

- **Storefront Next** → [sfnext-project-setup](mdc:.cursor/skills/build/sfnext-project-setup/SKILL.md), [sfnext-routing](mdc:.cursor/skills/build/sfnext-routing/SKILL.md), [sfnext-data-fetching](mdc:.cursor/skills/build/sfnext-data-fetching/SKILL.md), [sfnext-components](mdc:.cursor/skills/build/sfnext-components/SKILL.md)
- **Custom API** → [b2c-custom-api-development](mdc:.cursor/skills/build/b2c-custom-api-development/SKILL.md), [b2c-scapi-admin](mdc:.cursor/skills/build/b2c-scapi-admin/SKILL.md), [b2c-scapi-shopper](mdc:.cursor/skills/build/b2c-scapi-shopper/SKILL.md)
- **Operations** → [b2c-code](mdc:.cursor/skills/release/b2c-code/SKILL.md), [b2c-job](mdc:.cursor/skills/release/b2c-job/SKILL.md), [b2c-logs](mdc:.cursor/skills/operate/b2c-logs/SKILL.md), [b2c-sites](mdc:.cursor/skills/operate/b2c-sites/SKILL.md)

## Reference

- B2C CLI reference: https://salesforcecommercecloud.github.io/b2c-developer-tooling/cli/
- Install guide: https://salesforcecommercecloud.github.io/b2c-developer-tooling/guide/install
- Agent skills overview: https://salesforcecommercecloud.github.io/b2c-developer-tooling/guide/agent-skills
