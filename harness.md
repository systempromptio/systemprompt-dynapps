# Host-integration research & test harness method

How to audit one client-host integration end to end — the code, the wire, the
enforcement claim, the install harness, the docs page, and the dashboard — and
come out with a plan that fixes what it finds.

Written up from the OpenCode audit (2026-09-05). Part 1 is the reusable method,
Part 2 is that audit's result as a worked example, Part 3 applies the method to
**Claude Code**, pre-seeded with the leads and contradictions the OpenCode sweep
already turned up.

The method exists because the OpenCode audit found that every layer had drifted
in the *same direction*: the code was more restrictive than the product needed,
the harness asserted placement rather than behaviour, the docs recommended a
command that cannot work, and the dashboard shipped a command missing the one
flag that makes the integration happen at all. None of those were visible from
inside a single layer. You have to check all five against each other.

---

## Part 1 — The method

### Rule 0: never reason about the upstream product from our own integration code

The OpenCode audit's central error, caught only because it was challenged: we
concluded "OpenCode only speaks the OpenAI wire" from reading *our*
`install/render.rs`, which hardcodes `@ai-sdk/openai-compatible`. Upstream
OpenCode bundles 19 AI-SDK providers and speaks every wire. We had mistaken our
own choice for the product's constraint, then encoded that mistake a second time
as a catalog filter.

So: **research the upstream product from upstream sources first**, before
opening our integration. Docs, source, package manifest. Cite it. Only then read
what we do, and ask why the two differ.

### The five layers, and the question each must answer

| Layer | Question | Where |
|---|---|---|
| 1. Upstream product | What does the client actually support? | vendor docs + source |
| 2. Our integration | What do we write into it, and what do we restrict? | `../systemprompt-core/bin/bridge/src/integration/<host>/` |
| 3. Wire & dispatch | Which protocols can the gateway serve it, and what filters that? | `crates/entry/api/src/services/gateway/protocol/` |
| 4. Harness | Does the test prove behaviour, or only file placement? | `scripts/clean-client-install.sh`, `playwright/` |
| 5. Surface | Do docs and dashboard describe what the code does? | `services/content/documentation/`, `extensions/web/admin/`, `storage/files/admin/templates/` |

### Running it

Launch layer 1, layer 3 and layer 5 as parallel research agents; they don't
overlap. Layer 2 folds into whichever agent reads the integration dir. Keep
layer 4 for yourself — it is short and it is where the contradictions surface.

Demand `file:line` citations and an explicit **verified vs inferred** split. The
OpenCode audit's single most important finding was an *inference* honestly
flagged as such: that OpenCode honours `/etc/opencode` above user config is an
assumption stated in our own comments and never verified upstream.

### The four questions that found every real defect

Ask these literally. They are not rhetorical.

1. **Is this restriction ours or theirs?** Trace every filter, allowlist and
   enumerated constant back to a *product* limitation. If it traces back to
   another line of our own code, it is probably circular.
2. **Does this field enforce, or only advertise?** Prove it by absence: grep
   every read site and check whether the dispatch path is among them. `surface`
   and `x-inference-protocol` both turned out to be discovery-only, so filtering
   on them protects nothing while hiding working models.
3. **What does this actually enforce, per platform?** Find the `cfg()` gates.
   "Managed tier" is a claim about the *vendor's* precedence rules, not about
   file permissions. A `0644` file the user owns enforces nothing regardless of
   which directory it sits in.
4. **Would a real user's copy-paste work?** Run the docs command and the
   dashboard command as written, on a clean box, as an unprivileged user.

### Harness smells

- Asserts a **path** where it should assert **behaviour** ("the block is in
  `/etc`" vs "the client can reach the gateway and list the right models").
- **Pre-arranges the conditions** it claims to test — `clean-client-install.sh`
  chowns `/etc/opencode` to the test user, so the "managed tier" write succeeds
  through the ordinary unprivileged path. Green, and proving nothing about
  enforcement.
- **Asserts the opposite of current behaviour** and is therefore either failing
  or not being run (see Part 3, lead A — this is live right now for Claude Code).
- Tests the **easy path around** the documented one. The harness never runs the
  `sudo` command the docs tell users to run — which is why nobody noticed it is
  broken.

### Output

One plan with: what is true (cited), what is broken, the fix per layer, and
what is deliberately out of scope with the reason. Separate core changes (land
on core's `next`, released later) from astound changes (land on `next`, ship on
next deploy) — they have different latencies to production.

---

## Part 2 — Worked example: OpenCode (2026-09-05)

Findings, as reference for what this method yields.

**Layer 1.** OpenCode bundles `@ai-sdk/anthropic`, `@ai-sdk/google`,
`@ai-sdk/openai`, `@ai-sdk/openai-compatible` and 15 more, dispatching per
provider on the config's `npm` field. `openai-compatible` is only the fallback.

**Layer 2.** `install/render.rs` hardcodes that fallback;
`accepted_surfaces() == [ApiSurface::OpenAi]` then re-encodes the same choice as
a catalog filter, hiding every `gemini-*` and `claude-*` model from the host.

**Layer 3.** `surface` is advertisement only — `resolve_upstream`
(`services/gateway/service/resolve.rs:29-108`) never reads it; dispatch keys off
`provider.wire`. `x-inference-protocol` is read in exactly one place
(`routes/gateway/models.rs`) and has no effect on any inference endpoint. Three
inbound protocols (`/v1/messages`, `/v1/chat/completions`, `/v1/responses`),
four outbound wires, **no Gemini inbound**. All 12 combinations dispatch; the
gaps are fidelity (openai-chat rendering drops thinking blocks), not refusal.
Consequence: Gemini *models* are servable to an OpenAI-wire client today, but a
native `@ai-sdk/google` block pointed at us would 404.

**Layer 4.** The harness asserts placement, not models, and pre-chowns the
managed dir. Passes while proving nothing about either enforcement or catalog.

**Layer 5.** Docs recommend `sudo astound-bridge install` twice — impossible,
since `basedirs.rs:30-32` has no `SUDO_USER` handling and reads credentials from
`/root/.config/astound/`. Docs use `gateway.example.com` with no substitution
note. Dashboard has no OpenCode anything, and its copy-paste command never
passes `--host`, relying on PATH-sniffing.

**Enforcement reality.** Linux writes `0644`, has no elevation path
(`install/managed_file.rs:126-152`), and silently downgrades to the user tier
(`install/mod.rs:65-83`). No allowlist, no lockdown, on any platform. The only
OS-backed policy in the codebase is the Windows registry key, and it is Claude's.

---

## Part 3 — Now do Claude Code

Claude Code is the functional, shipping integration — the one real users install
today, and the one the dashboard is built around. It deserves the same sweep,
and the OpenCode work already surfaced enough to know where to dig.

Run Part 1's method against host `claude-code`. Below are the seeded leads.
**Verify each before acting** — they come from an audit aimed at OpenCode, so
Claude Code was peripheral to it.

### Lead A — the harness asserts what core deliberately refuses to do

The strongest lead, and near-certain.

`scripts/clean-client-install.sh:117-141` asserts:
- `/etc/claude-code/managed-mcp.json` exists, else `[FAIL] … policy not enforced`
- `managed-settings.json` contains `allowManagedMcpServersOnly`
- `claude mcp add` is refused with `"exclusive control"`

Core's `bin/bridge/src/install/managed_mcp/mod.rs:1-11` says the opposite, in
its module head:

> Claude Code CLI enterprise MCP policy: **the bridge never writes it, and
> removes any it wrote before.** … `managed-mcp.json` puts the CLI into
> exclusive mode (plugin and user servers vanish) and
> `allowManagedMcpServersOnly` … denies every server not on the allowlist —
> including Cowork's built-in workspace server, so the sandbox bash tool and
> every skill that needs it stop working.

`clear_policy()` is called on every enrolment
(`integration/claude_code_cli/mod.rs:125,169`).

**Resolved 2026-09-05: the harness was red.** A baseline run against production
exited 1 on exactly those two assertions:

```
[FAIL] /etc/claude-code/managed-mcp.json absent — policy not enforced
[FAIL] a user could still add an MCP server
```

They have been inverted to assert the real contract — no `managed-mcp.json`, no
`allowManagedMcpServersOnly`, the user *can* still add a server, and the
`systemprompt` MCP server *is* in the loaded set. A second session confirmed the
rewrite runs green.

The generalisable lesson: an assertion that contradicts the product will sit red
(or unrun) indefinitely, because a red harness teaches people to stop running
it. When a harness and a module doc disagree, the module doc is usually the
contract and the harness is usually the fossil.

#### How the `systemprompt` MCP server actually reaches each client

Since the removed policy is what the old assertions described, here is what
replaced it. Both clients end up pointed at the same loopback URL by different
files.

**Claude Code** gets it from a **per-plugin `.mcp.json`**, written by the bridge
into each mirrored plugin directory
(`bin/bridge/src/integration/claude_code_cli/bundle.rs:110-132` in core). Only
plugins that reference the server get one: the server projects a plugin's
`mcp_servers.include` list into a bundled `.mcp.json`
(`crates/domain/marketplace/src/bundle/mcp.rs`), the bridge reads the server
names out of that file, deletes it, and rewrites its own pointing at the
loopback proxy (`sync/apply/plugin.rs:108-125`). A plugin whose include list does
not name `systemprompt` never gets the entry. Claude-side the tool is
`mcp__systemprompt__systemprompt`.

**OpenCode** gets it from `~/.config/opencode/opencode.json`, as an
`mcp.systemprompt` entry of `"type": "remote"`
(`bin/bridge/src/integration/opencode/managed_resources/config_json.rs:19-53`).
Foreign keys survive; only entries whose URL points at the loopback origin are
bridge-owned and rewritten. OpenCode names tools `<server>_<tool>`, so the same
tool is `systemprompt_systemprompt` there — that spelling is OpenCode's own
convention and is not written by any of our code.

Both resolve to `http://127.0.0.1:48217/mcp/systemprompt`
(`proxy/mod.rs:32` for the default port, `proxy/loopback.rs:49-51` for the path),
with the loopback bearer on the `Authorization` header. So the two clients differ
only in which config file carries the entry and how the tool is named; the
transport, the port and the auth are identical.

### Lead B — Claude Code is not a `HostApp` — RESOLVED, and the answer inverts

Unlike `opencode`, `codex-cli`, `hermes` and `claude-desktop`, `claude-code` is
registered only as a `HostSync` (`integration/claude_code_cli/mod.rs:188`, absent
from the `register_host_app!` list at `integration/registry.rs:70-75`). It has
**no `accepted_surfaces()`** and no provider block. Discovery runs through
`install/mdm/linux/settings.rs:64-127`, which writes `apiKeyHelper`,
`env.ANTHROPIC_BASE_URL`, `CLAUDE_CODE_ENABLE_GATEWAY_MODEL_DISCOVERY=1` and
`CLAUDE_CODE_ATTRIBUTION_HEADER=0`.

**Resolved 2026-09-05, measured not inferred.** The `x-inference-protocol`
header is written only on the `HostApp` path, from `host.accepted_surfaces()`
(`integration/reapply.rs:96-106`, and only `if !surfaces.is_empty()`). Nothing
writes it for `claude-code`. A cold-start run confirmed the loopback proxy's
`/v1/models` returns all 26 ids including `gemini-2.5-pro`, `gemini-2.5-flash`
and `gemini-2.5-flash-lite` — the exact path Claude Code's discovery uses
(`ANTHROPIC_BASE_URL` = the loopback origin, `install/apply.rs:132`).

So the finding inverts: **Claude Code sees the whole catalogue precisely because
it is not a `HostApp`.** OpenCode sees a filtered 9 because it is one. The
surface filter only harms the hosts that implement it — which is an argument for
removing the filter rather than for extending it to Claude Code.

An earlier draft of this file inferred "Claude Code sends `anthropic`" from the
14-model measurement of `x-inference-protocol: anthropic`. That inference was
wrong; the measurement was of the endpoint, not of any client. Correcting it
cost nothing here, but it is exactly the Rule 0 mistake one layer over: do not
attribute a behaviour to a client without tracing who writes it.

### Lead C — `BridgeProfileResponse.models` is hardcoded to one surface (real, but not Claude Code's path)

`crates/shared/models/src/bridge/profile.rs:115` sets the profile's model list to
`advertised_model_ids(&[ApiSurface::Anthropic])` — a **hardcoded constant, not a
host-derived value**. Every consumer of the bridge profile therefore sees
Anthropic-surface models only.

Per Lead B this does **not** feed Claude Code's picker, which reads `/v1/models`
through the loopback proxy. The hardcode is still real and still worth a
verdict: find what does consume it, and decide whether the constant is
deliberate or the same class of mistake as `accepted_surfaces()`.

### Lead D — enforcement, honestly stated

Per platform, verified during the OpenCode sweep:

- **Linux**: `/etc/claude-code/managed-settings.json` *only if writable*, else
  falls back to `~/.claude/settings.json` (`install/mdm/linux/settings.rs:20-31`).
  No enforcement.
- **macOS**: `.mobileconfig` / managed preferences (`mdm/macos.rs`,
  `macos_payload.rs`), with a `sudo`/`osascript` elevation path.
- **Windows**: `HKLM\SOFTWARE\Policies\Claude` (`mdm/windows_policy.rs`) — the
  **only** genuine OS-backed, user-unoverridable policy tier in the codebase.

Check what `connect-claude-code.md` and `bridge-install.md` claim about this. If
either implies Linux enforcement, it is wrong in the same way the OpenCode page
was. Governance is real at the gateway; it is not real in a `0644` file.

### Lead E — the `sudo` defect probably applies here too

`basedirs.rs:30-32` (`env_dir("HOME").or_else(dirs::home_dir)`, no `SUDO_USER`)
is shared code, not OpenCode-specific. Any Claude Code instruction telling a
user to run the bridge under `sudo` has the same problem: credentials read from
`/root/.config/astound/`. Grep both docs pages and the dashboard templates for
`sudo`.

### Lead F — dashboard surface

`extensions/web/admin/src/services/bridge_profile/mod.rs:128-138` builds the
copy-paste command with no `--host`. For Claude Code this happens to work — it
is the PATH-sniffing default in `scripts/install-bridge.sh:329-331` — so the
command is correct today by accident rather than by intent. Decide whether to
make it explicit.

Also verified broken, and Claude-Code-facing:
`storage/files/admin/templates/bridge-setup.hbs:58` says
`systemprompt-bridge login sp-live-YOUR_TOKEN` — wrong binary for this brand
(`bridge_profile/mod.rs:86 BRIDGE_BINARY` is `astound-bridge`). Line 66's Linux
snippet has no `--host`. Fix these regardless of what the rest of the sweep
concludes; they are unambiguous.

`storage/files/admin/templates/profile.hbs` is the main connect card and is
already Claude-Code-shaped — check its verification copy ("Claude Code's header
should name this gateway. If it says `API Usage Billing`…") still matches the
shipped client's behaviour.

### Lead G — docs, against the code

`services/content/documentation/connect-claude-code.md` is the better-structured
of the two pages and was used as the model for fixing the OpenCode one. Audit it
anyway against `install-bridge.sh` and
`../systemprompt-core/bin/bridge/src/integration/claude_code_cli/`, specifically:

- the settings keys we actually write (Lead B's list) — are they documented?
- `astound-bridge doctor` output and exit code 11, still accurate?
- the MCP story: does the page describe per-plugin `.mcp.json` delivery, or the
  exclusive-mode policy that core removed?
- runtime dependencies, checksum verification, WSL systemd caveat — still true?

`bridge-install.md` covers per-OS install and cross-sells `--host`; keep the two
consistent.

Also note `codex-cli` is a registered host with **no documentation page at all**
and no dashboard presence. Out of scope here; worth its own ticket.

### The three harnesses, and what each one can prove

- **`just e2e`** — the Playwright suite: the functional and requirement specs, and
  a page spec per admin page carrying all four mandatory blocks (renders, actions,
  authorization, design language). It runs against a stack you started, seeds a
  deterministic dataset, and signs in as each of the seven principals.
  `just e2e-demo` re-runs the same dataset as a narrated walk and renders the demo
  book. `playwright/README.md` is the reference.

- **`just e2e-wires`** — curl against the gateway. Hits `/v1/models`, then all
  three inbound surfaces, buffered **and** streaming, and drives the real
  `systemprompt` tool schema rather than a hand-written stand-in. It proves the
  wire: request translation, terminal signals, usage and the audit row.
- **`just e2e-opencode`** and **`just e2e-claude-code`** — real client agent
  turns, every model in the catalog. A cell passes only if the work shows up in
  `mcp_tool_executions` and `ai_request_tool_calls`. Nothing is inferred from the
  client's stdout, which is the mistake §6c of `tech-debt.md` describes.
- **`just e2e-marketplaces`** — the same two clients, once per seeded bridge
  principal, asserting the entitlement rather than the model: the marketplace
  dirs, `installed_plugins` keys, skill dirs and MCP entries on disk are compared
  as exact sets against `playwright/setup/seed/entitlements.json`, the
  knowledge-bank tool is called (entitled) or refused by the proxy (not), and a
  real turn must reach `search_knowledge` — proven by rows for that user id.
  The three share `scripts/lib/e2e-client.sh` (throwaway HOME, proxy port
  guard, enrolment, the MCP pre-flight dance, the proof queries).

The honest constraint on the two client harnesses: **`claude -p` and
`opencode run` always stream.** Neither client offers a buffered mode, so no
client-driven test can exercise the buffered path. Buffered is proven at the
wire, by `e2e-wires`, and the split is deliberate rather than an oversight. Read
a green client run as "streaming works end to end through a real agent", not as
"both modes work".

### Verification for any change that comes out of this

1. `just vendor-core` first — `next` builds against the vendored core, and the
   build log must name `.vendor/systemprompt-core` paths or the patch silently
   dropped and the build proved nothing.
2. `just verify` mid-iteration, `just preflight` before landing.
3. `just clean-client-install "$PAT" https://astound.systemprompt.io` — capture
   the before state first (Lead A), so the diff is legible.
4. Drive one real prompt through Claude Code against the gateway, then confirm
   from the host: `systemprompt infra logs request list --limit 5` and
   `systemprompt infra logs audit <request-id>` for identity → policy → cost.
5. Docs go live only via `just publish` (`publish_pipeline`); editing the `.md`
   changes nothing on the running site.
6. Commit on `next`, never `main`. Peers share this clone's index, so
   `git commit --only <paths>`. Core changes commit on core's `next` and push —
   run no validation in the core checkout.
