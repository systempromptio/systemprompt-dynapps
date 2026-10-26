#!/usr/bin/env bash
# Claude Code is installed into two images (the server image for the agent farm
# and deploy/clean-client for the bridge smoke tests). One `ARG
# CLAUDE_CODE_VERSION=<x.y.z>` per Dockerfile pins it, and every Dockerfile
# must agree on the default so the farm and the smoke tests exercise the same
# client. Three rules over the tracked Dockerfiles:
#
#   * no `@anthropic-ai/claude-code@latest` (an unpinned npm install);
#   * no `install.sh | bash` without `-s "${CLAUDE_CODE_VERSION}"` (an
#     unpinned native install);
#   * every `ARG CLAUDE_CODE_VERSION=` default is the same string.
#
# Dockerfiles that do not install Claude Code are not required to carry the ARG.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

mapfile -t DOCKERFILES < <(git ls-files 'Dockerfile' '*/Dockerfile' '*Dockerfile*' | grep -vE '\.(sh|md|py)$' | sort -u)
[ "${#DOCKERFILES[@]}" -gt 0 ] || { echo "check-claude-code-version: no tracked Dockerfiles" >&2; exit 1; }

fail=0
declare -A defaults
for file in "${DOCKERFILES[@]}"; do
    if grep -nE 'claude-code@latest' "$file" | grep -vE '^[0-9]+:\s*#' >/dev/null; then
        echo "check-claude-code-version: $file installs @anthropic-ai/claude-code@latest; use @\${CLAUDE_CODE_VERSION}" >&2
        fail=1
    fi
    while IFS= read -r hit; do
        [ -n "$hit" ] || continue
        case "$hit" in
            *'-s "${CLAUDE_CODE_VERSION}"'*) ;;
            *)
                echo "check-claude-code-version: $file:${hit%%:*} pipes install.sh to bash without -s \"\${CLAUDE_CODE_VERSION}\"" >&2
                fail=1 ;;
        esac
    done < <(grep -nE 'claude\.ai/install\.sh[^|]*\|[[:space:]]*bash' "$file" | grep -vE '^[0-9]+:\s*#' || true)
    while IFS= read -r line; do
        [ -n "$line" ] || continue
        value="${line#*=}"
        [ -n "$value" ] || { echo "check-claude-code-version: $file declares ARG CLAUDE_CODE_VERSION without a default" >&2; fail=1; continue; }
        defaults["$file"]="$value"
    done < <(grep -E '^ARG[[:space:]]+CLAUDE_CODE_VERSION(=|$)' "$file" || true)
done

[ "${#defaults[@]}" -gt 0 ] || { echo "check-claude-code-version: no Dockerfile declares ARG CLAUDE_CODE_VERSION" >&2; exit 1; }
distinct="$(printf '%s\n' "${defaults[@]}" | sort -u)"
if [ "$(printf '%s\n' "$distinct" | wc -l)" -ne 1 ]; then
    echo "check-claude-code-version: ARG CLAUDE_CODE_VERSION defaults disagree across Dockerfiles:" >&2
    for file in "${!defaults[@]}"; do echo "  $file: ${defaults[$file]}" >&2; done
    fail=1
fi

[ "$fail" -eq 0 ] || exit 1
echo "check-claude-code-version: Claude Code pinned at $distinct in ${#defaults[@]} Dockerfile(s)"
