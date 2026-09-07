#!/usr/bin/env bash
# Actions-only publication of an explicitly requested v0.47.0 packaging repair.
# The caller has assessed the DMG on macOS and staged a signed candidate image.
# Markdown backticks in printf formats are intentionally literal.
# shellcheck disable=SC2016
set -euo pipefail
: "${EXPECTED_SHA:?}" "${BASE_DIGEST:?}" "${CANDIDATE_DIGEST:?}" "${IMAGE:?}" "${GITHUB_REPOSITORY:?}"
[[ "$EXPECTED_SHA" =~ ^[0-9a-f]{64}$ ]]
[[ "$BASE_DIGEST" =~ ^sha256:[0-9a-f]{64}$ ]]
[[ "$CANDIDATE_DIGEST" =~ ^sha256:[0-9a-f]{64}$ ]]
TAG=v0.47.0
BACKUP="repair-backup-v0.47.0-${EXPECTED_SHA:0:12}"
identity="https://github.com/$GITHUB_REPOSITORY/"
verify_blob() {
    cosign verify-blob --certificate-identity-regexp="$identity" \
        --certificate-oidc-issuer=https://token.actions.githubusercontent.com \
        --signature "$1.sig" --certificate "$1.pem" "$1"
}

mkdir original repaired
gh release download "$TAG" -D original
gh release view "$TAG" --json body,targetCommitish,url,assets > original/release-metadata.json
verify_blob original/SHA256SUMS
(cd original && sha256sum -c SHA256SUMS)
current=$(docker buildx imagetools inspect "$IMAGE:0.47.0" --format '{{json .Manifest.Digest}}' | tr -d '"')
[ "$current" = "$BASE_DIGEST" ] || { echo '0.47.0 moved since repair preparation; refusing to overwrite' >&2; exit 1; }
latest=$(docker buildx imagetools inspect "$IMAGE:latest" --format '{{json .Manifest.Digest}}' | tr -d '"')
printf '0.47.0=%s\nlatest=%s\n' "$current" "$latest" > original/image-digests.txt
# Preserve the old manifest behind a durable tag as well as recording its digest.
docker buildx imagetools create -t "$IMAGE:backup-0.47.0-${EXPECTED_SHA:0:12}" "$IMAGE@$BASE_DIGEST"
if gh release view "$BACKUP" >/dev/null 2>&1; then
    echo "Backup $BACKUP already exists; refusing to overwrite it" >&2
    exit 1
fi
gh release create "$BACKUP" --draft --target "$(jq -r .targetCommitish original/release-metadata.json)" \
    --title 'Original v0.47.0 before macOS repair' \
    --notes 'Rollback assets and original release metadata. Do not delete until repair acceptance.' original/*

cp original/SHA256SUMS repaired/
cp candidate/astound-bridge-macos.dmg candidate/astound-bridge-macos.dmg.sha256 repaired/
(cd repaired && sha256sum -c astound-bridge-macos.dmg.sha256)
awk -v sha="$EXPECTED_SHA" '$2 == "astound-bridge-macos.dmg" {$1=sha} {print $1 "  " $2}' \
    original/SHA256SUMS > repaired/SHA256SUMS
for name in astound-bridge-macos.dmg SHA256SUMS; do
    cosign sign-blob --yes --output-signature "repaired/$name.sig" \
        --output-certificate "repaired/$name.pem" "repaired/$name"
    verify_blob "repaired/$name"
done
jq -r '.body' original/release-metadata.json | \
    sed "/^\\*\\*macOS is not notarized\\*\\*/d; /astound-bridge-macos.dmg |/s/signed-cosign-only/notarized (app + DMG)/; s/$BASE_DIGEST/$CANDIDATE_DIGEST/g" > repaired/notes.md
{
    printf '\n### macOS packaging repair\n\n'
    printf 'Repaired %s from source `%s`, retaining core v0.47.0.\n\n' "$(date -u +%FT%TZ)" "${REPAIR_SOURCE_SHA:?}"
    printf 'DMG SHA-256: `%s`. Image digest: `%s`. Original release tag retained.\n\n' "$EXPECTED_SHA" "$CANDIDATE_DIGEST"
    printf 'Download through https://sp-dev.astound.digital/bridge-auth/setup on the Astound network. '
    printf 'Drag to Applications, eject the DMG and launch. Both app and DMG are signed, notarized and stapled. '
    printf 'For another gateway use `astound-bridge login --gateway <URL>`.\n'
} >> repaired/notes.md
gh release upload "$TAG" --clobber repaired/astound-bridge-macos.dmg* repaired/SHA256SUMS*
docker buildx imagetools create -t "$IMAGE:0.47.0" -t "$IMAGE:latest" "$IMAGE@$CANDIDATE_DIGEST"
gh release edit "$TAG" --notes-file repaired/notes.md
mkdir redownload
gh release download "$TAG" -D redownload
verify_blob redownload/SHA256SUMS
(cd redownload && sha256sum -c SHA256SUMS)
printf 'Repaired `%s`; rollback backup `%s`.\n' "$TAG" "$BACKUP" >> "$GITHUB_STEP_SUMMARY"
