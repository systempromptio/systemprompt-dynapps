---
title: "Secrets Management"
description: "Where the platform's secrets come from (file, environment, Vault or an external source), how the 1Password source reads them from a vault and reloads them without a restart, and the rotation runbook."
author: "Dynapps"
slug: "secrets-management"
keywords: "secrets, 1password, onepassword, op cli, service account, connect, vault, rotation, reload, secrets.json, external secrets"
kind: "guide"
public: true
tags: ["security", "secrets", "administration", "deployment"]
published_at: "2026-10-13"
updated_at: "2026-10-13"
after_reading_this:
  - "Choose a secrets source for a profile and know which one wins"
  - "Load every secret from a 1Password vault with a service account or a Connect server"
  - "Rotate a secret in 1Password and confirm the platform picked it up"
  - "Know which secrets reload live and which still need a restart"
related_playbooks:
  - title: "Authentication"
    url: "/documentation/authentication"
  - title: "Set Up Dynapps SSO (Microsoft Entra ID)"
    url: "/documentation/entra-id-sso-setup"
  - title: "Deploy"
    url: "/documentation/deploy"
---

# Secrets Management

**TL;DR:** The platform's secrets are one flat document — the shape of
`secrets.json` — loaded at boot from the source the profile names. With
`secrets.source: external` and `kind: onepassword` the document is read from
a 1Password vault, re-read on a schedule, and swapped in place when a value
changes, so rotating a provider key is a change in 1Password and nothing else.
Four infrastructure keys are pinned for the life of the process and still need
a restart. Every source is fail-closed, and no command or page ever shows a
value.

What each key must hold, who owns it and how it rotates by hand is in the
self-host reference, `deploy/self-host/SECRETS.md`. This page covers where the
document comes from and how it changes at run time.

## The secrets source

The profile's `secrets:` block names the source:

| `secrets.source` | Reads from |
|---|---|
| `file` | `secrets_path`, relative to the profile directory (`secrets.json`) |
| `env` | The process environment; run locally, the file first and the environment as fallback |
| `vault` | HashiCorp Vault KV v2. The token never sits in profile YAML |
| `external` | A provider an extension registered for `secrets.external.kind` — in this installation, `onepassword` |

Precedence, highest first: a subprocess handed its secrets by its parent (MCP
servers); `vault` or `external`, which win **even on a deployment host**, so a
container is never silently downgraded to whatever happens to be in its
environment; the deployment host's environment; `env` locally; `file`.

**Fail-closed.** A file, Vault fetch or external fetch that fails aborts the
boot. There is no fallback to the environment, because that would start the
process on whatever stale credentials the host carries.

## The external source contract

Core ships no external kind of its own. An extension contributes one at
compile time with `register_secrets_source!`, keyed by `kind`, and a binary
picks it up by linking the crate. Core hands the provider the rest of the
`secrets.external` block untouched and expects back a `secrets.json`-shaped
document, which it validates in one pass.

| Behaviour | Rule |
|---|---|
| Boot | Fetched once; any failure aborts the boot |
| Reload | `SecretsBootstrap::reload` re-runs the loader. If no value changed, nothing is swapped. If one did, the new set becomes the next **generation** |
| Failed reload | The current generation stays live and the error is recorded as `last_error` |
| Pinned keys | `encryption_master_key`, `oauth_at_rest_pepper`, `manifest_signing_secret_seed`, `database_url`. A reload that changes one is **refused** — data sealed, signed or connected with the old value would stop verifying |
| Schedule | `secrets.external.refresh_secs`; omitted means boot-only. Never more often than every **60 s** (a lower value is raised to 60) |

## The 1Password source

The `onepassword` kind (`extensions/secrets/onepassword`) reads the document
from a 1Password vault through one of two backends.

### Service account (`backend: cli`, the default)

Create a 1Password **service account** with read access to the vault (only
that vault), and give the process its token in `OP_SERVICE_ACCOUNT_TOKEN` (or
name another variable with `token_env`). The image ships a pinned `op` CLI at
`/usr/local/bin/op` (root `Dockerfile`, stage `opcli`); `op_bin` overrides the
path.

The platform runs `op` with a **cleared environment**: the token goes in as
`OP_SERVICE_ACCOUNT_TOKEN`, plus only `PATH`, `HOME`, `XDG_CONFIG_HOME` and
`TMPDIR`. The token is never on the command line, `op`'s output is parsed and
never logged, and errors name the vault, item, field or status — never a value.
Each distinct item is read once with `op item get --format json`; if an item
cannot be read whole, its references are read one by one with `op read`.

### Connect server (`backend: connect`)

For a self-hosted 1Password Connect server, set `backend: connect` and
`connect_url`, and provide the Connect token in `OP_CONNECT_TOKEN` (or the
variable named by `token_env`). Requests time out after `timeout_secs`
(default 10, max 120) and retry up to `retries` times (default 3, max 10) on
429, 5xx, timeouts and refused connections — never on 401, 403 or 404.
Redirects are never followed, so the token cannot leave the configured host.

### Mapping keys to 1Password

Two mutually exclusive forms; exactly one is required.

**`items:`** — one line per key, `key: op://<vault>/<item>/<field>`. Each
reference names its own vault. Items resolve by title, then id; fields by
label, then id.

**`item:`** — every CONCEALED or STRING field of that one item (in `vault`)
becomes the key named by its label.

Two matches at the same step — two items with one title, two fields with one
label — are an error, never a guess; reference by id instead.

```yaml
# profile.yaml
secrets:
  source: external
  external:
    kind: onepassword
    refresh_secs: 300            # omit for boot-only; floor 60
    backend: cli                 # or connect, with connect_url
    vault: dynapps-prod
    items:
      anthropic: op://dynapps-prod/platform/anthropic
      entra_oidc_client_secret: op://dynapps-prod/identity/entra_oidc_client_secret
      encryption_master_key: op://dynapps-prod/identity/encryption_master_key
```

A literal `token:` or `connect_token:` key in the block is refused at boot:
the profile names the variable, the environment holds the value.

On Kubernetes, the Helm chart's `secrets.mode: onepassword` renders this block
from `secrets.onepassword.*` (`backend`, `connectUrl`, `vault`,
`refreshSecs`, `items` or `item`, `tokenEnv`) and injects the token from the
Secret named in `secrets.onepassword.tokenSecret`; no `secrets.json` is
mounted.

## Reloads: the job, the console and the CLI

**The `secrets_refresh` job** wakes every minute and reloads once
`refresh_secs` (at least 60) has passed since the last load or reload; for any
other source it is a no-op. A reload that changed keys logs their names and
leaves a `user_activity` row under the job's owner; one refused for a pinned
key is reported as a failed run.

**The console.** `/admin/configuration` has a secrets card: source,
generation, load and last reload times, the last reload error, and the key
names — never a value. An admin can press **Reload now**, which runs the same
reload and reports `generation N, changed: <keys>` or the error.

**The CLI.**

```bash
systemprompt admin config secret check
```

builds the configured provider and reads the source itself, so it proves the
token and mappings work from where you run it. It prints the source, a detail
line (for 1Password, `onepassword <backend> vault <vault>`), the key names the
source returned, any missing required key (`oauth_at_rest_pepper`,
`database_url`), and — when a store is loaded in that process — its
generation, last reload time and last error. It never prints a value. For the
running server's generation, read the console card.

`systemprompt admin config secret set` refuses to run on a `vault` or
`external` profile — the file it writes would be ignored at boot — and says to
rotate in the source and run `secret check`.

## Rotation runbook

1. **Rotate in 1Password.** Update the field the key's `op://` reference
   points at. Do not rename the item or field, or the reference stops
   resolving and the next reload fails (the old generation stays live).
2. **Wait or reload.** Wait up to `refresh_secs` (plus the job's one-minute
   tick), or press **Reload now** on `/admin/configuration`.
3. **Confirm the generation moved.** The `/admin/configuration` card (or the
   Reload now flash) shows the generation incremented and the key among the
   changed ones, with no last error.
4. **Verify traffic.** For a provider key, send a request through the gateway
   and read it back:

   ```bash
   systemprompt infra logs request list --since 10m --provider anthropic
   systemprompt infra logs view --level error --since 10m
   ```

5. **Revoke the old value** at the issuer once the new one is serving.

### What still needs a restart

| Secret | Why |
|---|---|
| `database_url`, `encryption_master_key`, `oauth_at_rest_pepper`, `manifest_signing_secret_seed`, `signing_key_pem`, `jwt_secret` | Pinned: a reload that changes one is refused. Rotate per `deploy/self-host/SECRETS.md`, then restart |
| Secrets an MCP server uses | Each MCP server is a child process handed its secrets at spawn; it sees the new generation when the supervisor next restarts it |

Every other consumer that reads the store when it needs a value sees the new
generation after the reload. A previous generation is kept in memory for the
life of the process (it is never zeroized), which is why reloads only swap
when a value actually changed.
