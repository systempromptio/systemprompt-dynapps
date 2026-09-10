<picture>
  <source media="(prefers-color-scheme: dark)" srcset="storage/files/images/logo-white.svg">
  <source media="(prefers-color-scheme: light)" srcset="storage/files/images/logo.svg">
  <img src="storage/files/images/logo.svg" alt="Astound Digital" width="380">
</picture>

# Astound Digital AI platform

Astound Digital's deployment of SystemPrompt. This repository contains the server configuration, admin interface, MCP integrations, skills, knowledge bank, requirements workflow and branded desktop Bridge. The server links the `systemprompt` Rust library; Astound-specific behavior lives in the extension crates and service configuration.

This README covers local development and points to installation and release procedures. Requests sent through the gateway and connected MCP services are subject to their configured access rules and audit handling. Local client actions require the corresponding client integration.

## Choose an installation path

- **Use the deployed platform:** start at https://sp-dev.astound.digital and follow its sign-in and client setup instructions.
- **Install a released server:** follow [Docker installation](docs/install/docker.md). Access to the private GitHub package is required; no Rust build is needed.
- **Develop locally:** use the steps below. Building the Bridge is a separate, optional step.
- **Operate your own deployment:** see [single-node installation](deploy/self-host/SINGLE-NODE.md) or [multi-node installation](deploy/self-host/README.md).

## Local server setup

You need repository access, Git, Docker with Compose, [just](https://just.systems/), Bash, `curl`, `jq`, Python 3 and Rust installed through [rustup](https://rustup.rs/). The workspace requires Rust 1.96+ and pins its toolchain in [rust-toolchain.toml](rust-toolchain.toml). Provide an Anthropic, OpenAI or Gemini API key for inference.

```bash
git clone https://github.com/Astound-Digital/systemprompt-astound
cd systemprompt-astound
just setup-local
just start
```

`setup-local` prompts for a provider, creates `.systemprompt/profiles/local/`, starts PostgreSQL 18 with pgvector, builds the release workspace, runs migrations, bootstraps an admin and publishes web assets. HTTP defaults to port `8080`; PostgreSQL defaults to `5432`. Open **http://localhost:8080/admin/login** after startup.

The bootstrap admin email defaults to `git config user.email`, then `admin@localhost.localdomain`. To choose it explicitly, or run another clone on different ports:

```bash
just setup-local <anthropic_key> "" "" 8081 5436 you@example.com
```

The positional arguments are Anthropic key, OpenAI key, Gemini key, HTTP port, PostgreSQL port and admin email. The first supplied provider key becomes the default provider.

On Linux, setup can fetch a matching release using an authenticated `gh` CLI before provisioning. **The current recipe still runs a release build later**, so install Rust for this path. Use the published Docker image for installation without compiling.

The knowledge bank requires pgvector and otherwise prevents startup. Local setup supplies the correct database image. Its default retrieval provider runs locally without model credentials; see [knowledge database setup](docs/install/knowledge-db.md) and [model-provider configuration](docs/install/knowledge-model-provider.md).

Additional workflows have their own dependencies. Before using the requirements workflow outside Docker, follow its [runtime setup](services/requirements-workflow/README.md), including Node dependencies, the approval signing seed and evaluator credentials.

## Connect Claude Code locally

Build the Bridge **before** generating a short-lived connect code:

```bash
just bridge-build
```

The Bridge is a separate Cargo workspace and depends on an unpublished Core crate at `../systemprompt-core`. The recipe clones Core if absent and checks a clean existing checkout against `bridge/CORE_REF`. Resolve a reported mismatch before building; do not replace another developer's local changes.

Sign in or register at `/admin/login`, subject to the configured authentication and email-domain rules. Open `/admin/profile` and generate a connect code, then run:

```bash
just claude <code>
# For a gateway on another port:
just claude <code> http://localhost:8081
```

This runs Claude Code in a container without configuring the host's Claude installation. Credentials persist in storage scoped to the clone and gateway; subsequent runs can use `just claude`. For a non-default gateway, use `just claude "" http://localhost:8081`.

`just claude-reset` clears this clone's client state so the next run exercises sign-in again. A successful fresh connection must report that it is signing in with the supplied code, rather than reusing a stored credential.

For host installation and desktop clients, see [Bridge documentation](bridge/README.md). `just connect <code>` installs on the host and changes its client configuration and service setup.

## Repository layout

| Path | Contents |
|---|---|
| `src/` | Thin server entry point and extension imports. |
| `extensions/` | Rust web, CLI and MCP capabilities. |
| `services/` | Provider routing, access rules, MCP servers, skills, plugins and workflow configuration. |
| `bridge/` | Branded client, built separately from the server. |
| `storage/` | Web assets and local content; runtime knowledge files are gitignored. |
| `tests/` | Rust test workspace. |
| `deploy/`, `docs/` | Installation, operations and release procedures. |

`services/config/config.yaml` lists the flat resource files. Skills and plugins are discovered from their nested directories. Not every marketplace has to live here: the Europe development workspace (`astound-europe-dev`) is authored in [Astound-Digital/sfnext-claude-dev-kit](https://github.com/Astound-Digital/sfnext-claude-dev-kit) and arrives at boot as a signed services bundle pulled from GHCR. It is **not pinned in this release** — pin one with `just services-pin <sha256 digest>` to bring it back; see [docs/sfnext-marketplace.md](docs/sfnext-marketplace.md). Deployment profiles under `.systemprompt/profiles/` hold environment-specific settings and secret references. See [CLAUDE.md](CLAUDE.md) for development conventions and [gateway routes](docs/gateway-routes.md) for inference configuration.

## Build and verify changes

```bash
just build-status     # inspect shared build state before compiling
just server-status    # inspect the running server and binary
just build            # debug build; use --release for a release build
just publish          # publish web assets
just verify           # static checks, lint and Rust tests
just preflight        # release gate, including coverage and browser checks
just stop             # stop local services cleanly
```

Embedded public-site templates require a rebuild and server restart before publishing. Use [CLI help](docs/README.md) through `systemprompt --help` to discover operational commands. The [demo directory](demo/README.md) contains preflight and data-seeding helpers; use those only on an instance intended for demo data.

[![Coverage 75.78%](https://img.shields.io/badge/coverage-75.78%25-f97316?style=flat-square)](coverage/baseline.json)

## Core versions and releases

Work on `next`; `main` is the release branch. **The checked-in patch blocks are currently commented out**, so the server resolves Core from crates.io. Check both `Cargo.toml` and `tests/Cargo.toml` when changing dependency mode; the branch name alone does not establish which Core is built.

For unreleased Core development, enable both patch blocks and run `just vendor-core`. Server patches resolve to `.vendor/systemprompt-core`; the test workspace uses the corresponding relative path. Vendoring copies the sibling checkout, including uncommitted source changes. Version pins in both workspaces must match that Core version, or Cargo can ignore the patch. `bridge/CORE_REF` separately pins the Core revision used by CI.

Follow [the release procedure](docs/RELEASING.md) for version changes, migrations, offline query caches and promotion. The [branching guide](docs/BRANCHING.md) describes the patch-active development cycle; inspect the manifests for the current state.

## Deployment environments

| Environment | Address and behavior |
|---|---|
| Astound endpoint | `https://sp-dev.astound.digital` — verify releases against this endpoint. |
| SystemPrompt test environment | `https://astound.systemprompt.io` — the local Fly profile is named `production`. |

`just deploy` targets the Fly test environment, not the Astound endpoint. With Core patches active, use `just deploy-next` to refresh vendored source first; it refuses to run when patches are disabled. A registry push alone does not verify deployment to the Astound endpoint.
