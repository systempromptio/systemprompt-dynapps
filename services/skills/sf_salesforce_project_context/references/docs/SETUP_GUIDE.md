# Setup Guide

A step-by-step guide for Salesforce developers who are new to **Cursor IDE** and want to leverage this Salesforce Core agent kit on a project.

---

## Table of contents

1. [Prerequisites](#1-prerequisites)
2. [Install Cursor IDE](#2-install-cursor-ide)
3. [Install Salesforce CLI](#3-install-salesforce-cli)
4. [Install Node.js](#4-install-nodejs)
5. [Install Git](#5-install-git)
6. [Apply the agent kit to your project](#6-apply-the-agent-kit-to-your-project)
7. [Open the project in Cursor](#7-open-the-project-in-cursor)
8. [Authenticate your Salesforce org](#8-authenticate-your-salesforce-org)
9. [Verify the kit is loaded](#9-verify-the-kit-is-loaded)
10. [Fill in project-specific files](#10-fill-in-project-specific-files)
11. [Your first story](#11-your-first-story)
12. [Branching and deployment practice](#12-branching-and-deployment-practice)
13. [Troubleshooting](#13-troubleshooting)

---

## 1. Prerequisites

- A Salesforce org (Developer Edition, Sandbox, or Scratch Org).
- A Git host account (GitHub / GitLab / Bitbucket) with access to the project repo.
- A computer running Windows 10 / 11, macOS, or Linux.

You will install the following tools in the steps below:

| Tool | Purpose |
|---|---|
| Cursor IDE | AI-powered code editor (built on VS Code) |
| Salesforce CLI (`sf`) | Deploy, retrieve, and manage Salesforce metadata |
| Node.js (v18+) | Required by Salesforce CLI, ESLint, Prettier, and Jest |
| Git | Version control |

## 2. Install Cursor IDE

Cursor is a VS Code-based editor with built-in AI capabilities. It loads `.cursor/rules/`, `.cursor/agents/`, and `.cursor/skills/` from the project root automatically.

1. Go to the Cursor download page and install for your OS.
2. On first launch, sign in (or skip) and accept the defaults.

Recommended extensions (Cursor inherits VS Code marketplace):

- Salesforce Extension Pack
- Salesforce Apex
- Salesforce Lightning Web Components

## 3. Install Salesforce CLI

Install the modern `sf` CLI from the Salesforce CLI installer for your OS. Verify:

```bash
sf --version
```

You should see a recent version of `@salesforce/cli`.

## 4. Install Node.js

Install Node.js 18 or later. Verify:

```bash
node --version
npm --version
```

## 5. Install Git

Install Git for your OS. Configure your name / email once:

```bash
git config --global user.name "Your Name"
git config --global user.email "you@example.com"
```

## 6. Apply the agent kit to your project

From this kit's root, copy the kit folders into your Salesforce project:

```bash
# Inside your Salesforce project root
cp -r <path-to-kit>/.cursor .cursor
cp -r <path-to-kit>/docs/PROJECT_SCHEMA.md docs/PROJECT_SCHEMA.md
cp -r <path-to-kit>/docs/IMPLEMENTATION_BRIEF_TEMPLATE.md docs/IMPLEMENTATION_BRIEF_TEMPLATE.md
cp -r <path-to-kit>/docs/BRANCH_AND_DEPLOY_PRACTICE.md docs/BRANCH_AND_DEPLOY_PRACTICE.md
cp -r <path-to-kit>/docs/AGENT_GUIDE.md docs/AGENT_GUIDE.md
cp -r <path-to-kit>/stories/README.md stories/README.md
```

(On Windows / PowerShell use `Copy-Item -Recurse`.)

## 7. Open the project in Cursor

```bash
cursor .
```

Cursor will detect `.cursor/rules/always-active.md` and apply it as standing instructions for every chat / agent session.

Install npm dependencies if `package.json` is present:

```bash
npm install
```

## 8. Authenticate your Salesforce org

Use a memorable alias (referenced throughout this kit and stories):

```bash
sf org login web --alias <sandbox-alias> --instance-url https://test.salesforce.com
```

For a production-style org, use `https://login.salesforce.com`.

Verify:

```bash
sf org list
sf org display --target-org <sandbox-alias>
```

## 9. Verify the kit is loaded

Open the chat in Cursor and ask: "What rules and agents are available in this project?"

You should see the agent reference `.cursor/INDEX.md`, mention `always-active.md`, and list the worker / lead / lane agents.

## 10. Fill in project-specific files

Two files **must** be filled in before the agent can produce accurate work:

1. `.cursor/skills/Salesforce_Project_Context.md` — project identity, custom objects, existing automations, naming conventions.
2. `docs/PROJECT_SCHEMA.md` — every custom object and field the project owns.

Without these, the agent will fall back to generic patterns and may invent API names.

## 11. Your first story

1. Paste the story / ticket text into the chat.
2. The agent applies `story-intake.md`: parses the story, picks an agent, lists assumptions, and creates `stories/<STORY-ID>/` with the four files (`requirement.md`, `design-note.md`, `deployment.md`, `qa-test-request.md`).
3. Review the brief; clarify any open questions.
4. Create the feature branch:

   ```bash
   git checkout -b feature/<STORY-ID>
   ```

5. Let the agent implement; it will follow the rules in `always-active.md` (one trigger per object, 90% coverage, etc.) and update the story folder as it goes.

## 12. Branching and deployment practice

Read `docs/BRANCH_AND_DEPLOY_PRACTICE.md` and `.cursor/skills/DevOps.md`. The short version:

- One branch per feature / bug.
- Deploy from the branch to a sandbox.
- PR into `develop`; promote to `main` for production with approvals.

## 13. Troubleshooting

- **Agent does not seem to know the project.** Confirm `.cursor/skills/Salesforce_Project_Context.md` and `docs/PROJECT_SCHEMA.md` are filled in.
- **Deploy hangs / rolls back.** See `.cursor/skills/Deployment_Troubleshooting.md`.
- **App not in App Launcher.** See `.cursor/skills/Profile_PermissionSet_Patterns.md` ("App Launcher visibility").
- **Test class fails after a new validation rule.** Update the `TestDataFactory` to satisfy the new rule (see `.cursor/skills/Apex_Test_Strategy.md`).
- **Cursor does not pick up changes to `.cursor/`.** Restart Cursor or open a new chat.
