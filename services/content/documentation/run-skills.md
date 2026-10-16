---
title: "Use Your Skills and Run Them on the Agent Farm"
description: "For business users: use the Dynapps skills in Cowork or Claude Code, then run your Odoo and DynBrain skills unattended on the Agent Farm under the identity you choose."
author: "Dynapps"
slug: "run-skills"
kind: "guide"
public: true
tags: ["documentation", "getting-started", "skills", "agents"]
published_at: "2026-10-16"
updated_at: "2026-10-16"
keywords: "skills, run skills, cowork, claude code, agent farm, odoo, dynbrain, run_as, launch, business users"
after_reading_this:
  - "Find and use a Dynapps skill in Cowork or Claude Code"
  - "Launch an Agent Farm run that uses your Odoo and DynBrain tools"
  - "Choose whose identity a run uses and follow its cost"
related_playbooks:
  - title: "Set Up Dynapps on Your Laptop"
    url: "/documentation/setup"
  - title: "Odoo and DynBrain in the Agent Farm (full reference)"
    url: "/documentation/agent-farm-odoo-dynbrain"
  - title: "Agent Farm from Claude Code"
    url: "/documentation/agent-farm"
---

# Use Your Skills and Run Them on the Agent Farm

Finish [Set Up Dynapps on Your Laptop](/documentation/setup) first. You then
have the Dynapps skills in Claude Cowork, and Dynapps Odoo and DynBrain
connected under your own portal.dynapps.es account.

## Use a skill

A skill is a packaged set of instructions for one kind of job: a CRM report,
a call summary, an accounting audit. You can use one in two ways:

- **Just ask.** Describe the job in plain language ("prepare my call with
  ACME tomorrow"). Claude picks the matching skill on its own.
- **Name it.** Type `/` in Cowork or Claude Code and pick from the list. Skills
  appear as `/<plugin>:<skill>`, for example
  `/dynapps-portal:dynapps-portal-guide`, which routes how-to questions to
  DynBrain and live-record work to Dynapps Odoo.

Browse everything you can use in the [skills catalogue](/skills/). If a skill
you expect is missing, sync again from the **Marketplace** tab in Dynapps
Bridge; if it is still missing, ask your administrator for access.

Things to know:

- **Dynapps Odoo asks before every call.** It works on the live ERP, so each
  tool call needs your confirmation. DynBrain does not ask.
- **Reading Odoo is open; changing it is not.** Create, write, delete and
  execute tools need the `admin` or `odoo_writer` role.
- **Everything is recorded.** Every model and tool call is checked by
  governance and logged with your identity and its cost.

## Run a skill unattended on the Agent Farm

The Agent Farm runs agents in the background, so a job keeps going without your
laptop. A run reaches your data like this:

**You** (sign in at `/admin`) → **connector grant** (OAuth to portal.dynapps.es)
→ **farm run** (identity from `run_as`) → **gateway**
(`/api/v1/mcp/<server>/mcp`) → **portal MCP** (`dynapps-odoo`, `dynbrain`).

### 1. Check you have access

You need the `staff` group or an admin role. That opens the `dynapps-portal`
marketplace, its `dynapps-showcase` and `dynapps-farm` plugins and both
portal servers. To see a farm's agents you also need the marketplace that farm
names. If anything is missing, ask an admin.

### 2. Connect Odoo and DynBrain

Done in setup. If a tool call later returns 401, reconnect at
[/admin/connectors](/admin/connectors).

### 3. Install the farm plugin

In Claude Code, install `dynapps-farm` from `dynapps-portal`. It brings
the `dynapps-agent-farm` skill (launch, follow, cancel and schedule farm tasks
in plain language) plus both portal servers.

### 4. Launch

Pick whichever is easiest:

- **Ask Claude**, naming the farm: "run the CRM report on the agent farm".
  The `dynapps-agent-farm` skill only acts when you mention the farm.
- **From the console**: [/admin/farm](/admin/farm) → farm → agent → launch.
- **From the API**: `POST /api/public/agent-farm/runs`.

Defaults: 15 turns, 20 minutes, $2 budget.

### 5. Follow the run and its cost

| Where | What it shows |
|---|---|
| [/admin/farm/runs](/admin/farm/runs) | Every run, its status and output |
| [/admin/farm/clients](/admin/farm/clients) | Spend per client |
| [/admin/farm/people](/admin/farm/people) | Spend per person |

## Whose identity a run uses

Each farm (or a single agent) sets `run_as`. The agent's setting wins over the
farm's.

| Mode | Acts as | Refused when |
|---|---|---|
| `user` | The person who launched the run, with your own Odoo and DynBrain grants | Never at launch; tool calls return 401 if you never connected |
| `client` | The service account of the run's client | The run has no client, or the client has no service account (422) |
| `service_account` | The named service account | The account is missing or inactive (422) |

Client and service-account modes need an admin to connect Odoo and DynBrain
once for that account, under **Portal grants** on its page at
[/admin/identity/service-accounts](/admin/identity/service-accounts). Only farm
runs can spend those grants.

**Unattended farm runs can read Odoo but never write to it.**

## When something goes wrong

| What you see | What to do |
|---|---|
| The plugins or servers aren't offered | Ask an admin to add you to `staff` or an admin role |
| A tool call returns 401 from the portal | Reconnect at `/admin/connectors`, or the service account's Portal grants |
| Refused by `farm_tool_allowlist` | The agent doesn't list that tool; ask the farm's owner to add its full `mcp__server__tool` name |
| 422 "the run names no client" | Launch for a client, or ask for `client:` on the farm |
| 422 "client … names no service_account" | Ask an admin to give the client a service account, or use `user` mode |
| 422 "no active service account is resolved" | Ask an admin to fix or reactivate the account |
| The run touched the wrong person's records | Check the agent's `run_as` first, then the farm's |
| An Odoo write is refused by governance | Ask an admin for the `odoo_writer` role (interactive sessions only) |

Building or configuring farm agents yourself — declaring tools, setting
`run_as` in `agent-farms.yaml`, checking the gateway with a PAT — is in the
full reference: [Odoo and DynBrain in the Agent Farm](/documentation/agent-farm-odoo-dynbrain).
Developers following runs, events and evals from Claude Code: see
[Agent Farm from Claude Code](/documentation/agent-farm).
