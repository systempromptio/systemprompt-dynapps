---
title: "Create a Conversation and See It Land"
description: "Connect a client, invoke one skill on purpose, then find that conversation on the skill's Analysis page and open its transcript. Includes how long to wait and what to check when it does not appear."
author: "Astound Digital"
slug: "analysis-test-conversation"
keywords: "conversation, skill invocation, bridge, connect code, attribution, installation receipt, hooks, conversation rollup, session, transcript"
kind: "guide"
public: true
tags: ["enterprise", "admin", "analytics"]
published_at: "2026-09-16"
updated_at: "2026-09-29"
after_reading_this:
  - "Connect a client to this gateway and invoke a named skill deliberately"
  - "Find the resulting conversation on the skill's Analysis page"
  - "Open the transcript for that session"
  - "Diagnose a missing conversation: not routed, no hook report, rollup pending, or no install receipt"
related_docs:
  - title: "Analysis: Measure, Improve and Publish a Skill"
    url: "/documentation/analysis"
  - title: "Connect Claude Code"
    url: "/documentation/connect-claude-code"
  - title: "Develop Against a Local Gateway"
    url: "/documentation/develop-claude-code"
---

# Create a conversation and see it land

**TL;DR:** A conversation only exists because a signed-in person used a skill through a connected client. Connect Claude Code or OpenCode to this gateway, invoke one named skill, wait about a minute, then open `/admin/analysis/skills` filtered to that skill.

## Prerequisites

- A user account on this instance, and its email or user id.
- The gateway's HTTPS address, or `http://localhost:8080` for a local instance.
- At least one skill you can name, visible on `/admin/analysis/skills`. Pick one with a distinctive name.

## Step 1: Connect a client

Pick the route that matches where you are.

**Production or any remote gateway.** Follow [Connect Claude Code](/documentation/connect-claude-code) or [Connect OpenCode](/documentation/connect-opencode). Cowork is a separate client; connecting one does not configure the other.

**A local gateway you are developing against.** Issue a single-use code and run the containerised client:

```bash
systemprompt admin bridge issue-code --user-id you@example.com
just claude <code>
```

Codes are single-use with a ten-minute time to live, so issue one immediately before connecting. The full procedure, including the host-configuring variant, is in [Develop Against a Local Gateway](/documentation/develop-claude-code).

**Expected result:** the client starts without prompting for a code again.

## Step 2: Confirm the client is really routed through the gateway

In the connected client, ask any short question. Then open `/admin/history` in the console and confirm a request appears for your user.

A successful sign-in does not prove routing. This step does.

## Step 3: Invoke one skill on purpose

Model access and skill access are separate. Pick a skill by name from `/admin/analysis/skills` and invoke it explicitly rather than hoping the model reaches for it. In Claude Code, ask for the skill by name, or use the slash command your client exposes for it. Let the run finish.

The client's hooks report the invocation to this gateway as it happens: the plugin, the skill, the marketplace, and the services source that shipped it with that source's content hash.

**Expected result:** the skill's instructions are visible in the client's output, not a generic model answer.

## Step 4: Give the rollup a minute

Nothing on the Analysis pages is computed live from raw events. The
`conversation_rollup` job runs every minute and folds every conversation
touched since its last run into `conversation_facts` and the per-skill
`conversation_skill_facts`. Give it a minute before concluding anything is
wrong. `/admin/history` does not wait for the rollup, so your request is
visible there straight away.

## Step 5: Find the conversation

1. Open `/admin/analysis/skills` and pick a window that covers today.
2. Click the skill's name. Its page lists every conversation that invoked it.

Your session should be listed with the person, client and model, turns, tool
calls, artifacts, errors, cost and — once the judge has run — its AI score.
The same conversation is on `/admin/analysis/conversations`; the
**Conversations with skills** button on the Skills page opens that list
filtered to conversations that invoked any skill.

## Step 6: Open the transcript

Click the conversation. Its page shows the transcript turn by turn, with the
requests, tool calls, artifacts and governance decisions of each turn. The
same conversation is reachable from `/admin/conversations` and from your own
`/admin/history`.

## When it does not appear

Work down this list in order.

| Check | Symptom | What it means | Fix |
|---|---|---|---|
| 1. Reached the gateway? | Nothing in `/admin/history`. | The client's model traffic is not routed through this gateway. | Go back to Step 2. |
| 2. Hook reported? | The request is in History, but the skill's **Invoked** count did not move. | The plugin that carries the governance hooks is not installed on that client, so no invocation was reported. | Run `astound-bridge status`, sync, and restart the client. |
| 3. Rolled up? | Invoked moved but the conversation is missing from the skill's page. | The rollup has not passed yet. | Wait a minute and reload. After an upgrade an operator can rebuild everything with `systemprompt infra jobs run conversation_rollup -p all=true`. |
| 4. Installed? | The skill's **Installs** stays at zero. | Installs count verified installation receipts — a device that read back every file of the distributed bundle. Invocations still count without one. | Check **Installation receipts** on that marketplace's [Versions](/admin/analysis/versions) page. |

**Attributed is not the same as installed.** Attributed means the hook
reported the skill for that invocation — the percentage under the skill name
on the Skills table. A verified receipt is stricter: that device proved it
installed the published bundle.

## Troubleshooting

### The client asks for a code on every run

**Symptom:** A repeat run prompts for a connect code.
**Cause:** The stored credential did not validate against this gateway, usually because a different gateway issued it.
**Solution:** `just claude-reset`, then connect with a fresh code.

### The container cannot reach the gateway

**Symptom:** Connection refused from inside the client container.
**Cause:** Inside a container, `localhost` is the container.
**Solution:** `just claude` rewrites the address for you. By hand, use `http://host.docker.internal:8080`.

### The conversation is in History but not on the skill's page

**Symptom:** `/admin/history` shows the request; the skill's page does not list it.
**Cause:** No invocation of that skill was reported for the session — the model answered without it, or the hooks plugin is not installed on that client.
**Solution:** Re-run invoking the skill explicitly, and confirm the skill is served from a marketplace this user receives.

## Related pages

- [Analysis overview](/documentation/analysis)
- [Measure Which Skills Are Used](/documentation/analysis-measure-skills)
- [Connect Claude Code](/documentation/connect-claude-code)
- [Develop Against a Local Gateway](/documentation/develop-claude-code)
