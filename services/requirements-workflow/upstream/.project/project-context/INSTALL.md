# Project Context — Installation Guide

This file is machine-readable. If you received an error pointing here, follow the
steps below in order. Each step includes the exact command to run.

---

## Step 1 — Install dependencies

Run this once after copying the build into the project:

```bash
cd .cursor/.project/project-context
npm install
```

**Verify:** `node_modules/` directory exists inside `.cursor/.project/project-context/`.

---

## Step 2 — Create `.env`

Copy the example and fill in your credentials:

```bash
cp .cursor/.project/project-context/.env.example .cursor/.project/.env
```

> The `.env` file lives one level up from the engine, in `.cursor/.project/`.
> It is gitignored — never commit it.

Open `.cursor/.project/.env` and fill in all blank values (see Step 3 and 4).

---

## Step 3 — AWS Bedrock credentials

Get your Bedrock API key from the AWS console (it starts with `ABSK`):
1. Open [AWS Console → Amazon Bedrock → API keys](https://eu-central-1.console.aws.amazon.com/bedrock/home?region=eu-central-1#/api-keys)
2. Create or copy an existing key.
3. Set in `.cursor/.project/.env`:

```
PROJECT_CONTEXT_BEDROCK_REGION=eu-central-1
AWS_BEARER_TOKEN_BEDROCK=ABSK...
```

---

## Step 4 — Atlassian credentials

You need an Atlassian API token (not your password):
1. Go to https://id.atlassian.com/manage-profile/security/api-tokens
2. Click **Create API token**, give it a name, copy the value.

Set in `.cursor/.project/.env`:

```
ATLASSIAN_BASE_URL=https://your-company.atlassian.net
CONFLUENCE_SPACES=ENG,DOC          # comma-separated space keys
JIRA_PROJECTS=PROJ,PLATFORM        # comma-separated project keys
ATLASSIAN_EMAIL=you@yourcompany.com
ATLASSIAN_API_TOKEN=<paste token here>
```

---

## Step 5 — Review `config.yaml` (optional)

The config lives at `.cursor/.project/project-context/config.yaml`.
Defaults work out of the box, but you can adjust:
- `sources.confluence.spaces` / `sources.jira.projects` (overrides env vars)
- `bedrock.region` (default: `eu-central-1`)
- `openspec_archive.path` if your OpenSpec repo is in a non-standard location

---

## Step 6 — First sync

Run the sync command in Cursor chat:

```
/project-context:sync
```

This pulls the latest committed docs, syncs Confluence + Jira into `inbox/`, then indexes everything.

To force a full reindex from scratch, run the indexer directly from
`.cursor/.project/project-context`:

```bash
npm run index -- --rebuild
```

---

## Troubleshooting

| Error | Fix |
|-------|-----|
| `Cannot find package '@lancedb/lancedb'` | Run `npm install` in `.cursor/.project/project-context/` (Step 1) |
| `Missing .env` | Create `.cursor/.project/.env` from the example (Step 2) |
| `Missing AWS_BEARER_TOKEN_BEDROCK` | Fill in your Bedrock API key (Step 3) |
| `Missing ATLASSIAN_BASE_URL` or `ATLASSIAN_EMAIL` | Fill in Atlassian credentials (Step 4) |
| `UnrecognizedClientException` from Bedrock | Token is invalid or expired — get a new one (Step 3) |
| `401 Unauthorized` from Atlassian | API token is wrong or revoked — regenerate (Step 4) |
| Index is empty after sync | Check `CONFLUENCE_SPACES` / `JIRA_PROJECTS` match actual keys |
