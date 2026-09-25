#!/usr/bin/env bash
# Sync astound's own release version. The core crate pins and
# bridge/CORE_REF are a separate number, owned by scripts/sync-core-version.sh:
# astound can release without a core release.
#
#   scripts/sync-release-version.sh 0.21.0          # apply
#   scripts/sync-release-version.sh 0.21.0 --check  # verify only (CI guard)
#
# Covered pins:
#   Cargo.toml            workspace version
#   bridge/Cargo.toml     the bridge crate version (displayed + reported to the gateway)
#   deploy/self-host/profile.yaml.template   `# template-version:` on line 1
#   operator docs that pin the image by version
#
# macOS + Linux compatible (no GNU-only sed flags).
set -eu

VERSION="${1:?usage: sync-release-version.sh <version> [--check]}"
MODE="${2:-apply}"
cd "$(dirname "$0")/.."

case "$VERSION" in
  *[!0-9.]*|*..*|.*|*.) echo "ERROR: '$VERSION' is not a plain semver (X.Y.Z)"; exit 1 ;;
esac
IFS=. read -r MAJ MIN PATCH <<EOV
$VERSION
EOV
: "${PATCH:?ERROR: version must have three components}"

fail=0

# file, description, grep pattern that must match post-apply
check_or_apply() { # $1=file $2=sed-expr $3=expect-regex $4=label
    local file="$1" sedexpr="$2" expect="$3" label="$4"
    if [ "$MODE" = "--check" ]; then
        if ! grep -Eq "$expect" "$file"; then
            echo "DRIFT: $label in $file (expected /$expect/)"
            fail=1
        fi
    else
        sed -i.bak -e "$sedexpr" "$file" && rm -f "$file.bak"
        grep -Eq "$expect" "$file" || { echo "ERROR: failed to set $label in $file"; exit 1; }
    fi
}

# Cargo.toml — workspace version (first `version =` in [workspace.package]).
check_or_apply Cargo.toml \
    "s|^version = \"[0-9.]*\"|version = \"$VERSION\"|" \
    "^version = \"$VERSION\"" \
    "workspace version"

# bridge/Cargo.toml — the white-label bridge crate. Its version is what the
# desktop app displays and what the gateway heartbeat reports, so it tracks the
# astound release.
check_or_apply bridge/Cargo.toml \
    "s|^version = \"[0-9.]*\"|version = \"$VERSION\"|" \
    "^version = \"$VERSION\"" \
    "bridge crate version"

# The profile template is pinned to the image release on its first line.
# render-profile.sh --image and preflight.sh compare it with the image's
# org.opencontainers.image.version label and refuse a mismatch, so a profile
# rendered from one release cannot be handed to another release's loader —
# the schema is the image's, and "unknown field" is the only symptom otherwise.
TEMPLATE=deploy/self-host/profile.yaml.template
if [ -f "$TEMPLATE" ]; then
    check_or_apply "$TEMPLATE" \
        "1s|^# template-version: [0-9.]*\$|# template-version: $VERSION|" \
        "^# template-version: $VERSION\$" \
        "profile template version pin"
    head -1 "$TEMPLATE" | grep -Eq '^# template-version: ' || {
        echo "ERROR: $TEMPLATE must open with '# template-version: X.Y.Z' on line 1"
        [ "$MODE" = "--check" ] && fail=1 || exit 1
    }
fi

# Operator documentation that pins the image by version. These files are read
# on GitHub, so they carry the real number rather than a token; this script is
# the only thing allowed to change it, and --check refuses any other 0.x.y.
# Hosted documentation (services/content/) never carries a literal version —
# it uses @RELEASE_VERSION@, substituted at ingestion from the crate version
# (scripts/check-docs-version.sh enforces both halves).
# The compose files carry the tag a self-hoster actually pulls, in the `:?`
# error text and the usage comment. They are not swept as documentation
# elsewhere, so before this list covered them they kept naming the previous
# release and told operators to pin an image the release had moved past.
# A line ending `<!-- pinned-release -->` is exempt from both the sweep and the
# check: it states a fact about a specific past release, and moving the number
# forward would make it false. This mirrors scripts/check-docs-version.sh, which
# carries the same exemption — the two must agree or `just release` fails on a
# line `just lint-gates` has already passed.
# scripts/e2e-published-release.sh is deliberately NOT here: it compares
# against a literal 0.44.0 to detect pre-0.44 images, and sweeping that forward
# each release would silently disable the check.
for doc in deploy/self-host/README.md deploy/self-host/SINGLE-NODE.md \
           deploy/self-host/docker-compose.single.yml \
           deploy/self-host/docker-compose.node.yml \
           docs/install/docker.md docs/install/oracle-cloud.md; do
    [ -f "$doc" ] || continue
    if [ "$MODE" = "--check" ]; then
        other=$(grep -nE '(^|[^0-9.])0\.[0-9]+\.[0-9]+([^0-9.]|$)' "$doc" | grep -v "$VERSION" \
            | grep -v '<!-- pinned-release -->$' || true)
        if [ -n "$other" ]; then
            echo "DRIFT: $doc names a version other than $VERSION:"
            echo "$other"
            fail=1
        fi
    else
        sed -i.bak -E "/<!-- pinned-release -->$/!s/(^|[^0-9.])0\.[0-9]+\.[0-9]+([^0-9.]|$)/\1$VERSION\2/g" "$doc" && rm -f "$doc.bak"
    fi
done

if [ "$MODE" = "--check" ]; then
    [ "$fail" -eq 0 ] && echo "version sync OK: astound pinned to $VERSION" || exit 1
else
    echo "version sync applied: $VERSION"
fi
