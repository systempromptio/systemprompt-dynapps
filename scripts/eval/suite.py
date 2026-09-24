"""Suite plumbing for eval.sh: prompts, the run manifest, and the readiness check.

The manifest pins what a run measured — the prompt-set hash, the skill content
hashes and the plugin version — so two runs are only compared when they asked
the same questions.
"""
import datetime
import hashlib
import json
import os
import pathlib
import sys

import yaml

ROOT = pathlib.Path(__file__).resolve().parents[2]


def load(path):
    return yaml.safe_load(open(path, encoding="utf-8"))


def now():
    return datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def skill_hash(skill):
    h = hashlib.sha256()
    for f in sorted((ROOT / "services/skills" / skill).rglob("*")):
        if f.is_file():
            h.update(f.name.encode())
            h.update(f.read_bytes())
    return h.hexdigest()[:12]


def bridge_patch(plugin):
    win = pathlib.Path(os.environ.get("WIN_HOME", "/mnt/c/Users/ejb50"))
    stamp = win / f".claude/plugins/cache/{plugin}/{plugin}/current/.eval-patch"
    return json.loads(stamp.read_text()) if stamp.exists() else None


def prompt(suite, skill):
    dashed = skill.replace("_", "-")
    return f"/{suite['plugin']}:{dashed} {suite['skills'][skill]}\n\n{suite['suffix']}"


def manifest_start(path, label):
    suite = load(path)
    plugin = load(ROOT / f"services/plugins/{suite['plugin']}/config.yaml")["plugin"]
    prompts_hash = hashlib.sha256(open(path, "rb").read()).hexdigest()[:12]
    return {
        "label": label,
        "plugin": suite["plugin"],
        "plugin_version": plugin.get("version"),
        "suite_version": suite["suite_version"],
        "prompts_hash": prompts_hash,
        "model": suite["model"],
        "started_at": now(),
        "skills": {s: skill_hash(s) for s in suite["skills"]},
        # Why: a run on skills patched into the client ahead of a deploy is
        # served by the old marketplace version; the console credits it there.
        "bridge_patch": bridge_patch(suite["plugin"]),
        "runs": {},
    }


def manifest_finish(path, raw):
    m = json.load(open(path))
    raw = pathlib.Path(raw)
    attempts = dict(l.split() for l in open(raw / "attempts.txt") if l.strip())
    for skill in m["skills"]:
        f = raw / "results" / f"{skill}.json"
        try:
            r = json.load(open(f, encoding="utf-8-sig"))
        except (OSError, ValueError):
            m["runs"][skill] = {"session_id": None, "status": "no_result"}
            continue
        m["runs"][skill] = {
            "session_id": r.get("session_id"),
            "status": "error" if r.get("is_error") else "ok",
            "attempts": int(attempts.get(skill, 1)),
        }
    m["finished_at"] = now()
    json.dump(m, open(path, "w"), indent=2)


def had_mcp(result, cc_projects):
    try:
        sid = json.load(open(result, encoding="utf-8-sig"))["session_id"]
        text = open(pathlib.Path(cc_projects) / f"{sid}.jsonl", encoding="utf-8").read()
    except (OSError, ValueError, KeyError):
        return False
    return '"name":"mcp__' in text


if __name__ == "__main__":
    op, *args = sys.argv[1:]
    if op == "skills":
        print("\n".join(load(args[0])["skills"]))
    elif op == "model":
        print(load(args[0])["model"])
    elif op == "prompt":
        print(prompt(load(args[0]), args[1]))
    elif op == "manifest-start":
        print(json.dumps(manifest_start(*args), indent=2))
    elif op == "manifest-finish":
        manifest_finish(*args)
    elif op == "had-mcp":
        sys.exit(0 if had_mcp(*args) else 1)
