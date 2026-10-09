#!/usr/bin/env python3
"""Summarise one load-test run directory written by run.sh.

Reads hey.csv (every request: response-time, ..., status-code, offset),
waits.tsv, top.txt, statements.tsv, meta.json, and queries Prometheus at the
run's end for prometheus-queries.json. Writes <run>.json and <run>.md beside
the directory. Standard library only (python3 -I safe).
"""
import json
import os
import sys
import urllib.parse
import urllib.request
from collections import Counter, defaultdict

PROM = os.environ.get("LOADTEST_PROM", "http://localhost:19091")
HERE = os.path.dirname(os.path.abspath(__file__))


def pct(sorted_vals, q):
    if not sorted_vals:
        return None
    i = min(len(sorted_vals) - 1, max(0, int(round(q * (len(sorted_vals) - 1)))))
    return sorted_vals[i]


def load_hey(path):
    lat, status = [], Counter()
    first = last = None
    if not os.path.exists(path):
        return lat, status, 0.0
    with open(path) as fh:
        for line in fh:
            parts = line.strip().split(",")
            if len(parts) < 8:
                continue
            try:
                rt, code, off = float(parts[0]), parts[6], float(parts[7])
            except ValueError:
                continue
            status[code] += 1
            if code == "200":
                lat.append(rt)
            first = off if first is None else min(first, off)
            last = off if last is None else max(last, off + rt)
    span = (last - first) if first is not None else 0.0
    return sorted(lat), status, span


def prom(query, at):
    url = PROM + "/api/v1/query?" + urllib.parse.urlencode({"query": query, "time": at})
    try:
        with urllib.request.urlopen(url, timeout=15) as r:
            res = json.load(r)["data"]["result"]
    except Exception as exc:  # noqa: BLE001 - report, never crash the run
        return {"error": str(exc)}
    out = {}
    for s in res:
        key = ",".join(f"{k}={v}" for k, v in sorted(s["metric"].items())) or "value"
        v = float(s["value"][1])
        out[key] = None if v != v else round(v, 6)
    if not out:
        return None
    return out["value"] if list(out) == ["value"] else out


def waits(path):
    agg = defaultdict(int)
    samples = set()
    if os.path.exists(path):
        for line in open(path):
            p = line.rstrip("\n").split("\t")
            if len(p) != 5:
                continue
            samples.add(p[0])
            agg[(p[1], p[2], p[3])] += int(p[4])
    n = max(1, len(samples))
    rows = sorted(((k, v / n) for k, v in agg.items()), key=lambda kv: -kv[1])
    return [{"wait_type": k[0], "wait_event": k[1], "relation": k[2], "avg_sessions": round(v, 2)} for k, v in rows[:15]], len(samples)


def top(path):
    cpu, mem = defaultdict(list), defaultdict(list)
    if os.path.exists(path):
        for line in open(path):
            p = line.split()
            if len(p) < 4:
                continue
            pod = p[1]
            kind = "gateway" if "gateway" in pod else "admin" if "admin" in pod else pod.split("-")[0]
            try:
                cpu[kind].append(int(p[2].rstrip("m")))
                mem[kind].append(int(p[3].rstrip("Mi")))
            except ValueError:
                pass
    return {k: {"cpu_m_max": max(v), "cpu_m_avg": round(sum(v) / len(v)), "mem_mi_max": max(mem[k])} for k, v in cpu.items()}


def statements(path):
    out, sec = {"by_total_time": [], "by_calls": []}, None
    if os.path.exists(path):
        for line in open(path):
            line = line.rstrip("\n")
            if line.startswith("## "):
                sec = line[3:]
                continue
            if sec in out and "\t" in line and not line.startswith(("total_ms", "calls\t")):
                out[sec].append(line.split("\t"))
    return out


def main(run):
    meta = json.load(open(os.path.join(run, "meta.json")))
    lat, status, span = load_hey(os.path.join(run, "hey.csv"))
    total = sum(status.values())
    ok = status.get("200", 0)
    window = f"{int(meta['end'] - meta['start']) + 10}s"
    queries = json.load(open(os.path.join(HERE, "prometheus-queries.json")))
    pm = {k: prom(q.replace("$W", window), meta["end"]) for k, q in queries.items() if not k.startswith("_")}
    w, nsamples = waits(os.path.join(run, "waits.tsv"))

    def stat(path):
        d = {}
        for tok in meta.get(path, "").split():
            k, _, v = tok.partition("=")
            d[k] = int(v) if v.isdigit() else None
        return d
    mb, ma = stat("mock_before"), stat("mock_after")
    mock_served = {k: (ma.get(k) - mb.get(k)) if ma.get(k) is not None and mb.get(k) is not None else None for k in ma}
    errors = total - ok
    summary = {
        "meta": meta,
        "requests": total,
        "ok": ok,
        "status": dict(status),
        "error_rate": round(errors / total, 5) if total else None,
        "rps": round(total / span, 1) if span else None,
        "e2e_s": {q: (round(pct(lat, p), 4) if lat else None) for q, p in (("p50", .5), ("p95", .95), ("p99", .99))},
        "prometheus": pm,
        "mock_served": mock_served,
        "pg_waits": w,
        "pg_wait_samples": nsamples,
        "pods": top(os.path.join(run, "top.txt")),
        "statements": statements(os.path.join(run, "statements.tsv")),
        "log_pressure_lines": open(os.path.join(run, "log-pressure.txt")).read().strip() if os.path.exists(os.path.join(run, "log-pressure.txt")) else None,
    }
    ov99 = pm.get("overhead_p99_s")
    summary["stop"] = bool((errors and any(c.startswith(("5", "4")) for c in status if c != "200"))
                           or (isinstance(ov99, (int, float)) and ov99 > 0.2))
    json.dump(summary, open(run + ".json", "w"), indent=2)

    def ms(v):
        return "n/a" if not isinstance(v, (int, float)) else f"{v * 1000:.0f} ms"
    md = [f"# {meta['name']}", "",
          f"concurrency {meta['concurrency']}, set {meta['set']}, body {meta['body']}, {meta['duration_s']} s, image `{meta['image']}`", "",
          "| metric | value |", "|---|---|",
          f"| requests | {total} |", f"| rps | {summary['rps']} |",
          f"| status | {dict(status)} |", f"| error rate | {summary['error_rate']} |",
          f"| e2e p50/p95/p99 | {ms(summary['e2e_s']['p50'])} / {ms(summary['e2e_s']['p95'])} / {ms(summary['e2e_s']['p99'])} |",
          f"| overhead p50/p95/p99 | {ms(pm.get('overhead_p50_s'))} / {ms(pm.get('overhead_p95_s'))} / {ms(pm.get('overhead_p99_s'))} |",
          f"| mock served | {mock_served} |",
          f"| deployment selected | {pm.get('deployment_selected')} |",
          f"| failovers / shed / quota denials | {pm.get('failovers_total')} / {pm.get('load_shed_total')} / {pm.get('quota_denials_total')} |",
          f"| postgres restarts during run | {meta.get('pg_restarts')} |",
          f"| pods | {summary['pods']} |", f"| stop rule hit | {summary['stop']} |", "",
          "## pg waits (avg active sessions per 1 s sample)", "", "| type | event | relation | avg |", "|---|---|---|---|"]
    md += [f"| {r['wait_type']} | {r['wait_event']} | {r['relation']} | {r['avg_sessions']} |" for r in w]
    md += ["", "## top statements by total time", "", "| total ms | calls | mean ms | query |", "|---|---|---|---|"]
    md += [f"| {r[0]} | {r[1]} | {r[2]} | `{r[-1][:110]}` |" for r in summary["statements"]["by_total_time"] if len(r) >= 4]
    open(run + ".md", "w").write("\n".join(md) + "\n")
    print(f"{meta['name']}: {total} req, {summary['rps']} rps, status {dict(status)}, "
          f"e2e p99 {ms(summary['e2e_s']['p99'])}, overhead p99 {ms(ov99)}, stop={summary['stop']}")


if __name__ == "__main__":
    main(sys.argv[1].rstrip("/"))
