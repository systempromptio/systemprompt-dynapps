#!/usr/bin/env bash
# Deploy a published release to production from a clean worktree of
# origin/main. Invoked by `just deploy-release <version>`.
#
# Why a worktree: `just deploy` ships whatever the working tree holds. In
# 0.52.0 a peer had re-activated [patch.crates-io] on unreleased core in the
# shared checkout; in 0.53.0 the local `main` branch was stale and a peer held
# 44 modified files. The release is a commit on origin/main and nothing else.
#
# Why the release assets: the same `cargo build --release --workspace` that
# built the image built the server tarball, and each bridge binary can only be
# built on its own platform. Building here would ship a different binary from
# the one the release signed. DOWNLOADS_STRICT=1 refuses the host fallback and
# verifies every staged download against the release's SHA256SUMS.
#
# Why the five-minute watch: 0.53.0's machine answered healthy at +1 min and
# 503 from +2 to +5 min while the shared database stalled under the new
# instance's first scheduler runs. One probe is not a deploy verification.
set -euo pipefail

VERSION="${1:?usage: deploy-release.sh <version>}"
case "$VERSION" in v*) VERSION="${VERSION#v}" ;; esac
TAG="v$VERSION"

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORKTREE="$(dirname "$ROOT")/$(basename "$ROOT")-deploy-$VERSION"
LOCAL_PROFILE=".systemprompt/profiles/local/profile.yaml"
PROD_PROFILE=".systemprompt/profiles/production/profile.yaml"
# DEPLOY_DRY_RUN=1 stops after the preflight, before `cloud deploy`.
WATCH_SECONDS="${DEPLOY_WATCH_SECONDS:-300}"
WATCH_INTERVAL=15
DIGEST_WAIT_SECONDS="${DEPLOY_DIGEST_WAIT_SECONDS:-120}"
DIGEST_WAIT_INTERVAL=5

# A desktop/credential docker shim fails `docker build` with an error naming
# neither; the real binary is /usr/bin/docker.
if [ -x /usr/bin/docker ]; then export PATH=/usr/bin:$PATH; fi

log() { printf '\n==> %s\n' "$*"; }
die() { echo "deploy-release: $*" >&2; exit 1; }

for tool in git gh flyctl jq curl docker; do
    command -v "$tool" >/dev/null 2>&1 || die "$tool is required"
done
gh auth status >/dev/null 2>&1 || die "gh is not authenticated (the repository is private)"
[ -f "$ROOT/$PROD_PROFILE" ] || die "no production profile at $ROOT/$PROD_PROFILE"
[ -f "$ROOT/signing_key.pem" ] || die "no signing_key.pem in $ROOT"

tenant_id="$(sed -n 's/^  tenant_id: *//p' "$ROOT/$PROD_PROFILE" | head -1)"
[ -n "$tenant_id" ] || die "production profile has no tenant_id"
FLY_APP="sp-$(printf '%s' "$tenant_id" | tr -d '-' | cut -c1-12)"
HOST="$(sed -n 's/^  api_external_url: *//p' "$ROOT/$PROD_PROFILE" | head -1)"
[ -n "$HOST" ] || die "production profile has no api_external_url"

log "release $TAG must be origin/main"
git -C "$ROOT" fetch -q origin main "refs/tags/$TAG:refs/tags/$TAG" 2>/dev/null \
    || git -C "$ROOT" fetch -q origin main
main_sha="$(git -C "$ROOT" rev-parse origin/main)"
tag_sha="$(git -C "$ROOT" rev-parse "$TAG^{commit}" 2>/dev/null || true)"
[ -n "$tag_sha" ] || die "tag $TAG does not exist — release.yml has not published $VERSION yet"
[ "$tag_sha" = "$main_sha" ] || die "tag $TAG is $tag_sha but origin/main is $main_sha — deploy the release, not a later main"
gh release view "$TAG" --json assets --jq '.assets|length' >/dev/null \
    || die "no GitHub release $TAG"
docker manifest inspect "ghcr.io/astound-digital/systemprompt-astound:$VERSION" >/dev/null 2>&1 \
    || die "GHCR has no :$VERSION image — release.yml did not finish"
echo "    origin/main = $TAG = ${main_sha:0:12}"

log "worktree $WORKTREE"
[ ! -e "$WORKTREE" ] || die "$WORKTREE exists — a previous deploy left it for inspection; remove it first"
git -C "$ROOT" worktree add --detach "$WORKTREE" "$main_sha" >/dev/null
cd "$WORKTREE"
[ -z "$(git status --porcelain)" ] || die "fresh worktree is not clean"
grep -q '^\[patch\.crates-io\]' Cargo.toml tests/Cargo.toml && die "[patch.crates-io] is active on main"
cp -r "$ROOT/.systemprompt" .
cp "$ROOT/signing_key.pem" .
# The local profile carries absolute paths; publish_pipeline renders web/dist
# at `paths.system`, which must be THIS tree or the image COPYs the old one.
sed -i -E \
    -e "s#^(  system:) .*#\\1 $WORKTREE#" \
    -e "s#^(  services:) .*#\\1 $WORKTREE/services#" \
    -e "s#^(  bin:) .*#\\1 $WORKTREE/target/release#" \
    -e "s#^(  storage:) .*#\\1 $WORKTREE/storage#" \
    "$LOCAL_PROFILE"
grep -q "^  system: $WORKTREE\$" "$LOCAL_PROFILE" || die "could not re-point the local profile at the worktree"
grep -q "^  bin: $WORKTREE/target/release\$" "$LOCAL_PROFILE" || die "could not re-point paths.bin at the release binaries"

log "binaries from release $TAG"
just fetch-release "$VERSION"

log "downloads from release $TAG (strict)"
DOWNLOADS_STRICT=1 DOWNLOADS_RELEASE_TAG="$TAG" scripts/fetch-remote-downloads.sh
# The worktree runs the fetcher AS OF THE RELEASE COMMIT, which may predate
# strict mode; verify the staged bytes here regardless of which fetcher ran.
sums="$(mktemp)"
gh release download "$TAG" --pattern SHA256SUMS --output "$sums" --clobber
for f in storage/files/downloads/*; do
    name="$(basename "$f")"
    case "$name" in *.sha256) continue ;; esac
    want="$(grep " $name\$" "$sums" | cut -d' ' -f1)"
    [ -n "$want" ] || die "staged $name is not in the $TAG SHA256SUMS — it did not come from the release"
    got="$(sha256sum "$f" | cut -d' ' -f1)"
    [ "$got" = "$want" ] || die "staged $name is ${got:0:12}, release has ${want:0:12} — a stale or host-fallback copy"
    echo "    staged $name matches $TAG"
done

log "web/dist"
SYSTEMPROMPT_PROFILE="$WORKTREE/$LOCAL_PROFILE" target/release/systemprompt infra jobs run publish_pipeline --profile local
[ -f web/dist/index.html ] || die "publish_pipeline did not render web/dist here"

log "preflight"
target/release/systemprompt cloud doctor --profile production --distributed

machine_state() {
    flyctl machines list -a "$FLY_APP" --json | jq -r '.[0] | "\(.image_ref.digest) \(.updated_at)"'
}
before_state="$(machine_state)"
before="${before_state%% *}"
echo "    $FLY_APP image before: $before (updated ${before_state#* })"

if [ "${DEPLOY_DRY_RUN:-0}" = "1" ]; then
    cd "$ROOT"
    git worktree remove --force "$WORKTREE"
    log "DEPLOY_DRY_RUN=1: everything up to 'cloud deploy' passed for $TAG; nothing was deployed"
    exit 0
fi

log "cloud deploy --profile production"
target/release/systemprompt cloud deploy --profile production

# Why: `cloud deploy` returns when Fly accepts the release; the machine is
# swapped asynchronously. 0.54.0 read the digest immediately, saw the old one,
# died with "did not move" and skipped its own health watch while the machine
# rolled seconds later. Poll until the digest or updated_at moves.
log "waiting for $FLY_APP to roll (up to ${DIGEST_WAIT_SECONDS}s)"
roll_deadline=$((SECONDS + DIGEST_WAIT_SECONDS))
after_state="$(machine_state)"
while [ "$after_state" = "$before_state" ] && [ "$SECONDS" -lt "$roll_deadline" ]; do
    sleep "$DIGEST_WAIT_INTERVAL"
    after_state="$(machine_state)"
done
after="${after_state%% *}"
echo "    $FLY_APP image after:  $after (updated ${after_state#* })"
[ "$after_state" != "$before_state" ] \
    || die "the Fly machine did not roll within ${DIGEST_WAIT_SECONDS}s of 'cloud deploy' returning — nothing was deployed"

log "watching $HOST/health for ${WATCH_SECONDS}s (every ${WATCH_INTERVAL}s)"
deadline=$((SECONDS + WATCH_SECONDS))
saw_version=0
failures=0
while [ "$SECONDS" -lt "$deadline" ]; do
    body="$(curl -sS -m 10 -w '\n%{http_code}' "$HOST/health" 2>/dev/null || echo -e '\n000')"
    code="${body##*$'\n'}"
    payload="${body%$'\n'*}"
    stamp="$(date -u +%H:%M:%S)"
    if [ "$code" = "200" ] && printf '%s' "$payload" | grep -q "\"version\":\"$VERSION\""; then
        saw_version=1
        echo "    $stamp 200 $payload"
    else
        failures=$((failures + 1))
        echo "    $stamp $code ${payload:-<no body>}"
        if [ "$saw_version" = 1 ]; then
            echo "deploy-release: $HOST went unhealthy AFTER reporting $VERSION — read 'flyctl logs -a $FLY_APP' for 'pool timed out' / 'slow statement' (database stall) versus an application fault." >&2
        fi
    fi
    sleep "$WATCH_INTERVAL"
done
[ "$saw_version" = 1 ] || die "$HOST never reported version $VERSION within ${WATCH_SECONDS}s"
[ "$failures" -eq 0 ] || die "$HOST was unhealthy on $failures of the probes after the deploy — not a clean deploy; worktree kept at $WORKTREE"

log "served downloads match the release"
for f in storage/files/downloads/*; do
    name="$(basename "$f")"
    case "$name" in *.sha256) continue ;; esac
    want="$(grep " $name\$" "$sums" | cut -d' ' -f1)"
    [ -n "$want" ] || continue
    got="$(curl -fsSL "$HOST/files/downloads/$name" | sha256sum | cut -d' ' -f1)"
    [ "$got" = "$want" ] || die "$HOST/files/downloads/$name serves ${got:0:12}, release has ${want:0:12}"
    echo "    $name OK"
done
rm -f "$sums"

cd "$ROOT"
git worktree remove --force "$WORKTREE"

log "deployed $TAG to $FLY_APP"
echo "    main/tag: ${main_sha:0:12}"
echo "    image:    $before -> $after"
echo "    health:   $VERSION steady for ${WATCH_SECONDS}s"
echo "Record these in the release notes; a version string alone is not deploy evidence."
