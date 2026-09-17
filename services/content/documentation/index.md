---
title: "Get Started with the Astound AI Platform"
description: "Choose your Astound client and follow the platform setup, sign-in, and verification guides."
author: "Astound Digital"
slug: ""
kind: "guide"
public: true
tags: ["documentation", "getting-started", "bridge"]
published_at: "2026-02-18"
updated_at: "2026-09-13"
related_playbooks:
  - title: "Connect Claude Code"
    url: "/documentation/connect-claude-code"
  - title: "Connect Cowork"
    url: "/documentation/connect-cowork"
  - title: "Downloads"
    url: "/documentation/downloads"
---

# Get Started

Connect your AI client to your organization's gateway to use the models, skills, and tools your account can access. You need an account and Astound Bridge on your computer; you do not need to build or run a server.

## Choose your client

| What you want to use | Setup guide | Platforms |
|---|---|---|
| Claude Code in a terminal, including gateway Gemini models | [Connect Claude Code](/documentation/connect-claude-code) | macOS, Linux, native Windows; optional WSL |
| Claude Cowork in Claude Desktop | [Connect Cowork](/documentation/connect-cowork) | macOS and Windows |
| OpenCode | [Connect OpenCode](/documentation/connect-opencode) | See the client guide |

Claude Code and Cowork are different clients. Connecting one does not configure the other. On Windows, follow the native PowerShell setup and enrol each client separately. The Windows guide identifies the bridge build needed for Claude Code automation; WSL uses a separate Linux bridge.

## Before you start

Ask your administrator for your gateway's HTTPS address, access to an account, and the trusted bridge manifest public key if your device has not already been configured. Use the same gateway for downloading, signing in, and connecting. Do not rely on the app's default address.

Start with [Install on macOS](/documentation/install-macos), [Install on Windows](/documentation/install-windows), or the [Linux and WSL instructions](/documentation/bridge-install#linux-and-wsl). Then follow your client guide.

## Verify your first session

Ask a short question in the connected client. Check that the request appears in your gateway's [History](/admin/history), or ask your administrator to confirm it. A successful sign-in alone does not prove that model requests are routed through the gateway.

Next, try a read-only skill or tool that your administrator has enabled. Model access and tool access are separate: a working model response does not prove that plugins are installed or authorized. Browse the [skills catalogue](/skills/) for examples.

## Skills

Maintainers can use the [skill lifecycle guide](/documentation/skills-and-evaluations)
to trace a skill from its source through distribution, loading and measurement.

## Analysis: measuring and comparing skills

- [Analysis overview: the five tabs and the end-to-end path](/documentation/analysis)
- [Measure which skills are used](/documentation/analysis-measure-skills)
- [Create a conversation and see it land](/documentation/analysis-test-conversation)
- [Versions: sources, generations and compare](/documentation/analysis-versions)

## Reference and administration

- [Salesforce CRM and development connections](/documentation/salesforce)
- [Downloads and checksum verification](/documentation/downloads)
- [Bridge installation, sign-in, and troubleshooting](/documentation/bridge-install)
- [Authentication](/documentation/authentication)
- [User and access management](/documentation/enterprise-user-access)
- [Usage and analytics](/documentation/enterprise-analytics)
- [Model routing and data residency](/documentation/enterprise-model-routing)
- [Cost management and budgets](/documentation/enterprise-cost-management)
- [Audit and observability](/documentation/enterprise-audit-observability)
- [Artifacts and the tool-call ledger](/documentation/artifacts-and-tool-calls)
- [Safety and guardrails](/documentation/enterprise-safety-guardrails)
- [Tool governance](/documentation/enterprise-tool-governance)

For platform maintainers: [Develop Against a Local Gateway](/documentation/develop-claude-code).
