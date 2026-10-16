#!/usr/bin/env python3
"""Before/after tables from two sets of run.sh report directories.

  compare.py --before DIR [DIR ...] --after DIR [DIR ...] [--ref c20]
             [--budgets budgets.json]
Prints markdown: ceilings, one row per scenario (ramps at every step both
images ran), pg_stat_statements top-10 and lock waits at the reference step,
statements per request, seal lag, and the mock /stats proof. With --budgets
it also checks the candidate (--after) against the reference (--before) with
the relative budgets in that file and exits 1 on any breach. Standard
library only.
"""
import argparse
import glob
import json
import os
import re
import sys


def runs(dirs):
    out = {}
    for d in dirs:
        for p in sorted(glob.glob(os.path.join(d, "*.json"))):
            if p.endswith("ceilings.json"):
                continue
            r = json.load(open(p))
            out[r["meta"]["name"]] = r
    return out


def ceilings(dirs):
    c = {}
    for d in dirs:
        p = os.path.join(d, "ceilings.json")
        if os.path.exists(p):
            c.update(json.load(open(p)))
    return c


def ms(v):
    return "n/a" if not isinstance(v, (int, float)) else f"{v * 1000:.0f}"


def pod(r, k, f):
    return (r.get("pods", {}).get(k) or {}).get(f, "n/a")


def cells(r):
    if r is None:
        return ["—"] * 10
    pm = r["prometheus"]
    err = r["error_rate"]
    return [
        f"{r['rps']}",
        "n/a" if err is None else f"{err * 100:.1f}%" + ("" if r.get("valid") else " (invalid)"),
        f"{ms(r['e2e_s']['p50'])}/{ms(r['e2e_s']['p99'])}",
        f"{ms(pm.get('overhead_p50_s'))}/{ms(pm.get('overhead_p99_s'))}",
        f"{pod(r, 'postgres', 'cpu_m_avg')}/{pod(r, 'postgres', 'cpu_m_max')}",
        f"{pod(r, 'postgres', 'mem_mi_max')}",
        f"{pod(r, 'gateway', 'cpu_m_avg')}",
        f"{pod(r, 'admin', 'cpu_m_avg')}",
        f"{r.get('statements_per_request', 'n/a')}",
        f"{r['meta'].get('pg_restarts', 'n/a')}",
    ]


def key(name):
    m = re.match(r"(.*?)(?:-c(\d+))?$", name)
    return (m.group(1), int(m.group(2) or 0))


def ratio(cand, ref):
    if not isinstance(cand, (int, float)) or not isinstance(ref, (int, float)) or ref == 0:
        return None
    return cand / ref


def check_budgets(budgets, b, p, cb, cp):
    """Every breach of a relative budget, as human-readable lines."""
    breaches = []
    floor = budgets["ceiling_min_ratio"]
    for k in sorted(set(cb) & set(cp)):
        r = ratio(cp[k], cb[k])
        if cp[k] == "invalid":
            breaches.append(f"ceiling {k}: candidate ramp invalid")
        elif r is not None and r < floor:
            breaches.append(f"ceiling {k}: {cp[k]} < {floor:.0%} of {cb[k]}")
    stmt_cap = budgets["statements_per_request_max_ratio"]
    mem_cap = budgets["pg_mem_max_ratio"]
    denial = set(budgets.get("denial_scenarios", []))
    for name in sorted(set(b) & set(p), key=key):
        rb, rp = b[name], p[name]
        r = ratio(rp.get("statements_per_request"), rb.get("statements_per_request"))
        if r is not None and r > stmt_cap:
            breaches.append(f"{name}: DB stmts/req {rp['statements_per_request']} > "
                            f"{stmt_cap:.0%} of {rb['statements_per_request']}")
        r = ratio(pod(rp, "postgres", "mem_mi_max"), pod(rb, "postgres", "mem_mi_max"))
        if r is not None and r > mem_cap:
            breaches.append(f"{name}: PG mem max {pod(rp, 'postgres', 'mem_mi_max')} Mi > "
                            f"{mem_cap:.0%} of {pod(rb, 'postgres', 'mem_mi_max')} Mi")
    for name, rp in sorted(p.items(), key=lambda kv: key(kv[0])):
        scenario, step = key(name)
        if scenario in denial:
            continue
        ceiling = cp.get(scenario)
        if step and not (isinstance(ceiling, int) and step <= ceiling):
            continue
        err = rp.get("error_rate")
        if err:
            breaches.append(f"{name}: error rate {err * 100:.2f}% in a scenario that should not deny")
    ref = budgets["overhead_ref"]
    if ref in b and ref in p:
        cap = budgets["overhead_p99_max_ratio"]
        ob = b[ref]["prometheus"].get("overhead_p99_s")
        op = p[ref]["prometheus"].get("overhead_p99_s")
        r = ratio(op, ob)
        if r is not None and r > cap:
            breaches.append(f"{ref}: overhead p99 {ms(op)} ms > {cap:.0%} of {ms(ob)} ms")
    else:
        breaches.append(f"{ref}: reference step missing from one side; overhead budget not checked")
    return breaches


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--before", nargs="+", required=True)
    ap.add_argument("--after", nargs="+", required=True)
    ap.add_argument("--ref", default="s1-ramp-buffered-hpa-off-c20")
    ap.add_argument("--budgets")
    a = ap.parse_args()
    b, p = runs(a.before), runs(a.after)
    cb, cp = ceilings(a.before), ceilings(a.after)
    print("### Ceilings (last ramp step that passed the stop rule)\n")
    print("| ramp | 0.64.0 | patched | rps at ceiling (0.64.0 → patched) |\n|---|---|---|---|")
    for k in sorted(set(cb) | set(cp)):
        rb = b.get(f"{k}-c{cb.get(k)}") if isinstance(cb.get(k), int) else None
        rp = p.get(f"{k}-c{cp.get(k)}") if isinstance(cp.get(k), int) else None
        print(f"| {k} | {cb.get(k)} | {cp.get(k)} | {rb and rb['rps']} → {rp and rp['rps']} |")
    hdr = ("| run | image | rps | errors | e2e p50/p99 ms | overhead p50/p99 ms | PG CPU avg/max m | "
           "PG mem max Mi | gateway CPU avg m | admin CPU avg m | DB stmts/req | PG restarts |")
    print("\n### Per scenario\n")
    print(hdr)
    print("|" + "---|" * 12)
    for name in sorted(set(b) | set(p), key=key):
        for label, src in (("0.64.0", b), ("patched", p)):
            r = src.get(name)
            if r is None:
                continue
            print(f"| {name} | {label} | " + " | ".join(cells(r)) + " |")
    for label, src in (("0.64.0", b), ("patched", p)):
        r = src.get(a.ref)
        if not r:
            continue
        print(f"\n### Top statements by total time, {a.ref}, {label}\n")
        print("| total ms | calls | mean ms | query |\n|---|---|---|---|")
        for row in r["statements"]["by_total_time"][:10]:
            if len(row) >= 4:
                print(f"| {row[0]} | {row[1]} | {row[2]} | `{row[-1][:100]}` |")
        print(f"\n### Postgres waits, {a.ref}, {label} (avg active sessions per 1 s sample)\n")
        print("| type | event | relation | avg |\n|---|---|---|---|")
        for w in r["pg_waits"][:10]:
            print(f"| {w['wait_type']} | {w['wait_event']} | {w['relation']} | {w['avg_sessions']} |")
    print("\n### Seal lag (audit_chain_unsealed ai_requests, ai_safety_findings) after each patched run\n")
    for name in sorted(p, key=key):
        print(f"- {name}: {p[name].get('chain_unsealed')}")
    served = ok = 0
    for src in (b, p):
        for r in src.values():
            ms_ = r.get("mock_served") or {}
            served += sum(v for v in ms_.values() if isinstance(v, int))
            ok += r.get("ok", 0)
    print(f"\n### Mock proof\n\nMock /stats served {served} requests across these runs; "
          f"generators recorded {ok} HTTP 200s.")
    if a.budgets:
        breaches = check_budgets(json.load(open(a.budgets)), b, p, cb, cp)
        print("\n### Budgets\n")
        for line in breaches or ["all relative budgets hold"]:
            print(f"- {line}")
        if breaches:
            sys.exit(1)


if __name__ == "__main__":
    main()
