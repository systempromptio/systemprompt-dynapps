---
title: "Connect OpenCode"
description: "Route OpenCode through the Astound gateway: one installer flag writes the provider block and the API key, so every prompt is governed and audited instead of being refused with 403."
author: "Astound Digital"
slug: "connect-opencode"
keywords: "opencode, connect, install, bridge, openai compatible, provider, auth.json, loopback proxy, 403, no loopback credential presented, mcp"
kind: "guide"
public: true
tags: ["documentation", "getting-started", "opencode"]
published_at: "2026-09-04"
updated_at: "2026-09-04"
after_reading_this:
  - "Connect OpenCode to the gateway with one installer flag"
  - "Know which files the bridge writes and which tier each lands in"
  - "Diagnose a 403 from the loopback proxy without guessing at credentials"
related_playbooks:
  - title: "Connect Claude Code"
    url: "/documentation/connect-claude-code"
  - title: "Install the Desktop Bridge"
    url: "/documentation/bridge-install"
  - title: "Gateway API"
    url: "/documentation/gateway-api"
---

# Connect OpenCode

[OpenCode](https://opencode.ai/) talks the OpenAI-compatible wire, which the
gateway serves. The bridge registers it as a provider called `systemprompt`
pointed at the local loopback proxy, so OpenCode never holds a model vendor's
key and every request is governed, priced, and audited like any other.

This page covers Linux. On macOS and Windows the same command works; only the
paths differ, and they are listed below.

## Install and enrol in one command

```bash
curl -fsSL https://gateway.example.com/files/downloads/install.sh | sh -s -- \
  --download-base https://gateway.example.com/files/downloads \
  --code <code> --host opencode
```

Get the one-shot code from your profile page, or ask an administrator to mint
one. It expires in ten minutes, so run the installer first and take the code
last.

`--host` is repeatable and accepts a comma-separated list, so a machine that
runs both clients takes `--host claude-code,opencode`. Left off entirely, the
installer enrols Claude Code and adds OpenCode when the `opencode` binary is
already on your `PATH`. The same list can come from `$ASTOUND_HOSTS` for
unattended provisioning.

Already have the bridge installed? Enrol OpenCode on its own, without
reinstalling anything:

```bash
astound-bridge install --host opencode
```

## What the bridge writes

| Piece | Where | Tier |
|---|---|---|
| Provider block (`provider.systemprompt`) and the default `model` | `/etc/opencode/opencode.json` | admin-owned |
| Provider block, when `/etc` is not writable | `~/.config/opencode/opencode.json` | your own |
| API key for the `systemprompt` provider | `~/.local/share/opencode/auth.json` (`0600`) | your own |
| MCP connectors (`mcp.<name>`) | `~/.config/opencode/opencode.json` | your own |
| Managed skills | `~/.config/opencode/skills/` | your own |

The provider block belongs in the admin tier because OpenCode layers its config
with the managed directory above every user and project file: written there, no
personal config can route inference around the gateway. Where the bridge cannot
write `/etc/opencode` and has no way to ask for elevation — the ordinary
single-user Linux install — it writes the provider block to your own config
instead and says so. That still works, but it is not governance: you can edit
it. To keep the strong tier, run the enrolment once as root:

```bash
sudo astound-bridge install --host opencode
```

On macOS the admin tier is `/Library/Application Support/opencode/`, and on
Windows `%ProgramData%\opencode\`; the API key sits beside them under the
user's own `.local/share/opencode/` and `%USERPROFILE%\.local\share\opencode\`.

Every foreign key in these files survives. The bridge owns the
`provider.systemprompt` object, the top-level `model` when it names our
provider, and the MCP entries whose URL points at the loopback proxy. It
rewrites those and nothing else, so a config you wrote by hand is preserved
across syncs.

## Verify

The loopback proxy authenticates every request with a bearer token that is
local to this machine. Ask it for the model list:

```bash
curl -s -H "Authorization: Bearer $(cat ~/.config/astound/bridge-loopback.key)" \
  http://127.0.0.1:48217/v1/models
```

A JSON list of model ids means the proxy is up, your gateway credential is
valid, and the wire OpenCode uses is answering.

Then confirm OpenCode itself sees the provider:

```bash
grep -A3 '"systemprompt"' ~/.config/opencode/opencode.json /etc/opencode/opencode.json 2>/dev/null
test -s ~/.local/share/opencode/auth.json && grep -q systemprompt ~/.local/share/opencode/auth.json && echo "api key present"
```

## Choosing a model

Models are namespaced by provider, so they are written `systemprompt/<model>` —
for example `systemprompt/claude-sonnet-5`. Enrolment sets a default for you and
writes the list the gateway currently offers, so OpenCode's model picker shows
exactly the models your role is allowed. Ask for one that is not on that list
and the gateway refuses it rather than silently substituting another.

## Troubleshooting

| What you see | What it means | What to do |
|---|---|---|
| `403 forbidden: no loopback credential presented` | OpenCode reached the proxy with no `Authorization` header. Almost always the provider block or `auth.json` was never written: `sync` installs MCP connectors and no credential, so the connectors appear and inference does not. | `astound-bridge install --host opencode`, then restart OpenCode. |
| `403 forbidden: bad loopback secret` | The credential is real but belongs to a different bridge install answering on this port — most often Windows and WSL2 sharing `127.0.0.1:48217`. The body names the config directory of the install that answered. | Decide which install owns the port, stop the other, then re-run the enrolment from the one you keep. |
| MCP servers are listed but every prompt fails | The same missing-credential case as the first row; the two are written by different steps. | As above. |
| `connection refused` on `127.0.0.1:48217` | The proxy is not running. | `astound-bridge proxy &`, or `systemctl --user start astound-bridge-proxy.service`. |
| The model list is empty, or a model is rejected | Your role does not reach that model, or the manifest has not synced yet. | `astound-bridge sync`, then check the list with the `curl` above. |
| Enrolment says the instance does not enable this host | OpenCode is not in the host list your instance publishes for you. | Ask an administrator to enable the `opencode` host, then re-run the enrolment. |
| The provider block landed in your own config, not `/etc` | `/etc/opencode` was not writable and there was no elevation to offer. | Works as is; re-run with `sudo` to move it to the admin tier. |

`astound-bridge doctor` reports the proxy, the credential, and the synced
manifest in one pass, and is the fastest way to tell a local problem from a
gateway one.

## Removing it

Un-enrol OpenCode and leave everything else in place:

```bash
astound-bridge uninstall --host opencode
```

This strips the bridge-owned provider block from both tiers and removes the
`systemprompt` entry from `auth.json`, leaving every other provider and every
key you added untouched. If the provider block sits in the admin tier, run it
with `sudo`.

To remove the bridge itself, `astound-bridge uninstall` with no `--host`.

---

*Applies to Astound Digital @RELEASE_VERSION@ and later, which is the first
release whose installer enrols client applications.*
