# Install from the published image

Every push to `main` publishes a prebuilt, multi-arch (amd64 + arm64) image to
`ghcr.io/systempromptio/systemprompt-dynapps`. Installing means pulling it —
no Rust toolchain, no compile.

## 1. Access

The package is private, like the repository. You need a GitHub account with
read access to `systempromptio/systemprompt-dynapps` and a **personal access
token (classic)** with the `read:packages` scope
(GitHub → Settings → Developer settings → Personal access tokens).

```bash
echo "$GITHUB_PAT" | docker login ghcr.io -u <github-username> --password-stdin
```

If the pull is denied although the repo is visible to you, a maintainer must
grant your team read access on the package once: repository → Packages →
`systemprompt-dynapps` → Package settings → Manage access.

## 2. Configure

```bash
git clone https://github.com/systempromptio/systemprompt-dynapps.git   # for docker-compose.yml + .env.example
cd systemprompt-dynapps
cp .env.example .env
```

Edit `.env`:

| variable | required | notes |
|---|---|---|
| `ANTHROPIC_API_KEY` / `OPENAI_API_KEY` / `GEMINI_API_KEY` | one of them | the first set becomes the default provider |
| `SYSTEMPROMPT_ADMIN_EMAIL` | yes, first boot | the administrator the container's first-boot `admin setup` creates; the entrypoint refuses to start without it |
| `EXTERNAL_URL` | on a server | the public URL, e.g. `https://gateway.example.com` — sets `api_external_url` and CORS |
| `SYSTEMPROMPT_TAG` | no | `latest` follows `main`; pin a release with its version, e.g. `0.72.1` |
| `POSTGRES_PASSWORD`, `HTTP_PORT`, `PG_PORT` | no | bundled Postgres and port mapping |
| `DATABASE_URL` | remote DB only | see below |
| `DATABASE_WRITE_URL` | read/write split only | primary for writes when `DATABASE_URL` is a read replica; core refuses a standby as write target |

## 3. Run

```bash
docker compose pull && docker compose up -d      # or: just compose-up
curl -fsS http://localhost:8080/api/v1/health
```

First boot writes the docker profile, waits for Postgres, runs migrations and
bootstraps an admin. Open `http://localhost:8080/admin` to finish setup.

### Remote Postgres (Oracle VM and similar)

Set `DATABASE_URL=postgres://user:pass@host:5432/systemprompt` in `.env` and
start only the app: `docker compose up -d app`. The database needs the
`uuid-ossp` and `pgcrypto` extensions; pgvector is not required.

### More than one node

The `.env` path above is **single-node only**. On first boot it authors a
profile inside the container and generates that container's own signing key,
OAuth pepper, and manifest seed; a second node would generate different ones
and reject every token the first node minted. For two or more nodes (and for
any deployment where secrets come from a vault rather than a `.env` file) use
the rendered-profile path in `deploy/self-host/README.md`: one profile
directory, identical on every node, bind-mounted with `SYSTEMPROMPT_PROFILE_DIR`.
Oracle Cloud specifics: `oracle-cloud.md`.

### Upgrade, pin, roll back

```bash
docker compose pull && docker compose up -d                  # follow latest
SYSTEMPROMPT_TAG=0.72.1 docker compose up -d                 # pin (or set it in .env)
SYSTEMPROMPT_TAG=<previous-version> docker compose up -d     # roll back — tags are never rewritten
```

**Upgrading an existing install across 0.44.0:** the provider catalog and <!-- pinned-release -->
gateway routes moved out of the profile and into the image
(`services/ai/providers.yaml` / `services/ai/gateway.yaml`), and boot now
refuses to start if the profile still has a top-level `providers:` (or
`gateway:`) section. A fresh container is unaffected — first boot writes a
clean profile — but if you're persisting `.systemprompt/profiles/` across
upgrades (the volume/bind mount in your `docker-compose.yml`), migration
fails at "Running database migrations..." with `Profile initialization
failed ... still carries a top-level providers: section`. Fix it before
restarting: open the mounted `profile.yaml`, delete the entire `providers:`
block (and `gateway:` if present), and restart — there's nothing to move
those settings to, the catalog now ships baked into the image.

### Changing configuration between releases

`services/` is baked into the image and read once at boot, so a running node
serves what it was built with. To change it without waiting for the next image,
bind-mount your own copy over `/app/services` and restart — the container
already sets `SYSTEMPROMPT_SERVICES_PATH=/app/services`, so the mount is all
that is needed. Note that admin-UI edits to gateway routes write inside the
container and are lost when it is replaced unless that tree is mounted, and
that a `:ro` mount makes the UI editor fail on write. The served
[Deploy](https://dynapps.systemprompt.io/documentation/deploy) page covers both
in full.

Versioned tags (`X.Y.Z`) are immutable; `latest` and `sha-<commit>` move with
`main`. No `sha-<commit>` tag was published for 0.48.0 — that release has only <!-- pinned-release -->
`0.48.0` and `latest`, for the reason in *Verify the image* below. Core <!-- pinned-release -->
migrations are forward-only: roll the image back only to a version whose
migrations match the database.

### Verify the image

> **The 0.48.0 image is not signed.** GitHub Actions was unavailable on this <!-- pinned-release -->
> account when it shipped, so `release.yml` never ran and the image was built
> and pushed by hand. `cosign verify` checks for a certificate issued by
> Actions' OIDC provider, so it **fails against 0.48.0** — that failure means <!-- pinned-release -->
> the signature is absent, not that the image is untrusted. Verify that one
> release by digest instead:
> `sha256:cd4e2a50edb5f63a4f6b1426aa4fe7d41788d300d5158b5670b475b548a69437`.
> Later releases built by `release.yml` are signed and verify with the command
> below.

```bash
cosign verify \
  --certificate-identity-regexp='https://github.com/systempromptio/systemprompt-dynapps/' \
  --certificate-oidc-issuer=https://token.actions.githubusercontent.com \
  ghcr.io/systempromptio/systemprompt-dynapps:0.72.1
```

## 4. The bridge (Claude Code client)

The gateway serves its own bridge binaries at `<gateway>/files/downloads/`
(they are baked into the image), and the admin **Bridge Setup** page links
them. The same files are attached to the GitHub Release `vX.Y.Z`:

| file | platform |
|---|---|
| `dynapps-bridge-linux-x86_64.tar.gz`, `dynapps-bridge-linux-aarch64.tar.gz` | Linux (glibc ≥ 2.35) |
| `dynapps-bridge-macos.dmg` | macOS, universal (Apple Silicon + Intel) |
| `dynapps-bridge-windows.exe` | Windows x86_64 |
| `install.sh` | Linux one-liner installer |

```bash
curl -fsSL <gateway>/files/downloads/install.sh | sh -s -- \
  --download-base <gateway>/files/downloads --code <connect-code>
```

The macOS app and DMG are Developer ID signed, notarized and stapled, so
Gatekeeper opens them without a prompt (`scripts/verify-bridge-macos.sh <dmg>`
checks the team, notarization tickets and Gatekeeper verdict on a downloaded
copy). Until Dynapps provides a Windows code-signing certificate the Windows
exe is **not OS-signed**: SmartScreen asks for *More info → Run anyway*. Every
asset is checksummed (`SHA256SUMS`) and Sigstore-signed (`.sig`/`.pem`)
regardless — see the release notes for the `cosign verify-blob` line.

### Pointing a bridge at a different gateway (local instance, staging)

The bridge verifies every synced manifest against an ed25519 public key. Each
instance signs with its own key (derived from the profile's
`manifest_signing_secret_seed`, generated at `admin setup`), so a local
`just start` instance and the staging gateway never share one. The bridge
learns the key on its first sync (trust-on-first-use) and stores it in
`[sync] pinned_pubkey` **together with the gateway it was learned from**. A
pin for one gateway is ignored for another, and the next sync re-learns it.
`dynapps-bridge doctor` reports which gateway the pin belongs to.

The key can also be supplied out of band, and that source always wins:

| source | where | precedence |
|---|---|---|
| env var | `DYNAPPS_BRIDGE_POLICY_PUBKEY` | highest |
| managed policy | Windows `HKLM\SOFTWARE\Policies\dynapps` / `HKCU\…`, value `manifestPubkey`; macOS `io.systemprompt.dynapps` plist | second |
| config file | `[sync] pinned_pubkey` + `pinned_pubkey_gateway` in `dynapps-bridge.toml` | lowest |

A sync that fails with *"does not match the pubkey pinned from the policy"*
means one of the first two rows holds a key for a different gateway. Read the
current gateway's key and repin:

```bash
curl -s http://localhost:8080/v1/bridge/pubkey
dynapps-bridge install --apply --pubkey <base64 from the response>
```

### Windows: which registry hive the policy lands in

The Claude Desktop / Cowork policy (`SOFTWARE\Policies\Claude`) is written to
**HKCU** when the bridge runs as an ordinary user, and to **HKLM** when it runs
elevated (or via `install --apply`, which prompts once). Cowork honours HKCU
only while no HKLM key exists, so the bridge refuses an HKCU write that a
conflicting HKLM key would shadow and says so in the sync result. Every write
is read back; a value that did not land is reported, never assumed.
`dynapps-bridge doctor` shows which hive holds the policy.

## Building from source instead

```bash
docker compose -f docker-compose.yml -f docker-compose.build.yml up --build
```

This is the old path and needs the full toolchain inside the builder stage
(~25 min cold). It exists for Dockerfile work and air-gapped hosts, not for
installs.
