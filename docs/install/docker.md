# Install from the published image

Every push to `main` publishes a prebuilt, multi-arch (amd64 + arm64) image to
`ghcr.io/astound-digital/systemprompt-astound`. Installing means pulling it —
no Rust toolchain, no compile.

## 1. Access

The package is private, like the repository. You need a GitHub account with
read access to `Astound-Digital/systemprompt-astound` and a **personal access
token (classic)** with the `read:packages` scope
(GitHub → Settings → Developer settings → Personal access tokens).

```bash
echo "$GITHUB_PAT" | docker login ghcr.io -u <github-username> --password-stdin
```

If the pull is denied although the repo is visible to you, a maintainer must
grant your team read access on the package once: repository → Packages →
`systemprompt-astound` → Package settings → Manage access.

## 2. Configure

```bash
git clone https://github.com/Astound-Digital/systemprompt-astound.git   # for docker-compose.yml + .env.example
cd systemprompt-astound
cp .env.example .env
```

Edit `.env`:

| variable | required | notes |
|---|---|---|
| `ANTHROPIC_API_KEY` / `OPENAI_API_KEY` / `GEMINI_API_KEY` | one of them | the first set becomes the default provider |
| `EXTERNAL_URL` | on a server | the public URL, e.g. `https://gateway.example.com` — sets `api_external_url` and CORS |
| `SYSTEMPROMPT_TAG` | no | `latest` follows `main`; pin a release with its version, e.g. `0.42.0` |
| `POSTGRES_PASSWORD`, `HTTP_PORT`, `PG_PORT` | no | bundled Postgres and port mapping |
| `DATABASE_URL` | remote DB only | see below |

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
`vector` extension (`CREATE EXTENSION IF NOT EXISTS vector;`) — the bundled
image is `pgvector/pgvector:pg18` for that reason.

### Upgrade, pin, roll back

```bash
docker compose pull && docker compose up -d                  # follow latest
SYSTEMPROMPT_TAG=0.42.0 docker compose up -d                 # pin (or set it in .env)
SYSTEMPROMPT_TAG=0.41.0 docker compose up -d                 # roll back — tags are never rewritten
```

Versioned tags (`X.Y.Z`) are immutable; `latest` and `sha-<commit>` move with
`main`. Core migrations are forward-only: roll the image back only to a
version whose migrations match the database.

### Verify the image

```bash
cosign verify \
  --certificate-identity-regexp='https://github.com/Astound-Digital/systemprompt-astound/' \
  --certificate-oidc-issuer=https://token.actions.githubusercontent.com \
  ghcr.io/astound-digital/systemprompt-astound:0.42.0
```

## 4. The bridge (Claude Code client)

The gateway serves its own bridge binaries at `<gateway>/files/downloads/`
(they are baked into the image), and the admin **Bridge Setup** page links
them. The same files are attached to the GitHub Release `vX.Y.Z`:

| file | platform |
|---|---|
| `astound-bridge-linux-x86_64.tar.gz`, `astound-bridge-linux-aarch64.tar.gz` | Linux (glibc ≥ 2.35) |
| `astound-bridge-macos.dmg` | macOS, universal (Apple Silicon + Intel) |
| `astound-bridge-windows.exe` | Windows x86_64 |
| `install.sh` | Linux one-liner installer |

```bash
curl -fsSL <gateway>/files/downloads/install.sh | sh -s -- \
  --download-base <gateway>/files/downloads --code <connect-code>
```

Until Astound provides an Apple Developer ID and a Windows code-signing
certificate, the macOS and Windows binaries are **not OS-signed**: macOS
Gatekeeper blocks the first open (right-click → Open, or
`xattr -d com.apple.quarantine "/Applications/AstoundBridge.app"`), and
Windows SmartScreen asks for *More info → Run anyway*. Every asset is
checksummed (`SHA256SUMS`) and Sigstore-signed (`.sig`/`.pem`) regardless — see
the release notes for the `cosign verify-blob` line.

## Building from source instead

```bash
docker compose -f docker-compose.yml -f docker-compose.build.yml up --build
```

This is the old path and needs the full toolchain inside the builder stage
(~25 min cold). It exists for Dockerfile work and air-gapped hosts, not for
installs.
