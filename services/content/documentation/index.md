---
title: "Get Started with the Astound AI Platform"
description: "Choose your Astound client and follow the platform setup, sign-in, and verification guides."
author: "Astound Digital"
slug: ""
kind: "guide"
public: true
tags: ["documentation", "getting-started", "bridge"]
published_at: "2026-02-18"
updated_at: "2026-09-29"
keywords: "get started, astound ai platform, claude code, cowork, opencode, bridge, gateway, documentation"
after_reading_this:
  - "Pick the setup guide for your client and platform"
  - "Verify your first session reached the gateway"
  - "Find the page for any part of the platform"
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

Changing a skill and getting it in front of agents is one walkthrough:
[Update a Skill and Ship It](/documentation/update-a-skill) — edit, gate, publish,
import, verify. Maintainers can use the
[skill lifecycle guide](/documentation/skills-lifecycle) to trace a skill from
its source through distribution, loading and measurement.

## Sync: code and instance

- [Code ↔ Instance: sources, planes and sync](/documentation/services-sync) — the model behind it: where declarations come from, what they project into, which hash proves what
- [Access control: who reaches what, and why](/documentation/access-control) — entitlement, declared here and enforced from the database

## Analysis: measuring and comparing skills

- [Analysis overview: the tabs and the end-to-end path](/documentation/analysis)
- [Measure which skills are used](/documentation/analysis-measure-skills)
- [Evaluate a plugin before you ship it](/documentation/analysis-evaluate-plugins)
- [Create a conversation and see it land](/documentation/analysis-test-conversation)
- [Versions: hashes, history and compare](/documentation/analysis-versions)
- [Artifacts and the tool-call ledger](/documentation/artifacts-and-tool-calls)

## Connections

- [Salesforce CRM and development connections](/documentation/salesforce)
- [Downloads and checksum verification](/documentation/downloads)
- [Bridge installation, sign-in, and troubleshooting](/documentation/bridge-install)

## Administration

- [Dashboard](/documentation/dashboard) — the admin overview
- [Create and manage users](/documentation/admin-user-management) — including the one role table
- [Authentication](/documentation/authentication)
- [Set up SSO (AD FS)](/documentation/adfs-sso-setup)
- [User and access management](/documentation/enterprise-user-access)
- [Usage and analytics](/documentation/enterprise-analytics)
- [Cost management](/documentation/enterprise-cost-management)
- [Model routing and data residency](/documentation/enterprise-model-routing)
- [Gateway API](/documentation/gateway-api)
- [Conversation history and search](/documentation/enterprise-conversation-history)

## Governance and compliance

- [Governance modes: warn and enforce](/documentation/governance-modes)
- [Safety and guardrails](/documentation/enterprise-safety-guardrails)
- [Tool governance](/documentation/enterprise-tool-governance)
- [Audit and observability](/documentation/enterprise-audit-observability)
- [Export telemetry (OTLP)](/documentation/observability-export)
- [Data retention](/documentation/data-retention)
- [Roadmap and known limitations](/documentation/enterprise-roadmap)

## Deploy and operate

- [Stand up one node and PostgreSQL](/documentation/deploy)
- [Deploy on Oracle Cloud](/documentation/deploy-oracle-cloud)
- [Validate every stage](/documentation/deploy-troubleshooting)
- [Expose your instance remotely](/documentation/remote-access)
- [Target deployment architecture](/documentation/deploy-target-architecture)

For platform maintainers: [Develop Against a Local Gateway](/documentation/develop-claude-code).
