#!/usr/bin/env bash
# services/ai/gateway.yaml decides which upstream answers a request, so a
# route's `when:` block must only key on facts that distinguish requests.
# Two rules:
#
#   * `when.tool_names_any` never names a client tool. Agent hosts (Claude
#     Code, Desktop) declare their whole tool catalogue on every turn, so a
#     catalogue name selects every turn the host ever sends. Client tool
#     names are CamelCase (`WebSearch`, `WebFetch`, `ToolSearch`); any entry
#     with an uppercase letter, or one of those three names, fails.
#   * a route that exists to reach a provider's native tool service matches
#     server tools by their provider-defined type, `when.tool_types_any`.
#
# Why: a client tool name in a `when:` block parses happily and nothing
# else reads the predicate, so a misrouted fleet goes unnoticed until the bill.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FILE="$REPO_ROOT/services/ai/gateway.yaml"
[ -f "$FILE" ] || { echo "check-gateway-routes: $FILE is missing" >&2; exit 1; }

python3 - "$FILE" <<'PY'
import sys, yaml

path = sys.argv[1]
with open(path) as fh:
    doc = yaml.safe_load(fh)
routes = ((doc or {}).get("gateway") or {}).get("routes") or []
client_names = {"WebSearch", "WebFetch", "ToolSearch"}
errors = []
for index, route in enumerate(routes):
    label = route.get("id") or f"{route.get('model_pattern')} -> {route.get('provider')}"
    when = route.get("when") or {}
    for name in when.get("tool_names_any") or []:
        if name in client_names or any(ch.isupper() for ch in str(name)):
            errors.append(f"route {label} (#{index}): tool_names_any names client tool `{name}`")
    for kind in when.get("tool_types_any") or []:
        if not str(kind) or any(ch.isspace() for ch in str(kind)):
            errors.append(f"route {label} (#{index}): tool_types_any entry `{kind}` is not a type prefix")
for err in errors:
    print(f"check-gateway-routes: {err}", file=sys.stderr)
sys.exit(1 if errors else 0)
PY
