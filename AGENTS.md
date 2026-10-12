# AGENTS.md — systemprompt-dynapps

Dynapps's branded fork of the [systemprompt.io](https://systemprompt.io) evaluation template. Stand up a complete AI governance binary with demo data in under 10 minutes.

## What this is

An evaluation template, not a library. Clone, build, run. You get a self-hosted AI governance system with pre-configured agents, MCP servers, skills, scheduled jobs, and a static site. The binary is ~50 MB, needs only PostgreSQL, runs air-gapped.

It wraps [`systemprompt-core`](https://github.com/systempromptio/systemprompt-core) — the Rust crate that does the governance — with production-ready configuration, demo scripts, and a compile-time extension system.

## Try it

```bash
git clone https://github.com/systempromptio/systemprompt-dynapps
cd systemprompt-dynapps
just setup-local <your-anthropic-api-key>   # builds, runs migrations, starts Postgres
just start                                  # serves on http://localhost:8080
```

Then walk the scripts in `demo/` in order — they exercise the full governance pipeline via the CLI.

## Where things live

| Path | Contents |
|------|----------|
| `demo/` | Setup + seeding harness (preflight, seed data, sweep) |
| `services/` | Config-as-code (YAML + Markdown): agents, AI providers, MCP servers, plugins, skills, scheduler |
| `extensions/` | Compile-time Rust extensions (MCP servers + web rendering) |
| `storage/`, `content/` | Static assets and Markdown content |
| `docs/` | Release process + gateway integration guides |
| `deploy/` | The clean-client test container |
| `src/main.rs` | Thin binary entry point — delegates to `core/` |

Full developer guidance for working in this repo lives in [`CLAUDE.md`](./CLAUDE.md).

## Working agreement (owner instruction, 2026-09-07)

- Never stash changes. Commit work in place and resolve conflicts forward on `next`.
- Work directly on `next`. Do not create separate task/feature branches or draft
  PRs unless the owner explicitly requests that exception.
- Integrate existing task work into `next`, preserving concurrent changes, and
  remove temporary branches once their commits are safely merged.
- The live Dynapps endpoint is `https://dynapps.systemprompt.io`. Do not confuse
  it with the older Fly `production` profile. A registry push is not proof of
  deployment: verify the binary downloaded from the live endpoint itself.
- `https://dynapps.systemprompt.io` is our test environment. Its local profile
  is named `production`, and `just deploy` targets it. The owner permits
  deploying current `next` there for end-to-end testing; this does not deploy
  the separate Dynapps endpoint above. Preserve existing secrets and signing
  keys when restoring deployment access.

## Governance pipeline

Every tool call passes through four synchronous stages before execution: **scope check → configured secret scan → blocklist → rate limit**. Every decision is logged as a structured JSON event with a `trace_id` linking identity → agent → tool call → result → cost.

## Key facts

- Single Rust binary, ~50 MB; PostgreSQL is the only runtime dependency.
- Provider-agnostic: Anthropic, OpenAI, Gemini, local models.
- MCP-native — governance is the transport, not a proxy.
- Identity: `users.roles` is a free-text `TEXT[]` array; `admin` is just a role string. System-originated work (scheduled jobs, hooks, MCP) runs under an explicit `owner:` declared per resource in `services/scheduler/config.yaml`. The owner is a real admin user — there is no separate "system" user. Every governance audit row carries `(user_id, actor_kind, actor_id)`: see `services/content/documentation/authentication.md`.
- Compile-time extension model via the [`inventory`](https://docs.rs/inventory) crate.
- Core pins live in **two** manifests (`Cargo.toml`, `tests/Cargo.toml`). A stale pin silently disables `[patch.crates-io]` and builds against crates.io instead — see `docs/RELEASING.md` Step A0 before upgrading core.
- Configuration is YAML under `services/`, not a database UI.
- Air-gap capable.

## Licensing

Fork it, modify it, ship it.

## Links

- This fork: https://github.com/systempromptio/systemprompt-dynapps
- Upstream template: https://github.com/systempromptio/systemprompt-template
- Core: https://github.com/systempromptio/systemprompt-core
- Dynapps: https://dynapps.es
- Feedback: open a `feedback`-labelled issue, or email `ed@systemprompt.io`
