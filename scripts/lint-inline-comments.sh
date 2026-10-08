#!/usr/bin/env bash
set -uo pipefail

# Machete rule: inline `//` comments are banned in production crates.
#
# The only permitted full-line inline comments are the two whitelisted
# justification prefixes mandated by the rust-coding-standards skill:
#
#   // Why:    — a non-obvious invariant, hidden constraint, or exemption
#                justification (e.g. a permitted `let _ =`)
#   // JSON:   — a sanctioned `serde_json::Value` protocol-boundary usage
#   // SAFETY: — the discharge of an `unsafe` block's obligations. Not a
#                discretionary comment: clippy's `undocumented_unsafe_blocks`
#                is warn-level and the workspace builds with `-D warnings`, so
#                every `unsafe` block must carry one and it must start with
#                `SAFETY:` for clippy to recognise it. Banning the prefix here
#                would leave the two gates mutually unsatisfiable.
#
# Continuation lines of a whitelisted comment block are allowed. `//!` module
# heads are governed separately (rustdoc placement rules), as are `///` docs
# on public API items. `tests/**` and `build.rs` files are out of scope.
#
# A second check enforces rustdoc placement: `///` is reserved for public
# traits, top-level types (struct/enum/union/type), and `mod` declarations.
# Rustdoc on anything else — functions, methods, consts, statics, fields,
# enum variants, macros, or any `pub(crate)`/`pub(super)`/private item — is
# banned. A genuine invariant on such an item belongs in a `// Why:` comment;
# anything else is deleted.
#
# Two more checks bound `// Why:` itself, so a justification cannot decay back
# into narration:
#
#   * at most 6 `// Why:` blocks per production file (each prefix line opens a
#     block; continuation lines do not count). A file that needs more is
#     explaining a design its names should carry.
#   * no `// Why:` inside a function body (indent >= 8) unless it is a marker
#     block (`// Why: discard-ok:` / `// Why: lint-ok:` / `// Why: fail-soft-ok:`) or its first line names
#     an external system on the WHY_EXTERNAL allowlist below. Anything else in a
#     body is extracted into a named `let`, helper fn, or enum instead.
#
# Exemption: a path (one per line, `#` comments) in
# scripts/rustdoc-exemptions.txt. Reserve it for files where `///` is
# load-bearing rather than decorative — clap-derive CLIs, where the rustdoc
# *is* the generated `--help` text and deleting it silently guts the CLI.
#
# Scope: production sources in `extensions/**`, `src/**` and `bridge/src/**`, tracked or
# not (`git ls-files -co`) — an untracked new file must not pass vacuously.

EXEMPT_FILE="scripts/rustdoc-exemptions.txt"
WHY_CAP=6
WHY_EXTERNAL='AD FS|ADFS|SAML|Handlebars|axum|Postgres|sqlx|SQLx|core|Claude Code|MCP|zip|serde|schemars|Gemini|Salesforce|Atlassian|OAuth|reqwest|cargo|rustc|clippy|toolchain|Windows|macOS|Linux|SEP-'
is_exempt() {
    [ -f "$EXEMPT_FILE" ] || return 1
    grep -v '^[[:space:]]*#' "$EXEMPT_FILE" | grep -qxF "$1"
}

MATCHES=""
while IFS= read -r file; do
    case "$file" in
        tests/*|*/tests/*) continue ;;
        */build.rs) continue ;;
    esac
    is_exempt "$file" && continue
    FOUND=$(awk -v why_cap="$WHY_CAP" -v why_ext="$WHY_EXTERNAL" '
        BEGIN { why_ext_re = "(^|[^A-Za-z])(" why_ext ")" }
        /^[[:space:]]*\/\/ Why:/ {
            why_count++
            match($0, /^[[:space:]]*/)
            if (RLENGTH >= 8) {
                body = $0
                sub(/^[[:space:]]*\/\/ Why:[[:space:]]*/, "", body)
                if (body !~ /^(discard-ok|lint-ok|fail-soft-ok):/ && body !~ why_ext_re) {
                    print FILENAME ":" FNR ": // Why: inside a function body — extract a named let/fn/enum, or lead with the external system that forces it"
                }
            }
        }
        /^[[:space:]]*\/\/\// { prev_allowed = 0; if (!in_doc) doc_line = FNR; in_doc = 1; next }
        /^[[:space:]]*\/\/!/ { prev_allowed = 0; next }
        /^[[:space:]]*\/\// {
            in_doc = 0
            if ($0 ~ /^[[:space:]]*\/\/ (Why|JSON|SAFETY):/) { prev_allowed = 1; next }
            if (prev_allowed) { next }
            print FILENAME ":" FNR ":" $0
            next
        }
        /^[[:space:]]*#!?\[/ {
            open = gsub(/\[/, "["); close_n = gsub(/\]/, "]")
            if (open > close_n) { in_attr = 1; attr_depth = open - close_n }
            next
        }
        {
            if (in_attr) {
                attr_depth += gsub(/\[/, "[") - gsub(/\]/, "]")
                if (attr_depth <= 0) { in_attr = 0 }
                next
            }
            if ($0 ~ /^[[:space:]]*$/) { next }
            if (in_doc) {
                stripped = $0
                sub(/^[[:space:]]+/, "", stripped)
                if (stripped !~ /^(pub[[:space:]]+)?(unsafe[[:space:]]+)?(trait|struct|enum|union|type|mod)[[:space:]<]/) {
                    print FILENAME ":" doc_line ": rustdoc on non-type item (" stripped ") — /// is reserved for pub traits, top-level types, and modules; use // Why: or delete"
                } else if (stripped !~ /^pub[[:space:]]/) {
                    print FILENAME ":" doc_line ": rustdoc on non-public item (" stripped ") — use // Why: or delete"
                }
            }
            in_doc = 0
            prev_allowed = 0
        }
        END {
            if (why_count > why_cap) {
                print FILENAME ":1: " why_count " // Why: blocks (cap " why_cap ") — let names carry the design, keep only external constraints"
            }
        }
    ' "$file")
    [ -n "$FOUND" ] && MATCHES+="${FOUND}"$'\n'
done < <(git ls-files -co --exclude-standard 'extensions/*.rs' 'extensions/**/*.rs' 'src/*.rs' 'src/**/*.rs' 'bridge/src/*.rs' 'bridge/src/**/*.rs' | sort -u)

# `///` rustdoc is banned in test code (core rule, previously only logged by
# the observational audit): rustdoc is never rendered for test crates, so a
# doc there is a paraphrase by definition. Scaffolding `//` comments stay
# legal; `//!` heads are optional and legal.
while IFS= read -r file; do
    FOUND=$(grep -n '^[[:space:]]*///' "$file" \
        | sed "s|^|${file}:|;s|\$| — /// banned in test code, use //|" || true)
    [ -n "$FOUND" ] && MATCHES+="${FOUND}"$'\n'
done < <(git ls-files -co --exclude-standard 'tests/**/*.rs' ':(glob)**/tests/**/*.rs' | sort -u)

if [ -z "$MATCHES" ]; then
    echo "lint-inline-comments: OK (no unlisted inline comments)"
    exit 0
fi

echo "lint-inline-comments: inline // comments are banned in production crates,"
echo "and /// rustdoc is reserved for pub traits, top-level types, and modules."
echo "Delete the comment, or justify it with a '// Why:', '// JSON:' or '// SAFETY:' prefix;"
echo "// Why: is capped at ${WHY_CAP} per file and barred from function bodies unless external:"
echo ""
printf '%s' "$MATCHES"
exit 1
