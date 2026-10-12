#!/usr/bin/env bash
# Backfill storage/files/downloads/ from the GitHub release, or the deployed
# host if that is unavailable.
#
# Why this exists: storage/files/downloads/ is gitignored, and `cloud deploy`
# bakes the whole storage/ tree into the image it ships. Each platform's bridge
# binary can only be BUILT on that platform's toolchain (the Windows exe via
# cargo-xwin on Linux, the DMG on a Mac), so any single machine is always
# missing the artifacts the others produced. Deploying from such a machine
# without this step silently replaces the live image with one that 404s the
# other platforms' download links.
#
# For every asset the site links, this script keeps whatever is already staged
# locally (a fresh local build wins) and downloads the rest, verifying the
# published sha256.
#
# The GitHub release is tried FIRST, and the deployed host only as a fallback.
# Sourcing from the host made the live site the source of truth for its own next
# version: a deploy republished whatever that host happened to be serving, so a
# bad artifact survived every subsequent deploy and the real provenance — the
# release that built it — was never consulted. The release is immutable, signed,
# and produced by release.yml on the platform that can actually build each
# binary.
#
# The repository is PRIVATE, so this needs an authenticated `gh`. Without one it
# falls back to the host and says so; it does not fail, because a developer
# without gh auth must still be able to deploy.
#
# A missing asset WARNS and continues. It used to be fatal, on the reasoning
# that deploying would 404 a live link — true, but wrong as a default: each
# platform's bridge can only be built on that platform's toolchain, so a Linux
# host can never produce the macOS DMG and every deploy from one was blocked on
# an artifact it is structurally incapable of building.
#
# DOWNLOADS_STRICT=1 restores the hard failure. Use it when cutting a release,
# where a 404'd download link is worth stopping for. FETCH_ALLOW_MISSING=1 is
# still accepted and now a no-op, since continuing is the default.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DOWNLOADS_DIR="$REPO_ROOT/storage/files/downloads"
BASE="${DOWNLOADS_REMOTE_BASE:-https://dynapps.systemprompt.io/files/downloads}"
BASE="${BASE%/}"
REPO="${DOWNLOADS_GH_REPO:-systempromptio/systemprompt-dynapps}"
# Which release to backfill from. `latest` resolves the newest published
# release; pin an explicit vX.Y.Z when rebuilding an older image.
RELEASE_TAG="${DOWNLOADS_RELEASE_TAG:-latest}"

# DOWNLOADS_STRICT=1 is the release mode: every asset comes from the named
# release and nothing else. No host fallback (the host serves the PREVIOUS
# release, so a transient gh failure would silently ship an old binary — it
# did on 0.53.0's first pass), no "local build wins" (the release is the
# build), and every staged file is checked against the release's signed
# SHA256SUMS, not just its own sidecar.
STRICT="${DOWNLOADS_STRICT:-0}"

GH_SOURCE=""
if command -v gh >/dev/null 2>&1 && gh auth status >/dev/null 2>&1; then
    if [ "$RELEASE_TAG" = "latest" ]; then
        GH_SOURCE="$(gh release view --repo "$REPO" --json tagName --jq .tagName 2>/dev/null || true)"
    else
        GH_SOURCE="$RELEASE_TAG"
    fi
fi
if [ "$STRICT" = "1" ]; then
    [ -n "$GH_SOURCE" ] || { echo "ERROR: DOWNLOADS_STRICT=1 needs an authenticated gh and a published release (DOWNLOADS_RELEASE_TAG=$RELEASE_TAG)." >&2; exit 1; }
    echo "downloads: STRICT — every asset from the $GH_SOURCE release on $REPO, verified against its SHA256SUMS; no host fallback"
elif [ -n "$GH_SOURCE" ]; then
    echo "downloads: sourcing from the $GH_SOURCE release on $REPO (host $BASE is the fallback)"
else
    echo "downloads: no authenticated gh, or no published release; sourcing from $BASE" >&2
fi

# Every asset a page links. Checksummed ones are listed bare — their .sha256
# sidecar is fetched and verified alongside. install.sh has no sidecar.
CHECKSUMMED=(
    dynapps-bridge-windows.exe
    dynapps-bridge-linux-x86_64.tar.gz
    dynapps-bridge-macos.dmg
)
PLAIN=(
    install.sh
)
# Not published yet, and no page links them: preserved when present, skipped
# without error when absent — so publishing one later doesn't require touching
# this script. Optional means DOWNLOADS_STRICT=1 does NOT stop a release over
# them, which is the point: a strict release must not be blocked on an artifact
# nothing links.
#
# dynapps-bridge-macos.dmg moved to CHECKSUMMED above on 2026-09-11: the DMG is
# built, it ships in the release, and the homepage and header both link it. This
# comment used to say it had never been built and the macOS links were commented
# out site-wide — both stopped being true without anyone moving it, so a strict
# release was not guarding a link the site actually serves.
OPTIONAL_CHECKSUMMED=(
    dynapps-bridge-linux-aarch64.tar.gz
)

if command -v sha256sum >/dev/null 2>&1; then
    SHA_CHECK=(sha256sum -c)
else
    SHA_CHECK=(shasum -a 256 -c)
fi

mkdir -p "$DOWNLOADS_DIR"

if [ "$STRICT" = "1" ]; then
    for asset in "${CHECKSUMMED[@]}" "${OPTIONAL_CHECKSUMMED[@]}" "${PLAIN[@]}"; do
        rm -f "$DOWNLOADS_DIR/$asset" "$DOWNLOADS_DIR/$asset.sha256"
    done
fi
missing=()

FETCHED_FROM=""

fetch() { # fetch <name> -> 0 fetched, 1 not found at either source
    local name="$1" tmp
    tmp="$(mktemp -d)"
    FETCHED_FROM=""
    # The release first: immutable, signed, and built by the platform that can
    # produce that binary.
    # Why: GitHub's asset download 5xxs often enough that 0.53.0's first pass
    # lost one of five files to it; one retry covers the blip, and strict
    # mode then fails instead of falling through to the host.
    if [ -n "$GH_SOURCE" ] &&
        { gh release download "$GH_SOURCE" --repo "$REPO" --pattern "$name" \
              --dir "$tmp" --clobber >/dev/null 2>&1 ||
          { sleep 3 && gh release download "$GH_SOURCE" --repo "$REPO" --pattern "$name" \
              --dir "$tmp" --clobber >/dev/null 2>&1; }; } &&
        [ -f "$tmp/$name" ]; then
        mv "$tmp/$name" "$DOWNLOADS_DIR/$name"
        chmod 0644 "$DOWNLOADS_DIR/$name"
        rm -rf "$tmp"
        FETCHED_FROM="the $GH_SOURCE release"
        return 0
    fi
    if [ "$STRICT" != "1" ] && curl -fsSL -o "$tmp/$name" "$BASE/$name" 2>/dev/null; then
        mv "$tmp/$name" "$DOWNLOADS_DIR/$name"
        chmod 0644 "$DOWNLOADS_DIR/$name"
        rm -rf "$tmp"
        FETCHED_FROM="$BASE"
        return 0
    fi
    rm -rf "$tmp"
    return 1
}

for asset in "${CHECKSUMMED[@]}"; do
    if [ -f "$DOWNLOADS_DIR/$asset" ]; then
        # Local build wins; make sure its sidecar exists for the next machine.
        if [ ! -f "$DOWNLOADS_DIR/$asset.sha256" ]; then
            (cd "$DOWNLOADS_DIR" && { sha256sum "$asset" 2>/dev/null \
                || shasum -a 256 "$asset"; } > "$asset.sha256")
        fi
        echo "==> $asset: already staged locally, keeping it"
        continue
    fi
    if fetch "$asset" && fetch "$asset.sha256"; then
        (cd "$DOWNLOADS_DIR" && "${SHA_CHECK[@]}" "$asset.sha256" >/dev/null) || {
            echo "ERROR: checksum mismatch for fetched $asset — refusing to stage it." >&2
            rm -f "$DOWNLOADS_DIR/$asset" "$DOWNLOADS_DIR/$asset.sha256"
            exit 1
        }
        echo "==> $asset: fetched from $FETCHED_FROM and verified"
    else
        rm -f "$DOWNLOADS_DIR/$asset" "$DOWNLOADS_DIR/$asset.sha256"
        missing+=("$asset")
    fi
done

for asset in "${OPTIONAL_CHECKSUMMED[@]}"; do
    if [ -f "$DOWNLOADS_DIR/$asset" ]; then
        echo "==> $asset: already staged locally, keeping it"
    elif fetch "$asset" && fetch "$asset.sha256"; then
        (cd "$DOWNLOADS_DIR" && "${SHA_CHECK[@]}" "$asset.sha256" >/dev/null) || {
            echo "ERROR: checksum mismatch for fetched $asset — refusing to stage it." >&2
            rm -f "$DOWNLOADS_DIR/$asset" "$DOWNLOADS_DIR/$asset.sha256"
            exit 1
        }
        echo "==> $asset: fetched from $FETCHED_FROM and verified"
    else
        rm -f "$DOWNLOADS_DIR/$asset" "$DOWNLOADS_DIR/$asset.sha256"
        echo "==> $asset: not published anywhere yet, skipping (optional)"
    fi
done

for asset in "${PLAIN[@]}"; do
    if [ -f "$DOWNLOADS_DIR/$asset" ]; then
        echo "==> $asset: already staged locally, keeping it"
    elif fetch "$asset"; then
        echo "==> $asset: fetched from $FETCHED_FROM"
    else
        missing+=("$asset")
    fi
done

if [ "${#missing[@]}" -gt 0 ]; then
    echo >&2
    echo "WARNING: missing (not staged locally, not on $BASE):" >&2
    for m in "${missing[@]}"; do echo "    $m" >&2; done
    echo "Download links for these will 404 on the deployed site." >&2
    echo "Build them on a host with the right toolchain (just" >&2
    echo "bridge-package-macos / bridge-package-windows / bridge-package-linux)." >&2
    if [ "${DOWNLOADS_STRICT:-0}" = "1" ]; then
        echo "DOWNLOADS_STRICT=1 set — refusing to continue." >&2
        exit 1
    fi
    echo "Continuing anyway (set DOWNLOADS_STRICT=1 to make this fatal)." >&2
fi

if [ "$STRICT" = "1" ]; then
    sums="$(mktemp)"
    gh release download "$GH_SOURCE" --repo "$REPO" --pattern SHA256SUMS --output "$sums" --clobber \
        || { echo "ERROR: release $GH_SOURCE has no SHA256SUMS." >&2; exit 1; }
    staged=()
    for asset in "${CHECKSUMMED[@]}" "${OPTIONAL_CHECKSUMMED[@]}" "${PLAIN[@]}"; do
        [ -f "$DOWNLOADS_DIR/$asset" ] && staged+=("$asset")
    done
    for asset in "${CHECKSUMMED[@]}" "${PLAIN[@]}"; do
        [ -f "$DOWNLOADS_DIR/$asset" ] || { echo "ERROR: $asset is linked by the site and missing from release $GH_SOURCE." >&2; exit 1; }
    done
    for asset in "${OPTIONAL_CHECKSUMMED[@]}"; do
        [ -f "$DOWNLOADS_DIR/$asset" ] && continue
        grep -q " $asset\$" "$sums" || continue
        echo "ERROR: $asset is published in $GH_SOURCE (listed in SHA256SUMS) but was not fetched — the image would serve a stale copy." >&2
        exit 1
    done
    pattern="$(printf ' %s$\n' "${staged[@]}" | paste -sd'|')"
    expected="$(grep -E "$pattern" "$sums")"
    [ "$(printf '%s\n' "$expected" | grep -c .)" -eq "${#staged[@]}" ] \
        || { echo "ERROR: SHA256SUMS of $GH_SOURCE does not list every staged asset:" >&2; printf '    %s\n' "${staged[@]}" >&2; exit 1; }
    (cd "$DOWNLOADS_DIR" && printf '%s\n' "$expected" | "${SHA_CHECK[@]}" -) \
        || { echo "ERROR: a staged asset does not match the $GH_SOURCE SHA256SUMS — refusing to deploy it." >&2; exit 1; }
    rm -f "$sums"
    echo "==> strict: ${#staged[@]} assets match the $GH_SOURCE SHA256SUMS"
fi

echo "==> downloads complete: $(ls "$DOWNLOADS_DIR" | tr '\n' ' ')"
