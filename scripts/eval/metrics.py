"""Deterministic plugin eval metrics.

Inputs are the gateway export (one ConversationBundle per skill run, fetched
with a PAT) and the local Claude Code transcript of the same session (the only
place full tool results live). Every metric is a count, a sum or a fixed rule
over those files: the same exports always produce the same numbers, and no
model is asked for an opinion.

    metrics.py <evals/plugin dir> <label> [--against <label>]

Writes <label>/metrics.json, metrics.csv and summary.md; with --against, the
deltas go into summary.md and metrics.json["delta"].
"""
import csv
import json
import pathlib
import re
import sys

# Error classes, matched against tool RESULT bodies only (never inputs).
ERROR_CLASSES = {
    "sf_invalid_field": r"INVALID_FIELD|No such column",
    "sf_invalid_type": r"INVALID_TYPE|sObject type '[^']+' is not supported",
    "sf_malformed": r"MALFORMED_QUERY|ValidationError",
    "sf_access": r'errorCode\W+INSUFFICIENT_ACCESS',
    "sf_5xx": r'"status_code"\W*5\d\d|UNKNOWN_EXCEPTION',
    "google_iam": r"mcp\.googleapis\.com/tools\.call' denied|Permission '[^']*' denied",
    "google_bad_args": r"Invalid JSON payload|Invalid value at 'query'|does not match \[a-z",
    "timeout": r"timed out",
    "tool_missing": r"No such tool available|tool not found",
}
RECORD_QUERY = re.compile(
    r"FROM\s+(Account|Opportunity|Contact|Lead|Task|Event|Case)\b"
    r"|uiapi\W+query\W+\{?\W*(Account|Opportunity|Contact|Lead|Task|Event|Case)\b",
    re.I,
)
# A query pinned to one record by id is scoped by construction.
ID_PINNED = re.compile(r"\b(Id|AccountId|WhatId|WhoId|OpportunityId)\W+(=|eq|IN)\W", re.I)
WRITE_TOOL = re.compile(r"create|update|delete|upsert|send|insert", re.I)

METRICS = [
    "invoked", "completed", "success", "turns", "requests", "input_tokens",
    "output_tokens", "cache_read_tokens", "cache_creation_tokens", "cost_usd",
    "duration_s", "p95_latency_ms", "mcp_calls", "sf_calls", "google_calls",
    "builtin_calls", "failed_calls", "unscoped_queries", "writes",
    "answer_chars", "mock_mentions", "attempts",
] + list(ERROR_CLASSES)


def bundle_of(raw, skill):
    f = raw / "bundles" / f"{skill}.jsonl"
    if not f.exists() or not f.stat().st_size:
        return None
    lines = [json.loads(l) for l in open(f, encoding="utf-8") if l.strip()]
    # Why: a session can hold a side context; the skill run is the one with turns.
    return max(lines, key=lambda b: (b.get("facts") or {}).get("turn_count") or 0)


def cc_events(raw, skill):
    f = raw / "cc" / f"{skill}.jsonl"
    uses, results = [], []
    if not f.exists():
        return uses, results
    for line in open(f, encoding="utf-8"):
        content = (json.loads(line).get("message") or {}).get("content")
        if not isinstance(content, list):
            continue
        for c in content:
            if c.get("type") == "tool_use":
                uses.append(c)
            elif c.get("type") == "tool_result":
                results.append(c)
    return uses, results


def result_text(r):
    c = r.get("content")
    if isinstance(c, list):
        return " ".join(x.get("text", "") for x in c if isinstance(x, dict))
    return str(c or "")


def skill_metrics(raw, skill, manifest):
    run = manifest["runs"].get(skill, {})
    res_file = raw / "results" / f"{skill}.json"
    result = json.load(open(res_file, encoding="utf-8-sig")) if res_file.exists() else {}
    b = bundle_of(raw, skill)
    facts = (b or {}).get("facts") or {}
    uses, results = cc_events(raw, skill)
    dashed = f"{manifest['plugin']}:{skill.replace('_', '-')}"

    m = {k: 0 for k in METRICS}
    m["skill"] = skill
    m["session_id"] = run.get("session_id")
    m["context_id"] = (b or {}).get("context_id")
    m["attempts"] = run.get("attempts", 1)
    m["invoked"] = int(any(s.get("skill") == dashed for s in (b or {}).get("skills") or []))
    m["marketplace_hash"] = next(
        (s.get("marketplace_hash") for s in (b or {}).get("skills") or [] if s.get("skill") == dashed), None)
    for k_out, k_in in [("turns", "turn_count"), ("requests", "request_count"),
                        ("input_tokens", "input_tokens"), ("output_tokens", "output_tokens"),
                        ("cache_read_tokens", "cache_read_tokens"),
                        ("cache_creation_tokens", "cache_creation_tokens"),
                        ("duration_s", "duration_seconds"), ("p95_latency_ms", "p95_latency_ms")]:
        m[k_out] = facts.get(k_in) or 0
    m["cost_usd"] = round((facts.get("cost_microdollars") or 0) / 1e6, 4)

    for u in uses:
        name, inp = u.get("name", ""), json.dumps(u.get("input"))
        if not name.startswith("mcp__"):
            m["builtin_calls"] += 1
            continue
        m["mcp_calls"] += 1
        tool = name.split("__")[-1]
        if "salesforce" in name:
            m["sf_calls"] += 1
            if tool == "dispatch_readonly" and RECORD_QUERY.search(inp) \
                    and "MCP TEST" not in inp.upper() and not ID_PINNED.search(inp):
                m["unscoped_queries"] += 1
            if tool != "dispatch_readonly" and tool != "display_widget" and WRITE_TOOL.search(tool):
                m["writes"] += 1
            if re.search(r'"method": "(POST|PATCH|PUT|DELETE)"', inp):
                m["writes"] += 1
        elif "google" in name:
            m["google_calls"] += 1
    for r in results:
        text = result_text(r)
        hit = False
        for cls, pat in ERROR_CLASSES.items():
            if re.search(pat, text):
                m[cls] += 1
                hit = True
        if hit or r.get("is_error"):
            m["failed_calls"] += 1

    answer = result.get("result") or ""
    m["answer_chars"] = len(answer)
    m["mock_mentions"] = len(re.findall(r"\bMOCK\b", answer))
    m["completed"] = int(bool(result) and not result.get("is_error") and len(answer) >= 200)
    m["success"] = int(m["invoked"] and m["completed"] and m["sf_calls"] > 0
                       and m["unscoped_queries"] == 0 and m["writes"] == 0
                       and m["sf_invalid_field"] + m["sf_invalid_type"] + m["sf_malformed"] == 0)
    return m


def rollup(rows):
    n = len(rows) or 1
    tot = {k: sum(r[k] for r in rows) for k in METRICS}
    tot["skills"] = len(rows)
    tot["success_rate"] = round(tot["success"] / n, 3)
    tot["completed_rate"] = round(tot["completed"] / n, 3)
    tot["cost_usd"] = round(tot["cost_usd"], 4)
    for k in ["input_tokens", "output_tokens", "cache_read_tokens", "cost_usd", "turns", "mcp_calls",
              "failed_calls", "duration_s"]:
        tot[f"avg_{k}"] = round(tot[k] / n, 3)
    return tot


def delta(cur, base):
    out = {}
    for k, v in cur.items():
        if isinstance(v, (int, float)) and isinstance(base.get(k), (int, float)):
            out[k] = round(v - base[k], 4)
    return out


def main():
    evals, label = pathlib.Path(sys.argv[1]), sys.argv[2]
    against = sys.argv[sys.argv.index("--against") + 1] if "--against" in sys.argv else None
    run_dir = evals / label
    manifest = json.load(open(run_dir / "manifest.json"))
    raw = run_dir / "raw"
    rows = [skill_metrics(raw, s, manifest) for s in manifest["skills"]]
    out = {
        "label": label,
        "plugin": manifest["plugin"],
        "plugin_version": manifest.get("plugin_version"),
        "prompts_hash": manifest["prompts_hash"],
        "model": manifest["model"],
        "started_at": manifest["started_at"],
        "marketplace_hashes": sorted({r["marketplace_hash"] for r in rows if r["marketplace_hash"]}),
        "skill_hashes": manifest["skills"],
        "metric_names": METRICS,
        "plugin_totals": rollup(rows),
        "skills": rows,
    }
    lines = [f"# {manifest['plugin']} eval `{label}`", "",
             f"plugin {out['plugin_version']} · marketplace {', '.join(h[:12] for h in out['marketplace_hashes'])}"
             f" · prompts {out['prompts_hash']} · {out['model']} · {out['started_at']}", ""]
    if against:
        base = json.load(open(evals / against / "metrics.json"))
        if base["prompts_hash"] != out["prompts_hash"]:
            lines += [f"> prompts differ from `{against}` ({base['prompts_hash']}): deltas compare different questions.", ""]
        by = {r["skill"]: r for r in base["skills"]}
        out["delta"] = {"against": against,
                        "plugin_totals": delta(out["plugin_totals"], base["plugin_totals"]),
                        "skills": {r["skill"]: delta(r, by[r["skill"]]) for r in rows if r["skill"] in by}}
    t = out["plugin_totals"]
    d = (out.get("delta") or {}).get("plugin_totals", {})
    lines += ["| metric | value | Δ |", "|---|---|---|"]
    for k in ["skills", "success_rate", "completed_rate", "cost_usd", "avg_cost_usd", "avg_input_tokens",
              "avg_output_tokens", "avg_cache_read_tokens", "avg_turns", "avg_mcp_calls", "failed_calls",
              "unscoped_queries", "writes", *ERROR_CLASSES]:
        lines.append(f"| {k} | {t.get(k)} | {d.get(k, '')} |")
    lines += ["", "| skill | success | turns | cost $ | in tok | out tok | mcp | failed | unscoped |",
              "|---|---|---|---|---|---|---|---|---|"]
    for r in rows:
        lines.append(f"| {r['skill']} | {r['success']} | {r['turns']} | {r['cost_usd']} | {r['input_tokens']} "
                     f"| {r['output_tokens']} | {r['mcp_calls']} | {r['failed_calls']} | {r['unscoped_queries']} |")
    json.dump(out, open(run_dir / "metrics.json", "w"), indent=2)
    with open(run_dir / "metrics.csv", "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=["skill", "session_id", "context_id", "marketplace_hash", *METRICS])
        w.writeheader()
        w.writerows(rows)
    (run_dir / "summary.md").write_text("\n".join(lines) + "\n")
    print("\n".join(lines))


if __name__ == "__main__":
    main()
