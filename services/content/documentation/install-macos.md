---
title: "Install on macOS"
description: "Install the signed Astound Bridge on macOS, sign in with the device link, and route Claude Desktop and Claude Code through the gateway — with or without touching your own Claude Code settings."
author: "Astound Digital"
slug: "install-macos"
keywords: "macos, install, dmg, astound bridge, notarized, gatekeeper, claude desktop, claude code, cowork, device link, managed preferences, apiKeyHelper, --settings"
kind: "guide"
public: true
tags: ["documentation", "getting-started", "bridge", "macos", "claude-code"]
published_at: "2026-09-08"
updated_at: "2026-09-09"
after_reading_this:
  - "Install the signed, notarized Astound Bridge from the gateway's own download"
  - "Sign in with the device link and verify the bridge with doctor"
  - "Route Claude Desktop (Cowork) through the gateway with managed preferences"
  - "Route Claude Code through the gateway, either always or per terminal session"
  - "Undo any part of it without touching the rest"
related_playbooks:
  - title: "Install the Desktop Bridge (all platforms)"
    url: "/documentation/bridge-install"
  - title: "Downloads"
    url: "/documentation/downloads"
  - title: "Connect Claude Code"
    url: "/documentation/connect-claude-code"
---

# Install on macOS

macOS is fully supported: the Astound Bridge ships as a universal app (Apple
Silicon and Intel) that is Developer ID signed, notarized and stapled, and so
is the disk image it comes in. Gatekeeper accepts both; macOS may still ask
you to confirm opening an app downloaded from the internet. This
page is the macOS-only walk-through; the cross-platform reference is
[Install the Desktop Bridge](/documentation/bridge-install).

Replace `https://gateway.example.com` with your instance's URL throughout.
Astound staff on the corporate network use `https://sp-dev.astound.digital`,
which is also the gateway the app defaults to.

## 1. Download and install

1. Sign in at `https://gateway.example.com/bridge-auth/setup` — the page
   detects macOS and offers **Download for macOS**. The same file is on the
   public [Downloads](/documentation/downloads) page.
2. Optionally verify the checksum against the published sidecar:

   ```bash
   shasum -a 256 -c astound-bridge-macos.dmg.sha256
   ```

3. Open the disk image, drag **Astound Bridge** to Applications, eject the
   image, and launch the app from Applications. Do not run it from the mounted
   image: the updater and the install record need the app at
   `/Applications/AstoundBridge.app`.

To check the signature yourself:

```bash
spctl -a -vv /Applications/AstoundBridge.app      # source=Notarized Developer ID
```

The CLI is inside the bundle. Either call it by path or put it on `PATH` once:

```bash
alias astound-bridge=/Applications/AstoundBridge.app/Contents/MacOS/astound-bridge
```

## 2. Sign in

Click **Sign in** in the app window (or the menu-bar icon). The bridge opens
the gateway's device-link page in your browser; approve it with your work
account and the bridge stores a durable credential under
`~/Library/Application Support/astound/`. Sign-in is interactive by design —
proving identity in the browser is the point — so it needs the app window or a
terminal:

```bash
astound-bridge login                              # default gateway
astound-bridge login --gateway https://other.example.com
astound-bridge login --code <code>               # admin-issued code, unattended
```

Then check everything the bridge can check:

```bash
astound-bridge doctor
```

You want `0 fail`. The two warnings a fresh install shows — no install record,
no hook OAuth client yet — clear after step 3 and the first plugin hook.

## 3. Route your clients

The app runs a loopback inference proxy on `127.0.0.1:48217`. Clients talk
to that proxy with a machine-local secret; the proxy swaps it for a short-lived
gateway token and forwards. Nothing on your Mac ever holds a provider API key.

### Claude Desktop (Cowork)

```bash
astound-bridge install --apply --host claude-desktop
```

This writes the Claude Desktop managed preferences
(`/Library/Managed Preferences/com.anthropic.claudefordesktop.plist` and your
per-user copy): gateway mode against the proxy, the model list, and the
organisation's managed MCP servers. It asks for administrator approval once
and also offers a configuration profile under **System Settings › General ›
Device Management**; approve it there. Fully quit and relaunch Claude Desktop
so it re-reads the policy. Your first message is audited on the gateway.

### Claude Code (terminal) — always routed

```bash
astound-bridge install --apply --host claude-code
```

This merges three keys into your existing `~/.claude/settings.json` —
`env.ANTHROPIC_BASE_URL`, `env.CLAUDE_CODE_ENABLE_GATEWAY_MODEL_DISCOVERY`
(plus `CLAUDE_CODE_ATTRIBUTION_HEADER`) and `apiKeyHelper` — and nothing else;
your permissions, model choice, plugins and hooks stay as they were. Every
`claude` session in any terminal now routes through the gateway. Undo it with
`astound-bridge uninstall --host claude-code`, which removes exactly those keys.

While the gateway credential is active Claude Code disables claude.ai
connectors ("another auth source is set"). That is expected: the gateway is
your provider.

### Claude Code — per session, leaving your settings alone

If you keep your own Anthropic login for personal work and only want *some*
sessions governed, skip `--host claude-code`. Every `install --apply` also
writes a standalone settings file containing just the gateway keys:

```
~/Library/Application Support/astound/claude-code-settings.json
```

Pass it to Claude Code for the sessions you want routed:

```bash
claude --settings "$HOME/Library/Application Support/astound/claude-code-settings.json"
```

`--settings` layers that file over your user and project settings for that
run only; `~/.claude/settings.json` is never modified. A shell alias
(`alias claude-gw='claude --settings "$HOME/Library/Application Support/astound/claude-code-settings.json"'`)
turns it into a toggle: `claude` is yours, `claude-gw` is governed.

Skills, plugins and MCP servers arrive either way: `astound-bridge sync`
registers the organisation marketplace with Claude Code independently of how
inference is routed.

Before the first sync, obtain the manifest signing public key from your
administrator through a trusted channel and pin it:

```bash
astound-bridge install --pubkey '<administrator-provided-base64-public-key>'
astound-bridge sync
```

A fresh install without a pinned key refuses sync; successful login alone
does not establish manifest trust. `sync --allow-tofu` explicitly trusts the
key presented by the gateway on first use. Use that only for a controlled
test gateway or when your administrator has approved that trust policy, not
as a general way to bypass verification. Subsequent syncs use the pinned key.
On macOS, the first sync may ask for administrator approval to write the
organisation plugins. Complete that prompt before expecting sync to finish.

### Non-Claude models (Gemini, Vertex)

Claude Code's `/model` picker lists gateway models whose id contains `claude`
or `anthropic`; it hides everything else the gateway serves. The bridge fills
that gap on every `sync`: it reads the gateway's provider catalog and writes
`modelPicker.options` rows (with `model` and `label` fields) for the other
models into the settings it owns — the
standalone fragment always, and `~/.claude/settings.json` only when
`--host claude-code` is enrolled. So `Gemini 2.5 Flash`, `Vertex Gemini 2.5
Pro` and friends appear in the picker, and `--model gemini-2.5-flash` works
from the command line too.

The original 0.48.0 bridge wrote an array that Claude Code 2.1.263 ignores. <!-- pinned-release -->
The corrected format is on `next`; a gateway must ship the corrected bridge
before picker support can be assumed. If Claude reports a `modelPicker`
settings warning, upgrade the bridge and sync again. Until then, specify the
model with `--model`; do not mistake successful inference for a working picker.

To see the raw catalog:

```bash
curl -s -H "Authorization: Bearer $(cat "$HOME/Library/Application Support/astound/bridge-loopback.key")" \
  http://127.0.0.1:48217/v1/models | jq -r '.data[].id'
```

This applies to Claude Code only. Claude Desktop is given Anthropic models
exclusively — it does not support other families — and the bridge enforces
that in the managed policy it writes.

## 4. Verify end to end

```bash
astound-bridge doctor                              # 0 fail
claude -p 'Reply with exactly PONG'               # or a claude-gw session
```

An administrator sees the request within seconds:

```bash
systemprompt infra logs request list --limit 5
```

If `claude` answers but nothing appears there, it is not routed — its header
says *API Usage Billing* instead of naming the gateway. Re-run
`astound-bridge install --apply --host claude-code`, or use `--settings` as
above.

## Where things live

| Piece | Where |
|-------|-------|
| App + CLI | `/Applications/AstoundBridge.app` (`Contents/MacOS/astound-bridge`) |
| Bridge config, credential, loopback secret | `~/Library/Application Support/astound/` (respects `XDG_CONFIG_HOME`) |
| Standalone Claude Code fragment + key helper | `~/Library/Application Support/astound/claude-code-settings.json`, `claude-key-helper.sh` |
| Claude Desktop policy | `/Library/Managed Preferences/com.anthropic.claudefordesktop.plist` (+ per-user copy) |
| Claude Code user settings (only with `--host claude-code`) | `~/.claude/settings.json` |
| Organisation plugins, skills, MCP servers | `/Library/Application Support/Claude/org-plugins/`, marketplace under `~/.claude/plugins/` |
| Sync state | `~/Library/Application Support/astound-bridge/` |

## Uninstall

```bash
astound-bridge uninstall --host claude-code        # just the Claude Code keys
astound-bridge uninstall --host claude-desktop     # just the Desktop policy
astound-bridge uninstall                           # bridge integration, credentials kept
astound-bridge uninstall --purge                   # everything, including credentials
```

Then drag `/Applications/AstoundBridge.app` to the Trash.

## Troubleshooting

| Symptom | Cause |
|---------|-------|
| "Your apiKeyHelper script is failing", exit 127 | An older bridge wrote the helper path unquoted and macOS paths contain a space. Re-run `install --apply --host claude-code` with the current bridge; it writes the quoted form. |
| `login` says it needs a terminal | Device-link sign-in is interactive. Use the app window, a real terminal, or `--code`. |
| `update --check` reports 404 | The gateway has no `bridge_releases` feed yet. Download the current build from the gateway instead. |
| Claude Desktop ignores the gateway | It was running when the policy was written. Fully quit (Cmd+Q) and relaunch; check `defaults read /Library/Managed\ Preferences/com.anthropic.claudefordesktop inferenceProvider` says `gateway`. |
| Two Astound Bridge apps | `open -a AstoundBridge` can pick a stale copy. Launch `/Applications/AstoundBridge.app` by path and remove other copies. |
