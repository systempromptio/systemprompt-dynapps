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
keywords: "skills, run skills, cowork, claude code, agent farm, odoo, dynbrain, launch, business users"
after_reading_this:
  - "Find and use a Dynapps skill in Cowork or Claude Code"
  - "Launch an Agent Farm task from Claude Code"
related_playbooks:
  - title: "Set Up Dynapps on Your Laptop"
    url: "/documentation/setup"
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
laptop.

1. **Check you have access.** You need the `staff` group or an admin role.
   That opens the `dynapps-portal` marketplace and its `dynapps-farm` plugin.
   If anything is missing, ask an admin.
2. **Install the farm plugin.** In Claude Code, install `dynapps-farm` from
   `dynapps-portal`. It brings the `dynapps-agent-farm` skill: launch, follow,
   cancel and schedule farm tasks in plain language.
3. **Ask Claude, naming the farm**: "run the CRM report on the agent farm".
   The `dynapps-agent-farm` skill only acts when you mention the farm.
