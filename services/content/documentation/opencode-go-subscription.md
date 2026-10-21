---
title: "Bring Your Own OpenCode Go Subscription"
description: "Use your personal OpenCode Go subscription through the Dynapps gateway: add your Go key once, pick a go-* model in OpenCode Desktop, and keep governance and audit while usage bills to your own subscription."
author: "Dynapps"
slug: "opencode-go-subscription"
keywords: "opencode go, byok, bring your own key, subscription, go-*, glm, kimi, deepseek, minimax, qwen, mimo, personal key, opencode desktop, bridge"
kind: "guide"
public: true
tags: ["documentation", "getting-started", "opencode"]
published_at: "2026-10-21"
updated_at: "2026-10-21"
after_reading_this:
  - "Add your own OpenCode Go key to Dynapps from the bridge or your profile"
  - "Pick a go-* model in OpenCode Desktop and know which upstream serves it"
  - "Know what the gateway still governs and records, and who pays"
  - "Know the single-member terms: one key per person, never shared"
related_playbooks:
  - title: "Connect OpenCode"
    url: "/documentation/connect-opencode"
  - title: "Install the Desktop Bridge"
    url: "/documentation/bridge-install"
  - title: "Gateway API"
    url: "/documentation/gateway-api"
---

# Bring Your Own OpenCode Go Subscription

[OpenCode Go](https://opencode.ai/docs/go/) is a personal subscription to a set of open models: GLM, Kimi, DeepSeek, MiMo, MiniMax and Qwen. If you hold one, Dynapps can send your requests for those models to OpenCode Go using **your** key. The gateway keeps governing and recording each request, and the usage counts against your Go subscription, not the organization's account.

## What you need

- An OpenCode Go subscription and its API key, from the [OpenCode console](https://opencode.ai/).
- [OpenCode Desktop](https://opencode.ai/download) (the CLI works too).
- The Dynapps Bridge, installed and signed in. See [Install the Desktop Bridge](/documentation/bridge-install).

## 1. Install OpenCode Desktop and connect the bridge

Install OpenCode Desktop, then follow [Connect OpenCode](/documentation/connect-opencode). The bridge registers OpenCode with the gateway, so every model you are allowed to use shows up under the Dynapps provider.

## 2. Add your Go key

You can add the key in either of two places. Both save it to the same per-person store on the gateway.

- **Bridge:** open **Settings → Personal subscriptions**, choose **OpenCode Go**, paste the key and select **Save**. The bridge tests the key, then refreshes your profile, which rewrites OpenCode's configuration.
- **Web console:** open [your profile](/admin/profile) and find the **Personal subscriptions** card. Paste the key, then select **Save**. **Test** checks the key against OpenCode Go. **Remove** deletes it.

The gateway tests the key before saving it and rejects one that OpenCode Go refuses. The key is stored encrypted. Neither the console nor the bridge shows it again, only a label and when it was last used.

## 3. Pick a go-* model

After the profile refresh, OpenCode lists the Go models with a `go-` prefix. The prefix keeps them separate from the organization's own `deepseek-*` route, which uses the platform key.

| Model id | Upstream model | Endpoint |
|----------|----------------|----------|
| `go-glm-5.3` | `glm-5.3` | Chat Completions |
| `go-kimi-k3` | `kimi-k3` | Chat Completions |
| `go-deepseek-v4-pro` | `deepseek-v4-pro` | Chat Completions |
| `go-deepseek-v4.1-flash` | `deepseek-v4.1-flash` | Chat Completions |
| `go-mimo-v2.6-pro`, `go-mimo-v2.6-flash` | `mimo-v2.6-*` | Chat Completions |
| `go-minimax-m3` | `minimax-m3` | Anthropic Messages |
| `go-qwen3.8-max`, `go-qwen3.8-flash` | `qwen3.8-*` | Anthropic Messages |

Pick one in OpenCode and send a prompt. If you have not added a key, the gateway refuses `go-*` requests with a 403 telling you to add one.

## Governance and audit still apply

A request on your own key goes through the same gateway pipeline as any other request:

- the scope check, secret scan, blocklist and rate limit;
- the safety scanners, in both directions;
- the audit record, with your identity, the model, token counts and the trace id.

The only differences are the credential and who pays. The audit row records `billing_source=user_key`, and administrators see your usage in `infra logs request list` and the analytics dashboards like any other request. The cost shown there is an estimate from OpenCode's list prices, for comparison only.

## Billing

OpenCode Go bills usage to **your** subscription, within its own 5-hour, weekly and monthly limits. Dynapps does not pay for these requests and does not raise those limits. Check your usage in the OpenCode console.

## Single-member terms

An OpenCode Go subscription belongs to one person, and Dynapps enforces that:

- **One key per person.** The key is stored against your account and only your requests use it. It is never used for another member's request.
- **Never shared.** Do not add someone else's key, and do not give yours to a colleague or put it in a shared configuration. Each member who wants Go models adds their own key.
- **Remove it when you leave.** Removing the key, or closing your account, deletes it from the gateway.
