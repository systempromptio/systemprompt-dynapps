"""Generate the vet-group demo dataset as one SQL script on stdout.

Deterministic (fixed RNG seed), so re-running produces identical rows. The
statistical shapes (weekday/hour curve, model mix, token and cost
distributions, tool and hook-event mix, governance decision mix) were read
from a reference tenant as aggregates only; no row, name, prompt or id from
it is reproduced. Every person and organisation here is fictional.

Usage: python3 -I generate.py > /path/to/seed.sql
"""

import hashlib
import json
import math
import random
import sys
import uuid
from pathlib import Path
from datetime import datetime, timedelta, timezone

sys.path.insert(0, str(Path(__file__).resolve().parent))
import org  # noqa: E402

RNG = random.Random(20261018)
NS = uuid.UUID("6f1c2a52-0c1d-4e5b-9a8e-7e3d2b1c0a99")
UTC = timezone.utc
LOCAL_OFFSET = timedelta(hours=2)  # CEST for the whole window (DST ends 2026-10-25)
END_LOCAL = datetime(2026, 10, 18, 20, 0)
START_LOCAL = END_LOCAL.replace(hour=0) - timedelta(days=39)
END_UTC = (END_LOCAL - LOCAL_OFFSET).replace(tzinfo=UTC)


def uid(*parts):
    return str(uuid.uuid5(NS, "|".join(str(p) for p in parts)))


def sha(s):
    return hashlib.sha256(s.encode()).hexdigest()


def ts(dt):
    return dt.strftime("%Y-%m-%d %H:%M:%S.%f+00")


def to_utc(local):
    return (local - LOCAL_OFFSET).replace(tzinfo=UTC)


# ── output helpers ─────────────────────────────────────────────────────────
OUT = sys.stdout
TABLES = {}


def esc(v):
    if v is None:
        return r"\N"
    if isinstance(v, bool):
        return "t" if v else "f"
    if isinstance(v, (dict, list)):
        v = json.dumps(v, ensure_ascii=False)
    if isinstance(v, datetime):
        v = ts(v)
    if isinstance(v, (list, tuple)):
        v = "{" + ",".join(v) + "}"
    s = str(v)
    return s.replace("\\", "\\\\").replace("\t", "\\t").replace("\n", "\\n").replace("\r", "\\r")


def pgarr(items):
    return "{" + ",".join('"' + i.replace('"', '\\"') + '"' for i in items) + "}"


def add(table, cols, row):
    TABLES.setdefault(table, (cols, []))[1].append(row)


def flush(table):
    if table not in TABLES:
        return
    cols, rows = TABLES.pop(table)
    OUT.write(f"COPY {table} ({', '.join(cols)}) FROM STDIN;\n")
    for r in rows:
        OUT.write("\t".join(esc(v) for v in r) + "\n")
    OUT.write("\\.\n")


# ── reference shapes (aggregates) ──────────────────────────────────────────
# Mon..Sun relative volume, from the reference weekday curve, softened.
WEEKDAY_W = [0.85, 1.15, 1.25, 1.0, 0.7, 0.07, 0.1]
# Local hour weights: office day peaking late morning, a lunch dip, a
# smaller afternoon peak, a thin evening tail.
HOUR_W = [0.05, 0.02, 0.01, 0.01, 0.01, 0.03, 0.12, 0.45, 0.85, 1.0, 1.3, 1.55,
          0.8, 0.95, 1.1, 1.0, 0.85, 0.6, 0.3, 0.22, 0.25, 0.18, 0.1, 0.06]
# Model mix (Claude only) with per-model cache-read medians, output medians
# and price per token in microdollars (input, output, cache read, cache write).
MODELS = {
    "claude-opus-5-5": dict(w=0.44, cr=105000, out=360, lat=5100, price=(4.0, 20.0, 0.2, 5.0)),
    "claude-haiku-5-5": dict(w=0.30, cr=34000, out=160, lat=2400, price=(0.1, 0.5, 0.01, 0.125)),
    "claude-sonnet-5-5": dict(w=0.26, cr=120000, out=320, lat=5000, price=(2.0, 10.0, 0.2, 2.5)),
}
# DeepSeek, as declared in services/ai/providers.yaml (route deepseek-route).
# Reachable only by an explicit deepseek-* id, so it is never in the default mix.
DEEPSEEK = {
    "deepseek-flash": dict(w=0.7, cr=60000, out=300, lat=3200, price=(0.3, 1.2, 0.006, 0.3)),
    "deepseek-v4-pro": dict(w=0.3, cr=100000, out=380, lat=6400, price=(1.32, 3.96, 0.044, 1.32)),
}
ALL_MODELS = {**MODELS, **DEEPSEEK}
ROUTE = {"anthropic": "route:claude-star-4203d1", "deepseek": "route:deepseek-route"}

GOV_CHAIN_PASS = [
    {"detail": "No plaintext secrets detected in governed input", "result": "pass", "policy_id": "secret_scan"},
    {"detail": "Not a tool call", "result": "pass", "policy_id": "scope_check"},
    {"detail": "Not a tool call", "result": "pass", "policy_id": "tool_blocklist"},
]

SECRET_PATTERNS = [
    ("aws-secret-key", "AWS Secret Key", "AWS_SECRET_A...[REDACTED]"),
    ("postgres-url-with-password", "Postgres URL with password", "postgresql:/...[REDACTED]"),
    ("high-entropy-token", "High-entropy token (possible credential)", "odoo_api_ke...[REDACTED]"),
    ("bearer-token", "Bearer token", "Bearer eyJh...[REDACTED]"),
]

# ── project palettes: skills, MCP tools, prompts ───────────────────────────
# MCP tools: (server, tool, write?, source, model hint for input)
T_ODOO_READ = [
    ("dynapps-odoo", "orm_search_read", False), ("dynapps-odoo", "orm_aggregate", False),
    ("dynapps-odoo", "odoo_resolve", False), ("dynapps-odoo", "odoo_describe_model", False),
]
T_ODOO_PARTNER = [("odoo", "partner_search", False), ("odoo", "partner_get", False)]
T_ODOO_WRITE = [("odoo", "note_add", True), ("odoo", "activity_create", True)]
T_BRAIN = [("dynbrain", "docs_search", False), ("dynbrain", "docs_list_spaces", False)]
T_BRAIN_WRITE = [("dynbrain", "docs_create", True)]

PROJECT_PALETTE = {
    "vg-dunning": dict(
        skills=["dynapps-farm:dynapps_accounting_auditor", "dynapps-portal:dynapps_portal_guide"],
        mcp=T_ODOO_READ * 3 + T_ODOO_PARTNER * 2 + T_ODOO_WRITE * 2,
        models=("account.move", "res.partner", "mail.activity"),
        prompts=[
            "List the invoices of Kliniek Noord older than 45 days with no reminder sent yet.",
            "Draft a friendly second reminder for the overdue invoices of this client, in Dutch.",
            "Which clients crossed the 60-day threshold this week? Group by clinic.",
            "Schedule a follow-up call activity for the three largest overdue balances.",
            "Summarise the dunning status for Clinics BE for the Monday finance stand-up.",
        ]),
    "vg-ticket-followup": dict(
        skills=["dynapps-showcase:dynapps_ticket_solver", "dynapps-portal:dynapps_portal_guide"],
        mcp=T_ODOO_READ * 2 + T_ODOO_WRITE + T_BRAIN * 2,
        models=("helpdesk.ticket", "res.partner"),
        prompts=[
            "Open helpdesk tickets from the clinics with no reply in 48 hours — who owns them?",
            "Investigate ticket about the label printer at the Gent-Zuid reception and propose a fix.",
            "Write a closure note for this ticket and log it in Odoo.",
            "Which ticket categories grew this month across clinics?",
        ]),
    "vg-lab-results": dict(
        skills=["dynapps-portal:dynapps_portal_guide"],
        mcp=T_BRAIN * 3 + T_ODOO_READ * 2 + T_BRAIN_WRITE,
        models=("lab.result", "lab.sample"),
        prompts=[
            "Summarise this haematology panel for the referring vet in plain language.",
            "Compare the renal values with the previous sample of the same patient and flag changes.",
            "Which reference range applies to feline T4 in our lab handbook?",
            "Draft the cover note for the weekly results batch to the ambulatory team.",
        ]),
    "vg-no-shows": dict(
        skills=["dynapps-portal:dynapps_portal_guide", "dynapps-farm:dynapps_agent_farm"],
        mcp=T_ODOO_READ * 3 + T_ODOO_PARTNER * 2 + T_ODOO_WRITE,
        models=("calendar.event", "res.partner"),
        prompts=[
            "Which appointments yesterday were no-shows, per clinic?",
            "Draft a rebooking SMS for these clients, warm tone, no blame.",
            "Is there a pattern in Monday-morning no-shows over the last four weeks?",
            "Create follow-up activities for front desk on today's no-shows.",
        ]),
    "vg-invoice-recon": dict(
        skills=["dynapps-farm:dynapps_accounting_auditor"],
        mcp=T_ODOO_READ * 4 + T_ODOO_PARTNER + T_ODOO_WRITE,
        models=("account.bank.statement.line", "account.move", "account.payment"),
        prompts=[
            "Match last week's bank statement lines for Clinics NL against open invoices.",
            "Which payments are unreconciled for more than 10 days, by entity?",
            "Explain the difference between the AR ledger and the bank balance for September.",
            "Propose reconciliation for these 12 lines and note the ones you are unsure of.",
        ]),
    "vg-metrics-pilot": dict(
        skills=["dynapps-engineering:agent_farm_runs", "dynapps-showcase:dynapps_dev_assessment"],
        mcp=T_ODOO_READ * 4 + T_BRAIN,
        models=("account.move.line", "sale.order"),
        prompts=[
            "Revenue per clinic last month from the metrics tool, compared with the same month last year.",
            "Define 'active client' for the pilot and check it against the Odoo replica.",
            "Why does the clinic revenue number differ from the finance pack by 2%?",
            "Draft the metric definition note for treatment margin.",
        ]),
}
BUILTINS_CODE = [("Read", 6), ("Bash", 9), ("Grep", 2), ("Edit", 3), ("Write", 1), ("WebSearch", 1), ("Glob", 1)]
BUILTINS_DESKTOP = [("WebSearch", 2), ("Read", 2), ("WebFetch", 1)]

ASSISTANT_LINES = [
    "Done. I pulled the records through the governed Odoo tool and summarised them below.",
    "Here is the draft; I kept the amounts and dates exactly as they are in Odoo.",
    "I found the matching records. Three need a human decision, listed at the end.",
    "Summary ready. Nothing was written to Odoo; say the word and I will log the activities.",
    "I logged the follow-up activities and added a note on each record.",
]


def pick_w(pairs):
    tot = sum(w for _, w in pairs)
    x = RNG.random() * tot
    for v, w in pairs:
        x -= w
        if x <= 0:
            return v
    return pairs[-1][0]


def lognorm(median, sigma):
    return median * math.exp(RNG.gauss(0, sigma))


# ── organisation ───────────────────────────────────────────────────────────
NOW = datetime(2026, 10, 16, 12, 0, tzinfo=UTC)
CREATED = datetime(2026, 9, 1, 8, 0, tzinfo=UTC)
users = {}  # slug -> dict
for slug, name, group, roles, intensity in org.PEOPLE:
    users[slug] = dict(id=uid("user", slug), slug=slug, name=name, group=group, roles=roles,
                       intensity=intensity, email=f"{slug}@{org.DOMAIN}", kind="person")
for slug, name, desc in org.SERVICE_ACCOUNTS:
    users[slug] = dict(id=uid("user", slug), slug=slug, name=name, group="service-accounts",
                       roles=["user"], intensity=0, email=f"{slug}@{org.DOMAIN}", kind="service", desc=desc)
admin_id = users["vincent.moreau"]["id"]
acq_start = datetime(2026, 10, 5, 7, 0, tzinfo=UTC)

for u in users.values():
    created = acq_start if u["group"] == "vg-clinics-de-acquired" else CREATED
    add("users", ["id", "name", "email", "full_name", "display_name", "status", "email_verified", "roles",
                  "created_at", "updated_at", "account_kind"],
        [u["id"], u["slug"], u["email"], u["name"], u["name"], "active", True, pgarr(u["roles"]),
         created, created, u["kind"]])

for gid, gname, adg, entity, created in org.GROUPS:
    c = datetime.fromisoformat(created).replace(hour=8, tzinfo=UTC)
    add("groups", ["id", "name", "description", "is_system", "source", "created_at", "updated_at"],
        [gid, gname, org.GROUP_DESCRIPTIONS[gid], False, "dashboard", c, c])
    add("group_ad_mappings", ["ad_group", "group_id", "source", "created_at"], [adg, gid, "dashboard", c])

ad_of = {g[0]: g[2] for g in org.GROUPS}
for u in users.values():
    if u["kind"] == "service":
        add("group_members", ["group_id", "user_id", "source", "granted_by", "created_at", "valid_from"],
            ["service-accounts", u["id"], "manual", admin_id, CREATED, CREATED])
        continue
    c = acq_start if u["group"] == "vg-clinics-de-acquired" else CREATED
    add("group_members", ["group_id", "user_id", "source", "source_ad_group", "created_at", "valid_from"],
        [u["group"], u["id"], "adfs", ad_of[u["group"]], c, c])

for pid, pname, cc, practice, desc in org.PROJECTS:
    add("projects", ["id", "name", "description", "source", "created_at", "updated_at", "cost_centre", "practice"],
        [pid, pname, desc, "dashboard", CREATED, CREATED, cc, practice])
for u in users.values():
    if u["kind"] == "service":
        continue
    projs = org.GROUP_PROJECTS[u["group"]]
    u["projects"] = projs
    c = acq_start if u["group"] == "vg-clinics-de-acquired" else CREATED
    for p in projs:
        add("project_members", ["project_id", "user_id", "source", "granted_by", "created_at", "valid_from"],
            [p, u["id"], "manual", admin_id, c, c])
    add("user_scope_defaults", ["user_id", "primary_group_id", "primary_project_id", "source", "updated_at"],
        [u["id"], u["group"], projs[0], "manual", c])
    for r in u["roles"]:
        if r != "user":
            add("user_manual_roles", ["user_id", "role", "granted_by", "created_at", "valid_from"],
                [u["id"], r, admin_id, c, c])
for u in users.values():
    if u["kind"] == "service":
        add("user_scope_defaults", ["user_id", "primary_group_id", "primary_project_id", "source", "updated_at"],
            [u["id"], "service-accounts", None, "manual", CREATED])

TAG = "[vetgroup-demo] "
RULE_COLS = ["id", "entity_type", "entity_id", "rule_type", "rule_value", "access", "justification", "source",
             "created_at", "updated_at"]
for mp, groups in org.MARKETPLACE_GROUP_RULES.items():
    for g in groups:
        add("access_control_rules", RULE_COLS,
            [uid("rule", mp, g), "marketplace", mp, "group", g, "allow",
             TAG + f"Entity group {g} reaches the {mp} workspace; mapped from Entra ID.", "dashboard", CREATED, CREATED])
for mp, projs in org.MARKETPLACE_PROJECT_RULES.items():
    for p in projs:
        add("access_control_rules", RULE_COLS,
            [uid("rule", mp, p), "marketplace", mp, "project", p, "allow",
             TAG + f"Project {p} runs on the farm workspace.", "dashboard", CREATED, CREATED])
for server, band, value, access, why in org.MCP_RULES:
    add("access_control_rules", RULE_COLS,
        [uid("rule", server, band, value), "mcp_server", server, band, value, access, TAG + why, "dashboard",
         CREATED, CREATED])
# One person-band exception: the credit controller keeps farm access on a time limit.
add("access_control_rules", RULE_COLS,
    [uid("rule", "person", "lotte"), "mcp_server", "dynapps-odoo", "person", users["lotte.de.smet"]["id"], "deny",
     TAG + "Read-only during probation; person band overrides the Clinics BE finance group allow.", "dashboard",
     CREATED, CREATED])

for cid, cname, slug, svc in org.CLIENTS:
    add("clients", ["id", "name", "slug", "created_at", "service_account_user_id"],
        [cid, cname, slug, CREATED, users[svc]["id"] if svc else None])

# ── usage ──────────────────────────────────────────────────────────────────
REQ_COLS = ["id", "request_id", "user_id", "session_id", "context_id", "gateway_conversation_id",
            "client_session_id", "trace_id", "provider", "model", "requested_model", "route_match", "max_tokens",
            "tokens_used", "input_tokens", "output_tokens", "cost_microdollars", "latency_ms",
            "upstream_latency_ms", "finish_reason", "cache_hit", "cache_read_tokens", "cache_creation_tokens",
            "reasoning_tokens", "is_streaming", "status", "error_message", "actor_kind", "actor_id",
            "request_kind", "client_kind", "client_attestation", "message_count", "created_at", "updated_at",
            "completed_at", "has_offered_tools", "wire_protocol"]
MCP_COLS = ["mcp_execution_id", "tool_name", "server_name", "started_at", "completed_at", "execution_time_ms",
            "input", "output", "status", "error_message", "user_id", "session_id", "context_id", "trace_id",
            "request_method", "request_source", "actor_kind", "actor_id", "ai_tool_call_id", "source",
            "correlation", "payload_sha256", "created_at", "input_summary", "is_builtin", "artifact_kind"]
GOV_COLS = ["id", "user_id", "session_id", "tool_name", "agent_id", "agent_scope", "decision", "policy", "reason",
            "evaluated_rules", "plugin_id", "actor_kind", "actor_id", "context_id", "trace_id", "client_id",
            "tool_use_id", "created_at"]
EVT_COLS = ["id", "user_id", "session_id", "event_type", "tool_name", "plugin_id", "metadata", "dedup_key",
            "prompt_preview", "description", "cwd", "content_input_bytes", "content_output_bytes", "tool_use_id",
            "trace_id", "mcp_execution_id", "created_at"]
TC_COLS = ["id", "request_id", "tool_name", "tool_input", "mcp_execution_id", "ai_tool_call_id",
           "sequence_number", "created_at", "updated_at"]
MSG_COLS = ["id", "request_id", "role", "content", "sequence_number", "name", "tool_call_id", "created_at",
            "updated_at"]
SF_COLS = ["id", "ai_request_id", "phase", "severity", "category", "scanner", "excerpt", "blocked", "created_at"]
EV_COLS = ["ai_request_id", "kind_source", "attested_host", "native_marker", "ua_product", "ua_version",
           "sdk_os", "created_at"]
SCOPE_UPD = []  # (request id, project id)

counters = dict(requests=0, gov=0, mcp=0, events=0, findings=0)
rate_state = {}
bridge_sessions = set()
contexts = []
CONV_META = []


def gov_row(u, session_id, tool, decision, policy, reason, rules, ctx, trace, at, tool_use_id=None,
            actor_kind="user", client="sp_bridge"):
    gid = uid("gov", session_id, tool, at.isoformat(), tool_use_id or "", RNG.random())
    add("governance_decisions", GOV_COLS,
        [gid, u["id"], session_id, tool, None, "user", decision, policy, reason, rules, None, actor_kind,
         u["id"], ctx, trace, client, tool_use_id, at])
    counters["gov"] += 1


def prompt_decision(u, sess, ctx, trace, at, warn):
    principal = {"user_id": u["id"], "agent_id": None, "client_id": "sp_bridge", "session_id": sess,
                 "agent_scope": "user", "agent_session": None}
    chain = [dict(c, duration_ms=round(RNG.uniform(0.001, 3.0), 4)) for c in GOV_CHAIN_PASS]
    if warn is None:
        chain.append({"detail": f"{RNG.randint(1, 60)}/300 calls in 60s window", "result": "pass",
                      "policy_id": "rate_limit", "duration_ms": 0.02})
        decision, policy, reason = "allow", "default_allow", ""
        dec = {"decision": "allow", "matched_by": {"kind": "default_included"}}
    elif warn == "quota":
        window = RNG.choice([("86400", "user"), ("86400", "group"), ("2592000", "project")])
        reason = (f"quota window {window[0]}s for subject '{window[1]}' at "
                  f"{RNG.randint(81, 99)}% of its warning threshold")
        chain.append({"detail": reason, "result": "warn", "policy_id": "quota", "duration_ms": 0.04})
        decision, policy = "warn", "quota"
        dec = {"reason": {"kind": "policy_violation", "detail": reason, "policy": "quota"}, "decision": "warn"}
    elif warn == "rate_limit":
        reason = ('rate limit RateLimitWindow { name: "rate_limit", seconds: 60, limit: 300 } exceeded; '
                  'retry after 60000ms')
        chain.append({"detail": reason, "result": "warn", "policy_id": "rate_limit", "duration_ms": 0.008})
        decision, policy = "warn", "rate_limit"
        dec = {"reason": {"kind": "rate_limit_exceeded", "window": {"name": "rate_limit", "limit": 300,
                                                                     "seconds": 60}, "retry_after_ms": 60000},
               "decision": "warn"}
    else:  # secret
        pid, pname, red = RNG.choice(SECRET_PATTERNS)
        path = f"messages[{RNG.randint(1, 30)}].content"
        reason = f"secret detected: {pname} at prompt.forwarded.$.{path} ({red})"
        chain[0] = {"detail": reason, "result": "warn", "policy_id": "secret_scan", "duration_ms": 1.4}
        decision, policy = "warn", "secret_scan"
        dec = {"reason": {"kind": "secret_leak", "location": {"kind": "prompt", "path": path, "redacted": red},
                          "pattern_id": pid, "pattern_name": pname}, "decision": "warn"}
    chain.append({"detail": "Policy disabled in governance config", "result": "disabled",
                  "policy_id": "require_approval", "duration_ms": 0.0})
    rid = uid("govp", sess, at.isoformat())
    rules = {"id": rid, "chain": chain, "origin": "governed",
             "target": {"plugin_id": None, "tool_name": "user_prompt"}, "call_id": uid("call", rid),
             "decision": dec, "trace_id": trace, "principal": principal, "context_id": ctx,
             "detector_version": "0.69.0-calibration-1"}
    gov_row(u, sess, "user_prompt", decision, policy, reason, rules, ctx, trace, at)


def tool_decision(u, sess, ctx, trace, at, full_name, tool_use_id, server, deny=False):
    if deny:
        reason = (f"mcp_server:{server}: denied to user {u['id']} by group rule "
                  f"({u['group']}; deny beats allow in the group band)")
        rules = {"tool": full_name, "server": server, "band": "group", "rule_value": u["group"], "access": "deny"}
        gov_row(u, sess, full_name, "deny", "authz_rule_based", reason, rules, ctx, trace, at, tool_use_id,
                client=None)
        return
    if server in ("dynapps-odoo", "odoo", "dynbrain"):
        rules = {"tool": full_name, "server": server, "band": "group", "rule_value": u["group"], "access": "allow"}
        gov_row(u, sess, full_name, "allow", "authz_rule_based", "", rules, ctx, trace, at, tool_use_id,
                client=None)
    else:
        gov_row(u, sess, full_name, "allow", "default_allow", "", {"tool": full_name}, ctx, trace, at,
                tool_use_id, client=None)


def mcp_input(server, tool, palette):
    model = RNG.choice(palette["models"])
    if tool in ("orm_search_read",):
        return {"model": model, "domain": [["company_id", "=", RNG.randint(2, 6)]], "limit": 80}
    if tool in ("orm_aggregate", "odoo_aggregate"):
        return {"model": model, "group_by": ["company_id"], "measures": ["amount_residual:sum"]}
    if tool == "odoo_resolve":
        return {"model": "res.partner", "name": RNG.choice(["Kliniek Noord", "Dierenpraxis Oost", "Clinique Sud"])}
    if tool == "odoo_describe_model":
        return {"model": model}
    if tool in ("partner_search",):
        return {"query": RNG.choice(["overdue", "client", "boarding"]), "limit": 20}
    if tool in ("partner_get",):
        return {"id": RNG.randint(1000, 9000)}
    if tool == "note_add":
        return {"model": model, "id": RNG.randint(1000, 9000), "body": "Follow-up logged by assistant"}
    if tool == "activity_create":
        return {"model": model, "id": RNG.randint(1000, 9000), "activity_type": "call",
                "date_deadline": "2026-10-20"}
    if tool == "docs_search":
        return {"query": RNG.choice(["reference range", "rebooking script", "dunning letter template",
                                     "sample intake", "protocol"])}
    if tool == "docs_create":
        return {"space": "lab-reports", "title": "Weekly results digest"}
    return {}


def emit_tool(u, sess_hook, bridge, ctx, trace, at, req_id, seq, tool, palette, client_kind, deny_odoo):
    """One tool call: request intent, governance, execution, hook event."""
    tool_use_id = "toolu_" + sha(f"{req_id}{seq}")[:24]
    if isinstance(tool, tuple):
        server, name, write = tool
        full = f"mcp__{server}__{name}"
        inp = mcp_input(server, name, palette)
    else:
        server, name, write, full = "builtin", tool, False, tool
        inp = {"Read": {"file_path": "/home/user/work/notes.md"}, "Bash": {"command": "ls -la reports/"},
               "Grep": {"pattern": "invoice", "path": "."}, "Edit": {"file_path": "summary.md"},
               "Write": {"file_path": "summary.md"}, "WebSearch": {"query": "veterinary reference values"},
               "Glob": {"pattern": "**/*.csv"}, "WebFetch": {"url": "https://example.org/guide"}}.get(name, {})
    denied = deny_odoo and server == "dynapps-odoo"
    tool_decision(u, sess_hook, ctx, trace, at, full, tool_use_id, server, deny=denied)
    exec_id = uid("mcp", req_id, seq)
    dur = int(lognorm(420 if server != "builtin" else 60, 0.7))
    failed = denied or RNG.random() < 0.035
    if server == "builtin":
        src, srv, builtin, art = "hook_claude_code", "hook_claude_code", True, None
    else:
        src = "in_process" if server == "odoo" else "proxy"
        srv, builtin = server, False
        art = "card" if write else RNG.choice(["table", "table", "text"])
    out = None if server == "builtin" else json.dumps(
        {"artifact_type": art, "title": name.replace("_", " ").title(),
         "content": f"{RNG.randint(1, 40)} records" if not write else "Logged"})
    add("mcp_tool_executions", MCP_COLS,
        [exec_id, name if server != "builtin" else name, srv, at, at + timedelta(milliseconds=dur), dur,
         json.dumps(inp), out, "failed" if failed else "success",
         ("denied by access-control rule" if denied else ("upstream timeout" if failed else None)),
         u["id"], bridge if server != "builtin" else sess_hook, ctx, trace, "mcp",
         srv, "user", u["id"], tool_use_id, src, "exact", sha(json.dumps(inp) + exec_id), at,
         next(iter(inp.values()), "") if inp else "", builtin, art])
    counters["mcp"] += 1
    add("ai_request_tool_calls", TC_COLS,
        [uid("tc", req_id, seq), req_id, full, json.dumps(inp), exec_id, tool_use_id, seq, at, at])
    meta = {"cwd": "/home/user/work", "effort": {"level": "medium"}, "event_id": "envelope-" + sha(exec_id)[:32],
            "prompt_id": uid("prompt", sess_hook, at.date()), "tool_name": full, "tool_input": inp,
            "duration_ms": dur, "session_id": sess_hook, "tool_use_id": tool_use_id,
            "hook_event_name": "PostToolUseFailure" if failed else "PostToolUse", "permission_mode": "default"}
    if server != "builtin":
        meta["mcp_server"] = server
    if failed:
        meta["error"] = "denied" if denied else "timeout"
    hook_event(u, sess_hook, "PostToolUseFailure" if failed else "PostToolUse", at + timedelta(milliseconds=dur),
               tool=full, meta=meta, tool_use_id=tool_use_id, exec_id=exec_id,
               desc=("Failed: " if failed else "Completed: ") + full)
    return write, failed


def hook_event(u, sess, etype, at, tool=None, meta=None, prompt=None, tool_use_id=None, exec_id=None, desc=None):
    eid = uid("evt", sess, etype, at.isoformat(), tool or "", tool_use_id or "")
    meta = meta or {"cwd": "/home/user/work", "session_id": sess, "hook_event_name": etype}
    add("plugin_usage_events", EVT_COLS,
        [eid, u["id"], sess, etype, tool, "dynapps-commons", meta, sha(eid), prompt,
         desc or (("Asked: " + prompt[:120]) if prompt else etype), "/home/user/work",
         len(json.dumps(meta)), RNG.randint(0, 4000) if tool else 0, tool_use_id, sess, exec_id, at])
    counters["events"] += 1


def skill_call(u, sess_hook, bridge, ctx, trace, at, req_id, skill):
    tool_use_id = "toolu_" + sha(f"{req_id}skill")[:24]
    tool_decision(u, sess_hook, ctx, trace, at, "Skill", tool_use_id, "builtin")
    exec_id = uid("mcp", req_id, "skill")
    add("mcp_tool_executions", MCP_COLS,
        [exec_id, "Skill", "gateway", at, None, None, json.dumps({"skill": skill}), None, "success", None,
         u["id"], bridge, ctx, trace, "mcp", "gateway", "user", u["id"], tool_use_id, "gateway", "exact",
         sha(skill + exec_id), at, skill.split(":")[-1], True, None])
    counters["mcp"] += 1
    add("ai_request_tool_calls", TC_COLS,
        [uid("tc", req_id, "skill"), req_id, "Skill", json.dumps({"skill": skill}), exec_id, tool_use_id, 0, at, at])
    meta = {"cwd": "/home/user/work", "event_id": "envelope-" + sha(exec_id)[:32], "tool_name": "Skill",
            "tool_input": {"skill": skill}, "session_id": sess_hook, "tool_use_id": tool_use_id,
            "hook_event_name": "PostToolUse", "duration_ms": RNG.randint(20, 90)}
    hook_event(u, sess_hook, "PostToolUse", at + timedelta(milliseconds=60), tool="Skill", meta=meta,
               tool_use_id=tool_use_id, exec_id=exec_id, desc="Completed: Skill")


def safety(req_id, at, phase, category, severity, scanner, excerpt):
    add("ai_safety_findings", SF_COLS,
        [uid("sf", req_id, category, phase), req_id, phase, severity, category, scanner, excerpt, False, at])
    counters["findings"] += 1


def ramp(day_index):
    # Adoption: a pilot cohort first, the rest within ~3 weeks, then steady.
    return min(1.0, 0.3 + 0.7 * (day_index / 22.0) ** 1.3)


def run_session(u, start, n_req, project, client_kind, day_key, idx, forced_model=None, forced_prompt=None, shape_model=None):
    palette = PROJECT_PALETTE[project]
    sess_hook = uid("hooksess", u["slug"], day_key, idx)
    bridge = "sess_" + uid("bridge", u["slug"], day_key)
    bridge_sessions.add((bridge, u["id"], day_key))
    ctx = uid("ctx", u["slug"], day_key, idx)
    contexts.append((ctx, u["id"], sess_hook))
    gconv = "ctx_" + sha(ctx)[:16]
    deny_odoo = u["group"] == "vg-clinics-de-acquired"
    heavy = u["intensity"] >= 1.4
    builtins = BUILTINS_CODE if client_kind == "claude-code" else BUILTINS_DESKTOP
    t = start
    hook_event(u, sess_hook, "SessionStart", t)
    prompt_text = None
    skill_used = False
    first = True
    for i in range(n_req):
        new_prompt = first or RNG.random() < 0.2
        if new_prompt:
            if not first:
                t += timedelta(seconds=RNG.randint(90, 900))
                hook_event(u, sess_hook, "Stop", t - timedelta(seconds=5))
            prompt_text = forced_prompt or RNG.choice(palette["prompts"])
            slash = first and RNG.random() < 0.08
            if slash:
                sk = RNG.choice(palette["skills"]).split(":")
                prompt_text = f"/{sk[0]}:{sk[1].replace('_', '-')} " + prompt_text
            if first and RNG.random() < 0.05:
                prompt_text = "/dynapps-commons:who-am-i"
            hook_event(u, sess_hook, "UserPromptSubmit", t, prompt=prompt_text,
                       meta={"cwd": "/home/user/work", "prompt_id": uid("prompt", sess_hook, t.isoformat()),
                             "session_id": sess_hook, "hook_event_name": "UserPromptSubmit",
                             "permission_mode": "default"})
        if t > END_UTC:
            break
        model = forced_model or pick_w([(m, v["w"]) for m, v in MODELS.items()])
        if not forced_model and client_kind == "claude-code" and i > 0 and RNG.random() < 0.25:
            model = "claude-haiku-5-5"  # tool-loop side calls
        # A/B pairs keep the Claude run's token shape and price only the model
        spec = dict(ALL_MODELS[shape_model or model], price=ALL_MODELS[model]["price"])
        provider = "deepseek" if model in DEEPSEEK else "anthropic"
        probe = i == 0 and RNG.random() < 0.06
        req_id = uid("req", sess_hook, i)
        trace = uid("trace", req_id)
        status = "completed"
        if RNG.random() < 0.012:
            status = "failed"
        cr = 0 if probe else int(lognorm(spec["cr"] * (0.25 + 0.75 * min(1, i / 6)), 0.45))
        cw = 0 if probe else int(lognorm(900, 0.9))
        inp = RNG.randint(300, 500) if probe else RNG.randint(1, 60)
        out = 1 if probe else int(lognorm(spec["out"], 0.6))
        p_in, p_out, p_cr, p_cw = spec["price"]
        cost = 0 if status == "failed" else int(inp * p_in + out * p_out + cr * p_cr + cw * p_cw)
        lat = int(lognorm(spec["lat"] * (0.3 if probe else 1), 0.45))
        done = t + timedelta(milliseconds=lat)
        n_tools = 0 if probe else (0 if RNG.random() < 0.4 else (1 if RNG.random() < 0.75 else 2))
        msg_target = 2 + i * 2 + RNG.randint(0, 6)
        add("ai_requests", REQ_COLS,
            [req_id, req_id, u["id"], bridge, ctx, gconv, sess_hook, trace, provider, model, model,
             ROUTE[provider], 64000,
             None if status == "failed" else inp + out + cr + cw, None if status == "failed" else inp,
             None if status == "failed" else out, cost, lat if status == "completed" else 0,
             int(lat * 0.92), None if status == "failed" else ("tool_use" if n_tools else "end_turn"),
             cr > 0, None if status == "failed" else cr, None if status == "failed" else cw,
             0, True, status, "upstream overloaded (529)" if status == "failed" else None, "user", u["id"],
             "probe" if probe else "turn", client_kind, "native-marker", max(0, msg_target - 2), t, done,
             done, n_tools > 0, "anthropic.messages"])
        counters["requests"] += 1
        SCOPE_UPD.append((req_id, project))
        add("ai_request_client_evidence", EV_COLS,
            [req_id, "native-marker", client_kind,
             "claude-cli-entrypoint" if client_kind == "claude-code" else "claude-desktop-entrypoint",
             "claude-cli" if client_kind == "claude-code" else "Claude", "2.1.290" if client_kind ==
             "claude-code" else "1.4.8", RNG.choice(["darwin", "win32", "win32", "linux"]), t])
        add("ai_request_messages", MSG_COLS,
            [uid("msg", req_id, 0), req_id, "user", prompt_text or "continue", 0, None, None, t, t])
        add("ai_request_messages", MSG_COLS,
            [uid("msg", req_id, 1), req_id, "assistant", RNG.choice(ASSISTANT_LINES), 1, None, None, done, done])
        # gateway governance for the prompt
        warn = None
        x = RNG.random()
        bucket = rate_state.setdefault((u["id"], t.replace(minute=0, second=0, microsecond=0)), [0])
        bucket[0] += 1
        if heavy and bucket[0] > 22 and RNG.random() < 0.5:
            warn = "rate_limit"
        elif x < 0.055 * (1.6 if heavy else 1):
            warn = "quota"
        elif x < 0.062:
            warn = "secret"
        prompt_decision(u, bridge, ctx, trace, t - timedelta(milliseconds=40), warn)
        # safety scanners (both directions)
        if not probe:
            if RNG.random() < 0.6:
                safety(req_id, done, "response", "secret_observation", "low", "secrets",
                       "high-entropy-token: fingerprint:" + sha(req_id)[:64] + "...[REDACTED]")
            if u["group"].endswith("frontdesk") or "finance" in u["group"]:
                if RNG.random() < 0.18:
                    safety(req_id, t, "request", "pii_email", "low", "heuristic", "email: c***@mail.example")
                if RNG.random() < 0.1:
                    safety(req_id, t, "request", "pii_phone", "medium", "pii_extended", "phone: +32 4** ** ** **")
            if "finance" in u["group"] and RNG.random() < 0.004:
                safety(req_id, t, "request", "pii_credit_card", "high", "heuristic", "card: 4*** **** **** ****")
            if (u["group"].endswith("frontdesk") or u["group"] == "vg-clinics-de-acquired") and RNG.random() < 0.006:
                # national register number in a client note: would block under enforce
                safety(req_id, t, "request", "pii_ssn", "high", "pii_extended", "national id: **.**.**-***.**")
            if warn == "secret" or RNG.random() < 0.0012:
                safety(req_id, done, "response", "secret", "high", "secrets",
                       "credential: odoo_api_ke...[REDACTED]")
        # tool calls
        tt = done
        if not skill_used and not probe and RNG.random() < 0.55:
            skill_call(u, sess_hook, bridge, ctx, trace, tt, req_id, RNG.choice(palette["skills"]))
            skill_used = True
            tt += timedelta(milliseconds=200)
        for k in range(n_tools):
            if RNG.random() < (0.65 if client_kind == "claude-desktop" else 0.45):
                tool = RNG.choice(palette["mcp"])
            else:
                tool = pick_w(builtins)
            emit_tool(u, sess_hook, bridge, ctx, trace, tt, req_id, k + 1, tool, palette, client_kind,
                      deny_odoo)
            tt += timedelta(milliseconds=RNG.randint(300, 2500))
        t = max(tt, done) + timedelta(seconds=int(RNG.expovariate(1 / 25)) + 2)
        first = False
    CONV_META.append((ctx, u["id"], sess_hook, project, t, n_req))
    hook_event(u, sess_hook, "Stop", t)
    hook_event(u, sess_hook, "SessionEnd", t + timedelta(seconds=3),
               meta={"cwd": "/home/user/work", "reason": "prompt_input_exit", "session_id": sess_hook,
                     "hook_event_name": "SessionEnd"})


people = [u for u in users.values() if u["kind"] == "person"]
DEEPSEEK_FROM = datetime(2026, 9, 28)  # the evaluation starts mid-window
day = START_LOCAL
day_index = 0
while day <= END_LOCAL:
    wd = day.weekday()
    for u in people:
        if u["group"] == "vg-clinics-de-acquired" and day < datetime(2026, 10, 6):
            continue
        lam = 22 * u["intensity"] * WEEKDAY_W[wd] * ramp(day_index)
        if u["group"] == "vg-ambulatory-field" and wd >= 5:
            lam *= 4  # on-call weekends
        if u["group"] == "vg-clinics-de-acquired":
            lam *= 0.5 + 0.5 * min(1, (day - datetime(2026, 10, 6)).days / 8)
        if RNG.random() < 0.12:  # day off / leave
            continue
        n_day = int(lognorm(lam, 0.5)) if lam > 0.5 else (1 if RNG.random() < lam else 0)
        if n_day <= 0:
            continue
        day_key = day.strftime("%Y%m%d")
        idx = 0
        client_kind = "claude-code" if u["group"] in ("vg-data-solutions",) or (
            "finance" in u["group"] and RNG.random() < 0.35) else "claude-desktop"
        while n_day > 0:
            n = max(1, min(n_day, int(RNG.expovariate(1 / 11)) + 2))
            hour = pick_w(list(enumerate(HOUR_W)))
            start_local = day.replace(hour=hour, minute=RNG.randint(0, 59), second=RNG.randint(0, 59))
            start = to_utc(start_local) + timedelta(microseconds=RNG.randint(0, 999999))
            if start > END_UTC - timedelta(minutes=10):
                n_day -= n
                continue
            project = RNG.choice(u["projects"]) if RNG.random() < 0.4 else u["projects"][0]
            ck = client_kind if RNG.random() < 0.85 else (
                "claude-code" if client_kind == "claude-desktop" else "claude-desktop")
            forced = None
            if (u["group"] == "vg-data-solutions" or project == "vg-metrics-pilot") and \
                    day >= DEEPSEEK_FROM and RNG.random() < 0.3:
                forced = pick_w([(m, v["w"]) for m, v in DEEPSEEK.items()])
            run_session(u, start, n, project, ck, day_key, idx, forced_model=forced)
            idx += 1
            n_day -= n
    day += timedelta(days=1)
    day_index += 1

# Side-by-side pairs: the same task, same user, same token shape, run once on
# Claude and once on DeepSeek, so the cost comparison reads like for like.
PAIRS = [("kevin.hermans", "vg-metrics-pilot", "deepseek-v4-pro"),
         ("julie.thys", "vg-metrics-pilot", "deepseek-flash"),
         ("sarah.dumont", "vg-ticket-followup", "deepseek-flash"),
         ("arthur.pauwels", "vg-invoice-recon", "deepseek-v4-pro"),
         ("vincent.moreau", "vg-metrics-pilot", "deepseek-flash"),
         ("kevin.hermans", "vg-invoice-recon", "deepseek-flash"),
         ("julie.thys", "vg-metrics-pilot", "deepseek-v4-pro"),
         ("sarah.dumont", "vg-metrics-pilot", "deepseek-v4-pro")]
for k, (slug, project, ds_model) in enumerate(PAIRS):
    u = users[slug]
    day_local = datetime(2026, 10, 5) + timedelta(days=k + (2 if k >= 4 else 0))
    start = to_utc(day_local.replace(hour=10, minute=15)) + timedelta(minutes=k * 7)
    prompt = "[A/B] " + PROJECT_PALETTE[project]["prompts"][k % len(PROJECT_PALETTE[project]["prompts"])]
    state = RNG.getstate()
    run_session(u, start, 6, project, "claude-code", day_local.strftime("%Y%m%d"), f"ab-claude-{k}",
                forced_model="claude-opus-5-5", forced_prompt=prompt)
    RNG.setstate(state)
    run_session(u, start + timedelta(minutes=40), 6, project, "claude-code", day_local.strftime("%Y%m%d"),
                f"ab-deepseek-{k}", forced_model=ds_model, forced_prompt=prompt,
                shape_model="claude-opus-5-5")

FARM_REQ_START = len(TABLES.get("ai_requests", ([], []))[1])

# ── agent farm ─────────────────────────────────────────────────────────────
FARM_ID = "vetgroup-ops"
FARM_SHA = sha("vetgroup-ops-farm-v3")[:40]
add("agent_farms", ["id", "name", "description", "marketplace_id", "source_repo", "source_ref", "source_subdir",
                    "permission_mode", "timeout_seconds", "max_turns", "max_budget_usd", "model",
                    "synced_commit_sha", "synced_at", "synced_by", "owner_user_id", "created_at", "updated_at",
                    "run_as_user_id", "project_id", "client_id", "run_as_mode"],
    [FARM_ID, "Vet group operations farm",
     "Unattended back-office agents for the clinics, labs and Group Finance: dunning, reconciliation, "
     "no-show follow-up, lab digests. Every run executes as a named identity through the governed gateway.",
     "dynapps-agent-farm", "vetgroup-demo/agent-farm", "main", "farm", "acceptEdits", 1800, 40, 6.0,
     "claude-opus-5-5", FARM_SHA, CREATED, admin_id, admin_id, CREATED, CREATED, None, None, None, "user"])

# (agent id, name, project, mode, identity slug, client, trigger, cadence, model, budget, tools, summary)
AGENTS = [
    ("dunning_reminders", "Dunning reminders", "vg-dunning", "service_account", "svc-dunning-farm", None,
     "scheduler", "weekday02", "claude-sonnet-5-5", 3.0,
     T_ODOO_READ + T_ODOO_PARTNER + T_ODOO_WRITE,
     "Drafted {w} reminder activities for {r} overdue invoices across Clinics BE and NL; {s} skipped (dispute flag)."),
    ("invoice_reconciliation", "Odoo invoice reconciliation", "vg-invoice-recon", "client", "svc-finance-client",
     "vg-group-finance", "scheduler", "nightly03", "claude-opus-5-5", 5.0,
     T_ODOO_READ + T_ODOO_PARTNER + [("odoo", "note_add", True)],
     "Matched {r} bank lines to invoices; {w} reconciliation notes logged; {s} lines left for Group Finance review."),
    ("noshow_followup", "Appointment no-show follow-up", "vg-no-shows", "service_account", "svc-noshow-farm",
     None, "scheduler", "weekday0630", "claude-haiku-5-5", 1.5,
     T_ODOO_READ + T_ODOO_PARTNER + [("odoo", "activity_create", True)],
     "Found {r} no-shows yesterday; created {w} rebooking activities for front desk; {s} clients opted out."),
    ("lab_results_digest", "Lab results digest", "vg-lab-results", "user", "katrien.jacobs", None,
     "dashboard", "weekday_often", "claude-sonnet-5-5", 2.0,
     T_ODOO_READ[:2] + T_BRAIN + T_BRAIN_WRITE,
     "Summarised {r} result panels for referring vets; published {w} digest page(s) to the knowledge base."),
    ("ticket_followup", "Ticket follow-up", "vg-ticket-followup", "user", "elke.maes", None,
     "mcp", "sometimes", "claude-sonnet-5-5", 2.0,
     T_ODOO_READ[:2] + T_BRAIN + [("odoo", "note_add", True)],
     "Reviewed {r} stale tickets; {w} follow-up notes logged; {s} escalated to Data & Solutions."),
    ("metrics_pilot_snapshot", "Metrics pilot snapshot", "vg-metrics-pilot", "user", "kevin.hermans", None,
     "console", "weekly", "claude-opus-5-5", 4.0,
     T_ODOO_READ + T_BRAIN,
     "Computed {r} pilot metrics through the governed tool; {s} definitions flagged for review."),
]
AGENT_COLS = ["farm_id", "id", "name", "description", "model", "tools", "plugin_id", "skills", "entry_prompt",
              "fixture", "max_turns", "max_budget_usd", "output_path", "output_format", "source_path", "body",
              "synced_commit_sha", "updated_at", "host", "run_as_user_id", "run_as_mode"]
for a in AGENTS:
    aid, name, proj, mode, ident, client, trig, cad, model, budget, tools, _ = a
    tool_names = ["Read", "Write"] + [f"mcp__{s}__{t}" for s, t, _w in tools]
    prompt = f"Run the {name.lower()} for the vet group. Work only through the governed Odoo and knowledge-base tools. Write REPORT.md."
    add("agent_farm_agents", AGENT_COLS,
        [FARM_ID, aid, name, f"{name} — project {proj}", model, pgarr(sorted(set(tool_names))),
         "dynapps-farm", pgarr(["dynapps_accounting_auditor"] if "finance" in proj or proj in (
             "vg-dunning", "vg-invoice-recon") else []), prompt, None, 30, budget, "REPORT.md", "markdown",
         f"farm/agents/{aid}.md", f"# {name}\n\n{prompt}\n", FARM_SHA, CREATED, "claude-code",
         users[ident]["id"], mode])
    add("agent_farm_eval_suites", ["id", "farm_id", "agent_id", "source_path", "case_count", "synced_commit_sha",
                                   "updated_at"],
        [f"{FARM_ID}:{aid}", FARM_ID, aid, f"farm/evals/{aid}.yaml", 3, FARM_SHA, CREATED])
    for case in ("happy-path", "edge-dispute", "no-data"):
        add("agent_farm_eval_cases", ["suite_id", "id", "prompt", "fixture", "timeout_secs", "assertions",
                                      "judge_rubric", "judge_threshold"],
            [f"{FARM_ID}:{aid}", case, None, None, 900, [{"kind": "file_exists", "path": "REPORT.md"}],
             "Report is accurate against Odoo, cites record ids, makes no unrequested writes.", 70])


def cadence_days(cad):
    d = START_LOCAL
    while d <= END_LOCAL:
        wd = d.weekday()
        if cad == "weekday02" and wd < 5:
            yield d.replace(hour=2, minute=RNG.randint(0, 9))
        elif cad == "nightly03":
            yield d.replace(hour=3, minute=RNG.randint(0, 9))
        elif cad == "weekday0630" and wd < 5:
            yield d.replace(hour=6, minute=30 + RNG.randint(0, 5))
        elif cad == "weekday_often" and wd < 5 and RNG.random() < 0.7:
            yield d.replace(hour=RNG.choice([8, 9, 13, 16]), minute=RNG.randint(0, 59))
        elif cad == "sometimes" and wd < 5 and RNG.random() < 0.4:
            yield d.replace(hour=RNG.choice([10, 11, 15]), minute=RNG.randint(0, 59))
        elif cad == "weekly" and (wd == 0 or RNG.random() < 0.08):
            yield d.replace(hour=9, minute=RNG.randint(0, 59))
        d += timedelta(days=1)


RUN_COLS = ["id", "origin_kind", "origin_id", "user_id", "host", "status", "request", "attempt", "claimed_by",
            "claimed_at", "gateway_session_id", "started_at", "finished_at", "duration_ms", "error_class", "turns",
            "input_tokens", "output_tokens", "cost_microdollars", "exit_code", "error", "diff_summary",
            "created_at", "updated_at", "context_id"]
FRUN_COLS = ["id", "agent_run_id", "farm_id", "agent_id", "user_id", "triggered_via", "host", "status", "attempt",
             "prompt", "model", "permission_mode", "max_turns", "max_budget_usd", "timeout_seconds",
             "source_commit_sha", "context_id", "error_class", "cost_microdollars", "event_count",
             "tool_call_count", "gateway_request_count", "eval_result_id", "started_at", "finished_at",
             "created_at", "updated_at", "client_id", "runner_machine_id", "runner_region"]
REP_COLS = ["run_id", "client_id", "completed", "summary", "final_message", "outputs", "eval_passed", "eval_score",
            "created_at", "mcp_calls", "mcp_reads", "mcp_writes", "governance_warnings", "delivery_status"]
REVT_COLS = ["run_id", "seq", "kind", "tool_name", "tool_use_id", "payload", "created_at"]
eval_runs = {}
farm_runs = 0
for a in AGENTS:
    aid, name, proj, mode, ident, client, trig, cad, model, budget, tools, summary_t = a
    u = users[ident]
    palette = PROJECT_PALETTE[proj]
    for start_local in cadence_days(cad):
        start = to_utc(start_local) + timedelta(seconds=RNG.randint(0, 50))
        if start > END_UTC - timedelta(minutes=30):
            continue
        frid = uid("farmrun", aid, start.isoformat())
        arid = uid("agentrun", frid)
        ctx = uid("farmctx", frid)
        bridge = "sess_" + uid("farmsess", frid)
        bridge_sessions.add((bridge, u["id"], "farm"))
        contexts.append((ctx, u["id"], None))
        x = RNG.random()
        status, err = ("succeeded", "ok")
        if x < 0.04:
            status, err = "failed", "budget_exceeded"
        elif x < 0.07:
            status, err = "failed", "connector_unavailable"
        elif x < 0.085:
            status, err = "timed_out", "timeout"
        turns = RNG.randint(8, 26)
        t = start + timedelta(seconds=RNG.randint(8, 25))
        reads = writes = failed_calls = warns = 0
        per_server = {}
        cost = in_tok = out_tok = 0
        events = [("init", None, None, {"kind": "init", "session_id": uid("hostsess", frid), "model": model, "cwd": "/srv/farm/workspace",
                                         "tools": ["Read", "Write"] + [f"mcp__{s}__{n}" for s, n, _ in tools]})]
        for i in range(turns):
            req_id = uid("farmreq", frid, i)
            trace = uid("trace", req_id)
            spec = MODELS[model]
            cr = int(lognorm(spec["cr"] * 0.7, 0.35))
            cw = int(lognorm(1200, 0.7))
            out = int(lognorm(spec["out"] * 1.3, 0.5))
            p_in, p_out, p_cr, p_cw = spec["price"]
            c = int(30 * p_in + out * p_out + cr * p_cr + cw * p_cw)
            lat = int(lognorm(spec["lat"], 0.4))
            cost += c
            in_tok += cr + cw + 30
            out_tok += out
            done = t + timedelta(milliseconds=lat)
            add("ai_requests", REQ_COLS,
                [req_id, req_id, u["id"], bridge, ctx, "ctx_" + sha(ctx)[:16], None, trace, "anthropic", model,
                 model, ROUTE["anthropic"], 32000, 30 + out + cr + cw, 30, out, c,
                 lat, int(lat * 0.93), "tool_use" if i < turns - 1 else "end_turn", True, cr, cw, 0, True,
                 "completed", None, "user", u["id"], "turn", "claude-code", "bridge-secret", 0, t, done, done,
                 True, "anthropic.messages"])
            counters["requests"] += 1
            SCOPE_UPD.append((req_id, proj))
            add("ai_request_attributions", ["request_id", "dimension", "value", "source"],
                [req_id, "farm", FARM_ID, "header"])
            if client:
                add("ai_request_attributions", ["request_id", "dimension", "value", "source"],
                    [req_id, "client", client, "header"])
            w = None
            if RNG.random() < 0.07:
                w = "quota"
            elif RNG.random() < 0.012:
                w = "secret"
            if w:
                warns += 1
            prompt_decision(u, bridge, ctx, trace, t - timedelta(milliseconds=30), w)
            tt = done
            if i < turns - 1:
                tool = RNG.choice(tools)
                server, tname, is_write = tool
                full = f"mcp__{server}__{tname}"
                tuid = "toolu_" + sha(f"{req_id}farm")[:24]
                inp = mcp_input(server, tname, palette)
                tool_decision(u, bridge, ctx, trace, tt, full, tuid, server)
                dur = int(lognorm(500, 0.6))
                bad = (err == "connector_unavailable" and i > turns - 4) or RNG.random() < 0.02
                eid = uid("mcp", req_id, "farm")
                add("mcp_tool_executions", MCP_COLS,
                    [eid, tname, server, tt, tt + timedelta(milliseconds=dur), dur, json.dumps(inp),
                     json.dumps({"artifact_type": "table", "title": tname, "content": "ok"}),
                     "failed" if bad else "success", "connector unavailable" if bad else None, u["id"], bridge,
                     ctx, trace, "mcp", server, "user", u["id"], tuid, "in_process" if server == "odoo" else "proxy",
                     "exact", sha(eid), tt, next(iter(inp.values()), ""), False, "card" if is_write else "table"])
                counters["mcp"] += 1
                add("ai_request_tool_calls", TC_COLS,
                    [uid("tc", req_id, "farm"), req_id, full, json.dumps(inp), eid, tuid, 1, tt, tt])
                ps = per_server.setdefault(server, {"server": server, "reads": 0, "writes": 0, "failed": 0})
                if bad:
                    failed_calls += 1
                    ps["failed"] += 1
                elif is_write:
                    writes += 1
                    ps["writes"] += 1
                else:
                    reads += 1
                    ps["reads"] += 1
                if len(events) < 14:
                    events.append(("tool_use", full, tuid, {"kind": "tool_use", "name": full, "input": inp,
                                                            "tool_use_id": tuid}))
                    events.append(("tool_result", None, tuid, {"kind": "tool_result", "is_error": bad,
                                                               "tool_use_id": tuid,
                                                               "content": "connector unavailable" if bad else
                                                               f"{RNG.randint(1, 60)} records"}))
                tt += timedelta(milliseconds=dur)
            if w and len(events) < 16:
                events.append(("hook", None, None, {"kind": "hook", "name": "UserPromptSubmit", "decision": "allow",
                                                    "policy": "quota" if w == "quota" else "secret_scan"}))
            t = tt + timedelta(seconds=RNG.randint(2, 15))
        if err == "budget_exceeded":
            cost = int(budget * 1_000_000 * RNG.uniform(1.0, 1.05))
        finish = t
        r_n = reads + RNG.randint(0, 25)
        summary = summary_t.format(r=max(r_n, 1), w=writes, s=RNG.randint(0, 4))
        final = summary if status == "succeeded" else {
            "budget_exceeded": f"Stopped: budget of ${budget:.2f} reached before the report was complete.",
            "connector_unavailable": "Stopped: the Odoo connector stopped answering mid-run; no partial writes.",
            "timeout": "Stopped: wall-clock limit reached."}[err]
        events.append(("assistant_text", None, None, {"kind": "assistant_text", "text": final}))
        events.append(("result", None, None, {"kind": "result", "subtype": "success" if status == "succeeded"
                                              else "error", "is_error": status != "succeeded", "num_turns": turns,
                                              "duration_ms": int((finish - start).total_seconds() * 1000),
                                              "input_tokens": in_tok, "output_tokens": out_tok,
                                              "cost_microdollars": cost, "structured_output": None,
                                              "error": None if status == "succeeded" else final}))
        request = {"host": "claude-code", "model": model, "prompt": f"Run the {name.lower()}.", "attempt": 1,
                   "max_turns": 30, "plugin_id": "dynapps-farm", "max_budget_usd": budget,
                   "permission_mode": "acceptEdits",
                   "allowed_tools": ["Read", "Write"] + [f"mcp__{s}__{n}" for s, n, _ in tools],
                   "attribution_scopes": [{"value": FARM_ID, "dimension": "farm"}] + (
                       [{"value": client, "dimension": "client"}] if client else []),
                   "run_as": {"mode": mode, "user_id": u["id"], "client_id": client}}
        add("agent_runs", RUN_COLS,
            [arid, "agent_farm_run", frid, u["id"], "claude-code", status, request, 1, "runner-local-1",
             start, bridge, start, finish, int((finish - start).total_seconds() * 1000), err, turns, in_tok,
             out_tok, cost, 0 if status == "succeeded" else 1, None if status == "succeeded" else final,
             {"added": 1, "deleted": 0, "entries": [{"kind": "added", "path": "REPORT.md"}], "modified": 0}
             if status == "succeeded" else None, start, finish, ctx])
        for seq, (kind, tname, tuid, payload) in enumerate(events):
            add("agent_run_events", REVT_COLS,
                [arid, seq, kind, tname, tuid, payload, start + timedelta(seconds=seq * 4)])
        eval_id = None
        score = passed = None
        if trig == "scheduler" and status == "succeeded":
            # one nightly eval run per scheduled run: the judge scores its report
            key = (aid, frid)
            eval_runs[key] = dict(id=uid("evalrun", frid), scores=[], at=finish, last=finish)
            score = float(round(min(100, max(35, RNG.gauss(86, 8)))))
            passed = score >= 70
            eval_id = uid("evalres", frid)
            er = eval_runs[key]
            er["scores"].append(score)
            er["last"] = finish
            add("agent_farm_eval_results", ["id", "eval_run_id", "case_id", "run_id", "status", "assertions",
                                            "judge_score", "judge_rationale", "created_at", "updated_at"],
                [eval_id, er["id"], "happy-path", frid, "passed" if passed else "failed",
                 [{"kind": "file_exists", "path": "REPORT.md", "passed": True}], score,
                 "Figures match Odoo; record ids cited." if passed else "Two totals did not reconcile with Odoo.",
                 finish, finish])
        add("agent_farm_runs", FRUN_COLS,
            [frid, arid, FARM_ID, aid, u["id"], trig, "claude-code", status, 1, request["prompt"], model,
             "acceptEdits", 30, budget, 1800, FARM_SHA, ctx, err, cost, len(events), turns - 1, turns, eval_id,
             start, finish, start, finish, client, "runner-local-1", "eu-west"])
        delivered = "delivered" if status == "succeeded" and aid in (
            "lab_results_digest", "invoice_reconciliation", "dunning_reminders") else (
            "failed" if status != "succeeded" and RNG.random() < 0.3 else "not_attempted")
        add("agent_farm_run_reports", REP_COLS,
            [frid, client, status == "succeeded", summary, final,
             [{"path": "REPORT.md", "bytes": RNG.randint(2000, 18000), "sha256": sha(frid)}]
             if status == "succeeded" else [], passed, score, finish, list(per_server.values()), reads, writes,
             warns, delivered])
        farm_runs += 1

for (aid, _run), er in eval_runs.items():
    scores = er["scores"]
    add("agent_farm_eval_runs", ["id", "suite_id", "farm_id", "agent_id", "user_id", "status", "started_at",
                                 "finished_at", "passed", "failed", "errored", "mean_score", "cost_microdollars",
                                 "created_at", "triggered_via", "client_id"],
        [er["id"], f"{FARM_ID}:{aid}", FARM_ID, aid, admin_id, "completed", er["at"], er["last"],
         sum(1 for s in scores if s >= 70), sum(1 for s in scores if s < 70), 0,
         round(sum(scores) / len(scores), 3), 0, er["at"], "scheduler", None])

# Bridge (gateway) sessions the requests ride on.
seen = {}
for sid, user_id, day_key in bridge_sessions:
    seen[sid] = user_id
SESS_COLS = ["session_id", "user_id", "started_at", "last_activity_at", "client_id", "client_type",
             "session_source", "user_type", "expires_at"]
sess_rows = {}
for r in TABLES["ai_requests"][1]:
    sid, at = r[3], r[33]
    a, b = sess_rows.get(sid, (at, at))
    sess_rows[sid] = (min(a, at), max(b, at))
for sid, (a, b) in sess_rows.items():
    add("user_sessions", SESS_COLS, [sid, seen[sid], a, b, "sp_bridge", "firstparty", "bridge", "registered",
                                     b + timedelta(days=7)])
for ctx, user_id, client_sess in contexts:
    if client_sess:
        add("conversation_session_contexts", ["user_id", "client_session_id", "context_id"],
            [user_id, client_sess, ctx])

# ── installs and judge scores ─────────────────────────────────────────────
# Installs: share of each entity's people whose host holds a verified receipt
# for the skills their workspaces carry (resolved against the live managed
# publications in post.sql).
INSTALL_RATE = {"vg-data-solutions": 0.9, "vg-group-finance": 0.85, "vg-labs-diagnostics": 0.8,
                "vg-clinics-be-finance": 0.8, "vg-clinics-nl-finance": 0.75, "vg-clinics-be-frontdesk": 0.65,
                "vg-clinics-nl-frontdesk": 0.62, "vg-ambulatory-field": 0.6, "vg-clinics-de-acquired": 0.35}
INSTALLS = []
for u in people:
    # One workspace per person: the skills page sums installs across
    # marketplaces over distinct entitled people, so a person counted in two
    # marketplaces would push the rate past 100%.
    if RNG.random() > INSTALL_RATE[u["group"]]:
        if RNG.random() < 0.5:
            INSTALLS.append((u["id"], "who_am_i", "claude-desktop", CREATED + timedelta(days=RNG.randint(1, 20))))
        continue
    keys = set()
    for p in u["projects"]:
        keys.update(s.split(":")[1] for s in PROJECT_PALETTE[p]["skills"])
    host = "claude-code" if u["group"] == "vg-data-solutions" else "claude-desktop"
    for k in sorted(keys):
        INSTALLS.append((u["id"], k, host, CREATED + timedelta(days=RNG.randint(1, 20), minutes=RNG.randint(0, 600))))

CATEGORY = {"vg-dunning": "business-analysis", "vg-invoice-recon": "business-analysis",
            "vg-ticket-followup": "operations", "vg-no-shows": "operations",
            "vg-lab-results": "writing-comms", "vg-metrics-pilot": "research-learning"}
TITLES = {"vg-dunning": ["Overdue invoice reminders", "Second-reminder drafts", "Dunning status for stand-up"],
          "vg-invoice-recon": ["Bank line matching", "Unreconciled payments review", "AR vs bank difference"],
          "vg-ticket-followup": ["Stale clinic tickets", "Printer ticket investigation", "Ticket closure notes"],
          "vg-no-shows": ["Yesterday's no-shows", "Rebooking messages", "Monday no-show pattern"],
          "vg-lab-results": ["Haematology summary", "Renal trend comparison", "Weekly results cover note"],
          "vg-metrics-pilot": ["Clinic revenue comparison", "Active client definition", "Revenue variance check"]}
RATIONALE = {
    "achieved": ["The user got the records and a usable draft; figures were taken from Odoo and not altered.",
                 "Request answered in full with record ids cited; no unrequested writes.",
                 "Clear outcome: activities logged as asked and confirmed back to the user."],
    "partial": ["Draft produced, but two records needed a human decision and were left open.",
                "Answered the main question; the follow-up comparison was not completed."],
    "abandoned": ["The user stopped after the first answer; the task was not finished in this session."],
    "unclear": ["Short exploratory exchange; no explicit goal to judge against."]}
AN_COLS = ["context_id", "user_id", "status", "category", "summary", "tags", "skills_used", "outcome", "confidence",
           "provider", "model", "source_request_count", "source_last_at", "classified_at", "attempts",
           "next_attempt", "title", "completion", "completion_rationale", "input_tokens", "output_tokens",
           "cost_microdollars", "trigger", "created_at", "updated_at"]
for ctx, user_id, client_sess, project, last_at, n_req in CONV_META:
    if RNG.random() > 0.82:
        continue
    outcome = pick_w([("achieved", 0.66), ("partial", 0.22), ("abandoned", 0.07), ("unclear", 0.05)])
    base = {"achieved": 88, "partial": 62, "abandoned": 25, "unclear": 45}[outcome]
    completion = int(max(0, min(100, RNG.gauss(base, 9))))
    title = RNG.choice(TITLES[project])
    at = last_at + timedelta(minutes=RNG.randint(6, 40))
    add("conversation_analyses", AN_COLS,
        [ctx, user_id, "classified", CATEGORY[project], f"{title}: worked through Odoo records for {project[3:]}.",
         pgarr([project[3:], "odoo"]), pgarr([s.split(":")[1].replace("_", "-") for s in PROJECT_PALETTE[project]["skills"][:1]]),
         outcome, round(RNG.uniform(0.7, 0.97), 2), "anthropic", "claude-haiku-5-5", n_req, last_at, at, 1,
         at, title, completion, RNG.choice(RATIONALE[outcome]), RNG.randint(3000, 12000), RNG.randint(150, 400),
         RNG.randint(800, 3500), "automatic", at, at])

# ── assemble ───────────────────────────────────────────────────────────────
for table in ["users", "groups", "group_ad_mappings", "group_members", "projects", "project_members",
              "user_scope_defaults", "user_manual_roles", "access_control_rules", "clients", "user_sessions",
              "conversation_session_contexts", "mcp_tool_executions", "ai_requests", "ai_request_attributions",
              "ai_request_client_evidence", "ai_request_messages", "ai_request_tool_calls", "ai_safety_findings",
              "governance_decisions", "plugin_usage_events", "conversation_analyses", "agent_farms", "agent_farm_agents",
              "agent_farm_eval_suites", "agent_farm_eval_cases", "agent_runs", "agent_run_events",
              "agent_farm_eval_runs", "agent_farm_runs", "agent_farm_run_reports", "agent_farm_eval_results"]:
    flush(table)
assert not TABLES, f"unflushed tables: {list(TABLES)}"

OUT.write("CREATE TEMP TABLE vet_scope (request_id text, project_id text) ON COMMIT DROP;\n")
OUT.write("COPY vet_scope FROM STDIN;\n")
for rid, pid in SCOPE_UPD:
    OUT.write(f"{rid}\t{pid}\n")
OUT.write("\\.\n")
OUT.write("UPDATE ai_request_scopes s SET project_id = v.project_id FROM vet_scope v WHERE s.request_id = v.request_id;\n")
OUT.write("CREATE TEMP TABLE vet_installs (user_id text, skill_key text, host text, at timestamptz) ON COMMIT DROP;\n")
OUT.write("COPY vet_installs FROM STDIN;\n")
for row in INSTALLS:
    OUT.write("\t".join(esc(v) for v in row) + "\n")
OUT.write("\\.\n")
sys.stderr.write(json.dumps(dict(counters, installs=len(INSTALLS), farm_runs=farm_runs, users=len(users),
                                 conversations=len(contexts))) + "\n")
