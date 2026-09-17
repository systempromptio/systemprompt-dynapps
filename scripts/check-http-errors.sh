#!/usr/bin/env bash
# Reject inline `map_err(|e| ApiError::ctor(...))` at HTTP call sites.
#
# HTTP status mapping belongs in an entry-local error type's `From` impls
# (see each crate error.rs / error/ module);
# call sites propagate with bare `?` so the variant decides the status. An inline
# closure flattens distinct failure modes into one status (a dropped DB
# connection becomes a 404) and discards the typed cause.
#
# There is no marker-based exemption: a deliberate re-classification is a
# `match` onto a typed variant, and a foreign variant-less error is carried
# as a `#[source]` on a variant of the crate's own error type.
set -uo pipefail

SEARCH_DIR="extensions"
PATTERN='map_err\(\s*\|[^|]*\|\s*ApiError::'

if ! command -v rg >/dev/null 2>&1; then
    echo "check-http-errors: ripgrep (rg) is required" >&2
    exit 2
fi

RAW=$(rg -Un --multiline-dotall --no-heading --color=never \
    -g '*.rs' \
    "$PATTERN" "$SEARCH_DIR" 2>/dev/null || true)

HITS=$(printf '%s\n' "$RAW" | grep -v '^[[:space:]]*$' || true)

if [ -n "$HITS" ]; then
    echo "check-http-errors: inline map_err into ApiError at an HTTP call site."
    echo "Map domain -> HTTP via an entry-local error type's From impls and propagate"
    echo "with bare ?:"
    echo "$HITS"
    exit 1
fi

# The same defect in any layer: `map_err(|e| SomeError::Variant(format!(...)))`
# stringifies the typed cause and flattens distinct failure modes into one
# variant. Use a typed variant with #[from]/#[source] and bare `?`; a
# genuinely variant-less foreign error is carried as the `source` of such a
# variant (`AdminError::upstream`, `AdminError::invalid`,
# `MarketplaceError::crypto`), never stringified.
# A `map_err(` closure counts only when it stringifies its error binding into
# a `format!` within the next three lines. A closure that keeps the typed
# cause (a struct variant with a `source` field) and uses format! purely for
# context strings is the sanctioned pattern and is not flagged.
FLATTENED=$(
    while IFS= read -r f; do
        awk -v fname="$f" '
            { line[FNR] = $0; n = FNR }
            END {
                for (i = 1; i <= n; i++) {
                    if (!match(line[i], /map_err\(\|[a-z_][a-z0-9_]*[:|]/)) continue
                    bind = substr(line[i], RSTART + 9, RLENGTH - 10)
                    hi = i + 3; if (hi > n) hi = n
                    hit = 0; kept = 0
                    bre = "[^A-Za-z0-9_]" bind "[^A-Za-z0-9_]"
                    for (j = i; j <= hi; j++) {
                        stripped = line[j]
                        if (j == i) sub(/.*map_err\(\|[a-z_][a-z0-9_]*[:|]*/, "", stripped)
                        if (stripped ~ ("format!\\(.*\\)[[:space:]]*,[[:space:]]*&?" bind "[^A-Za-z0-9_]")) kept = 1
                        else if (stripped ~ /format!/ && (" " stripped " ") ~ bre) hit = 1
                        else if ((" " stripped " ") ~ bre) kept = 1
                    }
                    if (hit && !kept) printf "%s:%d:%s\n", fname, i, line[i]
                }
            }' "$f"
    done < <(git ls-files -co --exclude-standard \
        'extensions/**/*.rs' 'src/**/*.rs' 'bridge/src/**/*.rs' \
        | grep -v '/tests/' | sort -u)
)

if [ -n "$FLATTENED" ]; then
    echo "check-http-errors: map_err flattening a typed cause into format!()."
    echo "Add a typed error variant carrying the source (#[from]/#[error(...)])"
    echo "and propagate with bare ?:"
    echo "$FLATTENED"
    exit 1
fi

# The same defect, one level up: a route handler that returns a bare `Response`
# has no error channel at all, so it must build every failure by hand — which
# is how status codes and bodies drifted apart in the first place. Handlers
# return `Result<Response, E>` where `E: IntoResponse` decides the status
# (`AdminResult` for JSON, `AdminHtmlResult` for pages, or a flow's own typed
# channel such as an SSO redirect); axum's `impl IntoResponse for Result<T, E>`
# means the route table does not care.
#
# A route handler is an `async fn`; a synchronous `-> Response` is a renderer
# that never fails (a CSV body, a redirect, an `IntoResponse` impl) and has no
# error to type. Only the async form is checked, so a renderer needs no marker.
HANDLER_DIR="extensions/web/admin/src/handlers"

UNTYPED=$(
    while IFS= read -r f; do
        awk -v fname="$f" '
            /(^|[^a-z_])fn [A-Za-z_0-9]+[<(]/ { in_async = ($0 ~ /async fn /) }
            /->[[:space:]]*(impl IntoResponse|Response)[[:space:]]*[{]/ {
                if (in_async) printf "%s:%d:%s\n", fname, FNR, $0
            }' "$f"
    done < <(find "$HANDLER_DIR" -name '*.rs')
)

if [ -n "$UNTYPED" ]; then
    echo "check-http-errors: route handler returns an untyped response."
    echo "Return AdminResult<Response>, AdminHtmlResult<Response> for a page, or"
    echo "Result<Response, E> with an E: IntoResponse that is the flow's own failure"
    echo "channel, so the error type picks the status:"
    echo "$UNTYPED"
    exit 1
fi

# Statuses come from an `AdminError` variant, never a literal. These two are
# the ones that were being hand-written most often, and the ones where getting
# it wrong is least visible: a 500 that should have been a 404, or a 403 built
# beside the check instead of by it.
LITERALS=$(grep -rnE 'StatusCode::(INTERNAL_SERVER_ERROR|FORBIDDEN)' "$HANDLER_DIR" --include='*.rs' || true)

if [ -n "$LITERALS" ]; then
    echo "check-http-errors: raw status literal in a handler."
    echo "Use AdminError::Forbidden / AdminError::internal so the classification and the"
    echo "logging happen in one place:"
    echo "$LITERALS"
    exit 1
fi
