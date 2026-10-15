#!/bin/bash
# Step 90 — metrics for the run in evals/night-audit/<date>/ (gitignored):
# metrics.json and report.md with duration, turns, tokens, cost, MCP read and
# write calls per server, governance warnings, artifacts, DynBrain delivery,
# the eval score and pass/fail per recorded assertion. OUT_DIR overrides.
set -uo pipefail
source "$(cd "$(dirname "$0")" && pwd)/_night_common.sh"
header "Night audit 90 — report"
load_run_id
run_scope_sql
win="created_at BETWEEN $RUN_FROM_SQL AND $RUN_TO_SQL AND user_id = $RUN_USER_SQL"
out="${OUT_DIR:-$PROJECT_DIR/evals/night-audit/$(date -u +%Y-%m-%d)}"
mkdir -p "$out"
bt=$(bare_tool_sql tool_name)

sql_rows "SELECT $(server_sql) AS server, $bt AS tool, COUNT(*) AS n,
    COUNT(*) FILTER (WHERE decision = 'deny') AS errors
  FROM governance_decisions WHERE $win GROUP BY 1, 2 ORDER BY 1, 3 DESC" > "$STATE_DIR/mcp_calls.json"
sql_rows "SELECT decision, policy, COUNT(*) AS n FROM governance_decisions WHERE $win GROUP BY 1, 2 ORDER BY 3 DESC" > "$STATE_DIR/governance.json"
sql_rows "SELECT COUNT(*) AS requests, COALESCE(SUM(input_tokens), 0) AS input_tokens,
    COALESCE(SUM(output_tokens), 0) AS output_tokens, COALESCE(SUM(cache_read_tokens), 0) AS cache_read_tokens,
    COALESCE(SUM(cost_microdollars), 0) AS cost_microdollars FROM ai_requests WHERE $win" > "$STATE_DIR/gateway.json"
sql_rows "SELECT fr.status, fr.triggered_via, fr.started_at, fr.finished_at, ar.duration_ms, ar.turns,
    ar.input_tokens, ar.output_tokens, ar.cost_microdollars
  FROM agent_farm_runs fr LEFT JOIN agent_runs ar ON ar.id = fr.agent_run_id WHERE fr.id = $(sql_q "$RUN_ID")" > "$STATE_DIR/run_row.json"

python3 -I - "$STATE_DIR" "$out" "$RUN_ID" "$ODOO_SERVER" "$BRAIN_SERVER" "$ODOO_READ_TOOLS" "$BRAIN_WRITE_RE" "$BASE_URL" <<'PY'
import json, os, re, sys
state, out, run_id, odoo, brain, odoo_reads, brain_write_re, base = sys.argv[1:9]

def load(name, default):
    try:
        with open(os.path.join(state, name)) as fh:
            return json.load(fh)
    except Exception:
        return default

def num(v):
    try:
        return float(v)
    except (TypeError, ValueError):
        return 0.0

row = (load("run_row.json", []) or [{}])[0]
report = load("report.json", {})
gateway = (load("gateway.json", []) or [{}])[0]
followup = load("followup.json", None)
assertions = []
try:
    with open(os.path.join(state, "assertions.jsonl")) as fh:
        assertions = [json.loads(l) for l in fh if l.strip()]
except OSError:
    pass

odoo_read_set = set(odoo_reads.split())
servers = {}
for c in load("mcp_calls.json", []):
    s = servers.setdefault(c["server"], {"read": 0, "write": 0, "errors": 0, "tools": {}})
    n = int(num(c["n"]))
    is_write = (c["tool"] not in odoo_read_set) if c["server"] == odoo else \
        bool(re.match(brain_write_re, c["tool"])) if c["server"] == brain else False
    s["write" if is_write else "read"] += n
    s["errors"] += int(num(c["errors"]))
    s["tools"][c["tool"]] = n

gov = load("governance.json", [])
warns = sum(int(num(g["n"])) for g in gov if str(g.get("decision", "")).lower() == "warn")
outputs = [o.get("path") for o in report.get("outputs", []) or []]
delivered = any(a["name"].startswith("dynbrain docs_create") and a["pass"] for a in assertions)
article = open(os.path.join(state, "dynbrain_article_id")).read().strip() \
    if os.path.exists(os.path.join(state, "dynbrain_article_id")) else None

metrics = {
    "run_id": run_id,
    "target": base,
    "status": row.get("status"),
    "trigger": row.get("triggered_via"),
    "started_at": row.get("started_at"),
    "finished_at": row.get("finished_at"),
    "duration_s": round(num(row.get("duration_ms")) / 1000, 1) if row.get("duration_ms") else None,
    "turns": row.get("turns"),
    "tokens": {"input": int(num(row.get("input_tokens") or gateway.get("input_tokens"))),
               "output": int(num(row.get("output_tokens") or gateway.get("output_tokens"))),
               "cache_read": int(num(gateway.get("cache_read_tokens")))},
    "cost_usd": round(num(row.get("cost_microdollars") or gateway.get("cost_microdollars")) / 1e6, 4),
    "gateway_requests": int(num(gateway.get("requests"))),
    "mcp": servers,
    "governance": {"warn_count": warns, "decisions": gov},
    "artifacts": outputs,
    "delivery": {"dynbrain": report.get("delivery_status") or ("delivered" if delivered else "not_found"),
                 "article_id": article},
    "farm_report": {k: report.get(k) for k in ("mcp_calls", "mcp_reads", "mcp_writes", "governance_warnings")
                    if k in report},
    "eval": {"passed": report.get("eval_passed"), "score": report.get("eval_score")},
    "followup": followup,
    "assertions": assertions,
    "passed": bool(assertions) and all(a["pass"] for a in assertions),
}
with open(os.path.join(out, "metrics.json"), "w") as fh:
    json.dump(metrics, fh, indent=2, default=str)

t = metrics["tokens"]
lines = [
    f"# Night audit run {run_id}", "",
    f"Target {base} · trigger `{metrics['trigger']}` · status **{metrics['status']}** · "
    f"overall **{'PASS' if metrics['passed'] else 'FAIL'}**", "",
    "| Metric | Value |", "|---|---|",
    f"| Duration | {metrics['duration_s']} s |",
    f"| Turns | {metrics['turns']} |",
    f"| Tokens in / out / cache read | {t['input']} / {t['output']} / {t['cache_read']} |",
    f"| Cost | ${metrics['cost_usd']} |",
    f"| Gateway requests | {metrics['gateway_requests']} |",
    f"| Governance warnings | {warns} |",
    f"| DynBrain delivery | {metrics['delivery']['dynbrain']} (article {article or '-'}) |",
    f"| Eval | passed={metrics['eval']['passed']} score={metrics['eval']['score']} |", "",
    "## MCP calls", "", "| Server | Read | Write | Denied | Tools |", "|---|---|---|---|---|",
]
for name, s in sorted(servers.items()):
    tools = ", ".join(f"{k}×{v}" for k, v in sorted(s["tools"].items(), key=lambda kv: -kv[1]))
    lines.append(f"| {name} | {s['read']} | {s['write']} | {s['errors']} | {tools} |")
lines += ["", "## Artifacts", ""] + [f"- `{p}`" for p in outputs or ["(none)"]]
if followup:
    p = followup.get("proposal") or {}
    a = followup.get("applied")
    lines += ["", "## Follow-up", "",
              f"- Proposal: {json.dumps(p.get('fix'), ensure_ascii=False)}",
              f"- Proposal session {p.get('session_id')}, turns {p.get('num_turns')}, cost ${p.get('cost_usd')}",
              f"- Applied: {'yes, session ' + str(a.get('session_id')) if a else 'no (APPROVE_FIX not set)'}"]
lines += ["", "## Assertions", "", "| Step | Assertion | Result | Detail |", "|---|---|---|---|"]
for a in assertions:
    detail = str(a.get("detail", "")).replace("|", "\\|")
    lines.append(f"| {a['step']} | {a['name']} | {'PASS' if a['pass'] else 'FAIL'} | {detail} |")
with open(os.path.join(out, "report.md"), "w") as fh:
    fh.write("\n".join(lines) + "\n")
print(json.dumps({"passed": metrics["passed"], "out": out}))
PY
info "written: $out/metrics.json, $out/report.md"
