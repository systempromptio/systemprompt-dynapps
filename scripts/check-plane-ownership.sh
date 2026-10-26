#!/usr/bin/env bash
# Check that no sync plane writes a column another plane owns.
#
# `services/access-control/rules.yaml` is the one source of entitlement (see
# CLAUDE.md, "Every entitlement is declared in ..."). Its plane is the only
# code that may set an entitlement column: `access_control_entities`'s
# `default_included` and every column of `access_control_rules`. Other planes
# may own *rows* in those tables, never the access on them.
#
# The class this blocks: the gateway-route catalog plane re-registered every
# route on boot with `ON CONFLICT DO UPDATE SET default_included = false`, so a
# route rules.yaml declared `default: open` was open on a fresh database and
# closed after the next restart — whichever plane ran last won, and every PAT
# /v1/messages call was denied with `gateway_route:... not assigned`. A plane
# that only inserts unseen rows (`ON CONFLICT DO NOTHING`) owns existence and
# is allowed; rewriting an existing row's access is not.
#
# Flagged, in any SQL string literal under the repository layer:
#   - `UPDATE access_control_entities ... SET ... default_included`
#   - `INSERT INTO access_control_entities ... ON CONFLICT ... DO UPDATE SET
#     ... default_included`
#   - any `INSERT INTO` / `UPDATE` of `access_control_rules`
# outside the allowlist below. The allowlist is by file path, never by comment
# marker, so an exemption is a reviewed edit to this script.
#
#   repositories/access_rules/   the access-control plane itself (apply_sync*)
#   governance/.../gateway_acl.rs  the console's sanctioned per-route edit path
#   entitlement/rules.rs           the console's sanctioned grant editor
#                                  (console:<user> rows, never boot)
#
# REPO_DIR (space-separated, globs allowed) overrides the scanned directories.
set -uo pipefail

REPO_DIRS=()
for d in ${REPO_DIR:-extensions/web/shared/admin-core/src/repositories extensions/web/admin/*/src/repositories}; do
    [ -d "$d" ] && REPO_DIRS+=("$d")
done

[ "${#REPO_DIRS[@]}" -gt 0 ] || { echo "check-plane-ownership: no repository directory - nothing to check"; exit 0; }

python3 - "${REPO_DIRS[@]}" <<'PY'
import re, sys, pathlib

ALLOW = (
    "extensions/web/shared/admin-core/src/repositories/access_rules/",
    "extensions/web/admin/governance/src/repositories/config/gateway_acl.rs",
    "extensions/web/shared/admin-core/src/repositories/entitlement/rules.rs",
)
LITERAL = re.compile(r'r(#*)"(.*?)"\1|"((?:[^"\\]|\\.)*)"', re.S)
ENT = re.compile(r'\b(?:INSERT\s+INTO|UPDATE)\s+access_control_entities\b', re.I)
RULES = re.compile(r'\b(?:INSERT\s+INTO|UPDATE)\s+access_control_rules\b', re.I)
ENT_UPDATE_SET = re.compile(r'\bUPDATE\s+access_control_entities\b.*\bSET\b.*\bdefault_included\b', re.I | re.S)
ENT_CONFLICT_SET = re.compile(r'\bDO\s+UPDATE\s+SET\b.*\bdefault_included\b', re.I | re.S)

violations = []
for root in (pathlib.Path(a) for a in sys.argv[1:]):
    for path in sorted(root.rglob("*.rs")):
        rel = path.as_posix()
        if any(rel.startswith(a) or rel == a for a in ALLOW):
            continue
        src = path.read_text(encoding="utf-8")
        for m in LITERAL.finditer(src):
            sql = m.group(2) if m.group(2) is not None else m.group(3)
            line = src.count("\n", 0, m.start()) + 1
            if RULES.search(sql):
                violations.append(f"{rel}:{line}: writes access_control_rules outside the access-control plane")
            elif ENT.search(sql) and (ENT_UPDATE_SET.search(sql) or ENT_CONFLICT_SET.search(sql)):
                violations.append(f"{rel}:{line}: rewrites access_control_entities.default_included outside the access-control plane")

if violations:
    print("check-plane-ownership: a sync plane writes an entitlement column it does not own")
    for v in violations:
        print(f"  {v}")
    print("  Insert unseen rows with ON CONFLICT DO NOTHING, or move the write into repositories/access_rules/.")
    sys.exit(1)
print("check-plane-ownership: OK")
PY
