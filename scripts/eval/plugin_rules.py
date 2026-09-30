"""The console's plugin-evaluation rules, for the eval harness.

The console's Versions → Evaluation tab and the `analysis-plugin-eval`
export compute every run's figures with `plugin_eval.sql`
(extensions/web/admin/catalog/src/repositories/analysis/). `consistent_row` returns
those figures for one run: from the downloaded dataset when the gateway
serves it, otherwise from `score_bundle`, a line-for-line port of the SQL
over the exported conversation bundle. When both exist they are compared,
so a report can never quietly disagree with the console.
"""
import json
import re

# The rules of plugin_eval.sql. Change them there and here together.
SCHEMA = r"INVALID_FIELD|No such column|No such relation|INVALID_TYPE|is not supported|MALFORMED_QUERY|ValidationError"
ACCESS = r"INSUFFICIENT_ACCESS|Permission '[^']*' denied|PERMISSION_DENIED"
UPSTREAM = r'"status_code"\W*5\d\d|UNKNOWN_EXCEPTION'
BAD_ARGS = r"Invalid JSON payload|Invalid value at '|does not match \[a-z"
TIMEOUT = r"(?i)timed out"
WIDGET_FAIL = r'"status_code"\W*[45]\d\d|isError'
WRITE_TOOL = r"(?i)(create|update|delete|upsert|send|insert)"
WRITE_METHOD = r'"method"\s*:\s*"(POST|PATCH|PUT|DELETE)"'
UNAVAILABLE = (r"(?i)(don't|do not|cannot|can't) (have|find|access|see)[^.]{0,80}(tool|connector)"
               r"|not (available|connected) in this session|no [a-z ]{0,30}tool (is )?available")
PLACEHOLDER = r"(?i)\bMOCK\b|placeholder|sample data"

FACT_FIELDS = ["turns", "requests", "input_tokens", "output_tokens", "cache_read_tokens",
               "cache_creation_tokens", "cost_usd", "duration_s"]
RULE_FIELDS = ["mcp_calls", "connector_calls", "builtin_calls", "failed_calls", "schema_errors",
               "access_errors", "upstream_errors", "bad_arguments", "timeouts", "widgets", "writes",
               "answer_chars", "repeated_calls", "placeholder_mentions", "tools_unavailable",
               "completed", "success"]


def score_bundle(b, plugin_id):
    """plugin_eval.sql over one exported ConversationBundle."""
    f = b.get("facts") or {}
    turns = {r["request_id"]: r["created_at"] for r in b.get("requests") or [] if r.get("kind") == "turn"}
    by_req = {}
    for m in b.get("messages") or []:
        if m["request_id"] in turns and m["role"] != "system":
            by_req.setdefault(m["request_id"], []).append(m)
    msgs, calls = [], {}
    for rid, ms in by_req.items():
        ms.sort(key=lambda m: m["sequence_number"])
        for i, m in enumerate(ms):
            msgs.append((turns[rid], m))
            if m["role"] == "assistant" and m["content"].startswith("[tool_use:"):
                result = ms[i + 1]["content"] if i + 1 < len(ms) else ""
                key = (m["sequence_number"], m["content"])
                # Why: each turn repeats the thread before it; keep one copy of
                # a call, the one whose result is longest (as the SQL does).
                if key not in calls or len(result) > len(calls[key]):
                    calls[key] = result
    inputs = {}
    for (_, content) in calls:
        inputs[content] = inputs.get(content, 0) + 1
    row = {k: 0 for k in RULE_FIELDS}
    for (_, content), result in calls.items():
        tool = re.match(r"^\[tool_use:([^ \]]+)", content).group(1)
        classes = {k: bool(re.search(p, result)) for k, p in
                   [("schema_errors", SCHEMA), ("access_errors", ACCESS), ("upstream_errors", UPSTREAM),
                    ("bad_arguments", BAD_ARGS), ("timeouts", TIMEOUT)]}
        if tool.startswith("mcp__"):
            row["mcp_calls"] += 1
            row["connector_calls"] += int(tool.startswith(f"mcp__plugin_{plugin_id}_"))
        else:
            row["builtin_calls"] += 1
        row["failed_calls"] += int(any(classes.values()))
        for k, hit in classes.items():
            row[k] += int(hit)
        row["widgets"] += int(tool.endswith("__display_widget") and not re.search(WIDGET_FAIL, result))
        row["writes"] += int(bool(re.search(WRITE_TOOL, tool) or re.search(WRITE_METHOD, content)))
    row["repeated_calls"] = sum(n - 1 for n in inputs.values() if n > 1)
    answers = sorted(((t, m["sequence_number"], m["content"]) for t, m in msgs
                      if m["role"] == "assistant" and not m["content"].startswith("[tool_use:")), reverse=True)
    answer = answers[0][2] if answers else ""
    row["answer_chars"] = len(answer)
    row["tools_unavailable"] = int(bool(re.search(UNAVAILABLE, answer)))
    row["placeholder_mentions"] = len(re.findall(PLACEHOLDER, answer))
    row["completed"] = int(row["widgets"] > 0 or row["answer_chars"] >= 200)
    row["success"] = int(bool(row["completed"]) and row["connector_calls"] > 0 and row["schema_errors"] == 0
                         and row["writes"] == 0 and not row["tools_unavailable"])
    row.update(turns=f.get("turn_count") or 0, requests=f.get("request_count") or 0,
               input_tokens=f.get("input_tokens") or 0, output_tokens=f.get("output_tokens") or 0,
               cache_read_tokens=f.get("cache_read_tokens") or 0,
               cache_creation_tokens=f.get("cache_creation_tokens") or 0,
               cost_usd=round(((f.get("cost_microdollars") or 0)
                               - (f.get("side_call_cost_microdollars") or 0)) / 1e6, 6),
               duration_s=f.get("duration_seconds") or 0)
    return row


def load_dataset(raw):
    """`analysis-plugin-eval` rows keyed by (client session, dashed skill), or None."""
    f = raw / "datasets" / "analysis-plugin-eval.json"
    if not f.exists() or not f.stat().st_size:
        return None
    body = json.load(open(f, encoding="utf-8"))
    out = {}
    for r in body.get("rows", []) if isinstance(body, dict) else body:
        r = dict(r)
        r["cost_usd"] = float(r.get("cost_usd") or 0)
        r["duration_s"] = r.get("duration_seconds") or 0
        for k in ("completed", "success"):
            r[k] = int(bool(r.get(k)))
        out[(r.get("client_session_id"), r["skill"])] = r
    return out


def consistent_row(bundle, plugin_id, dataset, session_id, dashed_skill, mismatches):
    """The console's figures for one run, and where they came from."""
    local = score_bundle(bundle, plugin_id) if bundle else None
    served = (dataset or {}).get((session_id, dashed_skill))
    if served and local:
        # Why: the console reads every turn; a bundle carries the last few
        # requests of each thread. The console can see more (history the
        # bundle no longer holds), never less, so only a shortfall is a fault.
        for k in RULE_FIELDS:
            if k in ("answer_chars", "completed", "success"):
                continue
            if int(served.get(k) or 0) < int(local.get(k) or 0):
                mismatches.append(f"{dashed_skill}.{k}: console {served.get(k)} < rules {local.get(k)}")
    if served:
        return {k: served.get(k, 0) for k in FACT_FIELDS + RULE_FIELDS}, "console"
    if local:
        return local, "rules"
    return None, "missing"
