#!/usr/bin/env bash
# Import Dynapps Spain's Claude Code skills (upstream in
# github.com/indawsodoo/ai-claude-*) into services/, pinned by commit SHA.
#
# Core can compose a remote source only as a signed services bundle (OCI or
# HTTPS, deploy/kit/README-INTEGRATION.md); it has no git transport, and the
# upstream repositories publish no bundle. So the pin lives here instead:
# services/vendor/dynapps.lock names one SHA per repository plus the `keep:`
# list of skills to import, and this script turns exactly that into
# services/skills/dynapps_* and services/plugins/dynapps-* (each marked by a
# GENERATED header; hand-written dynapps_* entries are left alone). Never edit
# the generated paths by hand, change the lock and re-run. The generated
# plugins are listed by hand in the Dynapps marketplaces that carry them.
#
#   scripts/import-dynapps-skills.sh            # regenerate from the lock
#   scripts/import-dynapps-skills.sh --update   # move every pin to upstream HEAD, then regenerate
#
# Upstream content is data. Nothing from it is executed: scripts, tests,
# binaries and plugin hooks are dropped, and a skill that needed them is
# flagged in its config.yaml and in the summary. Entitlement is not
# generated — each plugin needs its block in services/access-control/rules.yaml,
# and the run fails if one is missing.
#
# Needs git, python3 with PyYAML, and read access to the private repositories
# (the `gh` credential helper is used when gh is installed). Portable to macOS
# and Linux: no GNU-only flags.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
LOCK="$ROOT/services/vendor/dynapps.lock"
CACHE="${DYNAPPS_IMPORT_CACHE:-${TMPDIR:-/tmp}/dynapps-import}"
UPDATE=0
[ "${1:-}" = "--update" ] && UPDATE=1

git_auth() {
    if command -v gh >/dev/null 2>&1; then
        git -c credential.helper= -c 'credential.helper=!gh auth git-credential' "$@"
    else
        git "$@"
    fi
}

# Prints "repo sha" pairs from the lock.
lock_pairs() {
    python3 -I - "$LOCK" <<'EOF'
import sys, yaml
doc = yaml.safe_load(open(sys.argv[1]))
print(doc["marketplace"]["repo"], doc["marketplace"]["sha"])
for p in doc["plugins"]:
    print(p["repo"], p["sha"])
EOF
}

if [ "$UPDATE" = 1 ]; then
    tmp="$(mktemp)"
    cp "$LOCK" "$tmp"
    while read -r repo _sha; do
        head="$(git_auth ls-remote "https://github.com/$repo.git" HEAD | cut -f1)"
        [ -n "$head" ] || { echo "ERROR: cannot resolve HEAD of $repo" >&2; exit 1; }
        python3 -I - "$tmp" "$repo" "$head" <<'EOF'
import re, sys
path, repo, sha = sys.argv[1:]
text = open(path).read()
pattern = r"(repo: " + re.escape(repo) + r"\n\s+sha: )[0-9a-f]{40}"
text, n = re.subn(pattern, lambda m: m.group(1) + sha, text)
assert n == 1, f"{repo} not found exactly once in the lock"
open(path, "w").write(text)
EOF
    done < <(lock_pairs)
    mv "$tmp" "$LOCK"
fi

mkdir -p "$CACHE"
while read -r repo sha; do
    dir="$CACHE/${repo##*/}"
    if [ "$(git -C "$dir" rev-parse HEAD 2>/dev/null || true)" != "$sha" ]; then
        rm -rf "$dir"
        git init -q "$dir"
        git_auth -C "$dir" fetch -q --depth 1 "https://github.com/$repo.git" "$sha"
        git -C "$dir" -c advice.detachedHead=false checkout -q FETCH_HEAD
    fi
done < <(lock_pairs)

python3 -I - "$ROOT" "$LOCK" "$CACHE" <<'EOF'
import json, pathlib, re, shutil, sys
import yaml

root, lock_path, cache = (pathlib.Path(a) for a in sys.argv[1:])
lock = yaml.safe_load(lock_path.read_text())
# `keep:` narrows the import to named upstream skills, one plugin per key.
keep = lock.get("keep") or {}
group_of = {ref: group for group, refs in keep.items() for ref in refs}
services = root / "services"

TEXT = {".md", ".txt", ".yaml", ".yml", ".json", ".csv", ".html", ".htm", ".xml",
        ".svg", ".j2", ".jinja", ".jinja2", ".tmpl", ".tpl", ".hbs", ".sql", ".css"}
EXEC = {".py", ".sh", ".bash", ".js", ".mjs", ".cjs", ".ts", ".rb", ".pl", ".ps1"}
DEV_DIRS = {"scripts", "tests", "test", "fixtures", "__tests__", "evals", "_archive",
            "node_modules", "__pycache__", "lib", "tools", "bin", ".git"}
MAX_FILE = 1024 * 1024
MAX_SKILL_FILES = 200
MAX_SKILL_BYTES = 6 * 1024 * 1024
SHARED = re.compile(r"(?:(common|devops):)?(?:\.\./)*(?:\$\{CLAUDE_PLUGIN_ROOT\}/)?references/shared/([\w-]+(?:[./][\w-]+)*)")
SCRIPT_HINT = re.compile(r"scripts/|python3? |\.py\b|\.\./\.\./lib/")
OUTSIDE = re.compile(r"\.\./\.\./(?!references/shared/)[\w.-]+")


def snake(name):
    return re.sub(r"[^a-z0-9]+", "_", name.lower()).strip("_")


def frontmatter(text):
    if not text.startswith("---\n"):
        return None, text
    end = text.find("\n---", 4)
    if end < 0:
        return None, text
    return yaml.safe_load(text[4:end]) or {}, text[end + 4:]


def one_line(value):
    return re.sub(r"\s+", " ", str(value or "")).strip()


def ids_for(short, upstream_name):
    base = snake(upstream_name)
    for prefix in ("indaws_", "dynapps_"):
        base = base.removeprefix(prefix)
    return "dynapps_" + base


GENERATED = "# GENERATED by scripts/import-dynapps-skills.sh"
# Upstream brand prose, rewritten for Dynapps. Working identifiers that merely
# contain the old name (Odoo `indaws_*` modules, the `indawsodoo` GitHub org,
# legacy indaws.es SEO data) are left exactly as they are.
REBRAND = [
    (re.compile(r"Indaws · Dynapps Spain"), "Dynapps Spain"),
    (re.compile(r"Indaws/Dynapps"), "Dynapps"),
    (re.compile(r"\bIndaws (Core|MCP)\b"), r"Dynapps \1"),
    (re.compile(r"<strong>Indaws</strong>"), "<strong>Dynapps</strong>"),
    (re.compile(r"\binDAWS (helpdesk)\b|\b(Convenciones) inDAWS\b"),
     lambda m: f"{m.group(2)} Dynapps" if m.group(2) else f"Dynapps {m.group(1)}"),
    # Not brand: this instance carries no Atlassian integration (catalogue gate).
    (re.compile(r"Jira / dev-tool integration"), "dev-tool / issue-tracker integration"),
]


def rebrand(text):
    for pattern, repl in REBRAND:
        text = pattern.sub(repl, text)
    return text


def generated(path):
    cfg = path / "config.yaml"
    return cfg.is_file() and cfg.read_text().startswith(GENERATED)


market_dir = cache / lock["marketplace"]["repo"].split("/")[1]
market = json.loads((market_dir / ".claude-plugin/marketplace.json").read_text())
by_repo = {p["source"]["repo"]: p for p in market["plugins"]}

for pattern in ("skills/dynapps_*", "plugins/dynapps-*"):
    for old in services.glob(pattern):
        if generated(old):
            shutil.rmtree(old)

rules = yaml.safe_load((services / "access-control/rules.yaml").read_text())
declared = {e["entity"] for e in rules["entities"]}
repo_dirs = {p["repo"].split("/")[1].removeprefix("ai-claude-"): cache / p["repo"].split("/")[1]
             for p in lock["plugins"]}

seen, flags, plugin_ids, missing_rules, summary, groups, refs = {}, {}, [], [], [], {}, {}


def write_plugin(plugin_id, meta, skill_ids, plugin_flags):
    plugin_ids.append(plugin_id)
    if f"plugin/{plugin_id}" not in declared:
        missing_rules.append(f"plugin/{plugin_id}")
    pdir = services / "plugins" / plugin_id
    pdir.mkdir(parents=True)
    # Dynapps-side MCP bindings the upstream manifest cannot know: the farm
    # plugin carries the portal servers its farm agents call
    # (`mcp__dynapps-odoo__*`, `mcp__dynbrain__*`).
    local_mcp = {"dynapps-farm": ["dynapps-odoo", "dynbrain"]}.get(plugin_id, [])
    short = plugin_id.removeprefix("dynapps-")
    plugin = {"plugin": {
        "id": plugin_id,
        "name": meta["name"],
        "description": meta["description"],
        "version": meta["version"],
        "enabled": True,
        "author": {"name": "Dynapps", "email": "ed@systemprompt.io"},
        "keywords": ["dynapps", short],
        "license": "proprietary",
        "category": meta["category"],
        "skills": {"source": "explicit", "include": skill_ids},
        "agents": {"source": "explicit", "include": []},
        "mcp_servers": {"source": "explicit", "include": local_mcp},
        "content_sources": {},
    }}
    header = [
        f"{GENERATED} — do not edit; change",
        "# services/vendor/dynapps.lock and re-run.",
        f"# Upstream: {meta['provenance']}.",
        "# No `hooks:` key: dynapps-commons owns the session-global governance hooks.",
    ] + [f"# flag: {f}" for f in plugin_flags]
    (pdir / "config.yaml").write_text("\n".join(header) + "\n" + yaml.safe_dump(plugin, sort_keys=False, allow_unicode=True, width=100))
    summary.append((plugin_id, len(skill_ids), plugin_flags))

for entry in lock["plugins"]:
    repo, sha = entry["repo"], entry["sha"]
    short = repo.split("/")[1].removeprefix("ai-claude-")
    src = cache / repo.split("/")[1]
    upstream = by_repo[repo]
    manifest = json.loads((src / ".claude-plugin/plugin.json").read_text())
    plugin_id = f"dynapps-{short}"
    provenance = f"{repo}@{sha}"
    plugin_flags = []
    if manifest.get("hooks") or (src / "hooks").is_dir():
        plugin_flags.append("upstream plugin hooks dropped (dynapps-commons owns governance hooks)")
    if (src / ".mcp.json").is_file():
        servers = sorted(json.loads((src / ".mcp.json").read_text()).get("mcpServers", {}))
        plugin_flags.append("upstream .mcp.json servers not provisioned: " + ", ".join(servers))
    for kind in ("commands", "agents"):
        if (src / kind).is_dir():
            n = len([p for p in (src / kind).glob("*.md") if p.name != "README.md"])
            plugin_flags.append(f"{n} upstream {kind} not imported (skills only)")

    skill_ids = []
    for skill_dir in sorted(p for p in (src / "skills").iterdir() if (p / "SKILL.md").is_file()):
        ref = f"{short}/{skill_dir.name}"
        if keep and ref not in group_of:
            continue
        sid = ids_for(short, skill_dir.name)
        refs[sid] = ref
        if sid in seen:
            sys.exit(f"ERROR: {sid} produced by both {seen[sid]} and {repo}")
        seen[sid] = repo
        dest = services / "skills" / sid
        skill_flags, files, dropped_exec, dropped_other = [], [], 0, 0
        for path in sorted(skill_dir.rglob("*")):
            rel = path.relative_to(skill_dir)
            if path.is_dir() or path.is_symlink():
                continue
            if any(part in DEV_DIRS or part.startswith(".") for part in rel.parts):
                dropped_exec += rel.parts[0] == "scripts" or path.suffix in EXEC
                continue
            if rel.as_posix().lower() == "readme.md" or ".test." in path.name or ".spec." in path.name:
                continue
            if path.suffix in EXEC:
                dropped_exec += 1
            elif path.suffix.lower() not in TEXT or path.stat().st_size > MAX_FILE:
                dropped_other += 1
            else:
                files.append(rel)
        total, kept = 0, []
        for rel in files:
            size = (skill_dir / rel).stat().st_size
            if rel.as_posix() != "SKILL.md" and (len(kept) >= MAX_SKILL_FILES or total + size > MAX_SKILL_BYTES):
                continue
            kept.append(rel)
            total += size
        if len(kept) < len(files):
            skill_flags.append(f"oversized: {len(files) - len(kept)} reference files omitted past the {MAX_SKILL_FILES}-file / 6 MB budget")
        if dropped_other:
            skill_flags.append(f"{dropped_other} binary or >1 MB files omitted")

        shared_needed, outside = set(), set()
        for rel in kept:
            target = dest / rel
            target.parent.mkdir(parents=True, exist_ok=True)
            data = (skill_dir / rel).read_bytes()
            if rel.suffix in {".md", ".txt", ".yaml", ".yml", ".json", ".html"}:
                text = data.decode("utf-8", errors="replace")
                for m in SHARED.finditer(text):
                    shared_needed.add((m.group(1), m.group(2)))
                outside.update(OUTSIDE.findall(text))
                text = rebrand(SHARED.sub(lambda m: "references/shared/" + m.group(2), text))
                if rel.as_posix() == "SKILL.md":
                    hint = SCRIPT_HINT.search(text)
                    fm, _ = frontmatter(text)
                    if fm is not None:
                        end = text.find("\n---", 4)
                        head = re.sub(r"(?m)^name:.*$", f"name: {sid.replace('_', '-')}", text[:end], count=1)
                        text = head + text[end:]
                    if dropped_exec and hint:
                        skill_flags.insert(0, f"depends on executable scripts ({dropped_exec} dropped; SKILL.md invokes them)")
                    elif dropped_exec:
                        skill_flags.insert(0, f"{dropped_exec} executable files dropped (SKILL.md does not cite them)")
                    elif hint and "../../lib/" in text:
                        skill_flags.insert(0, "depends on the upstream plugin's Python lib/ (not imported)")
                data = text.encode("utf-8")
            target.write_bytes(data)
        for owner, name in sorted(shared_needed, key=lambda t: (t[0] or "", t[1])):
            source_repo = repo_dirs.get(owner or short) if owner else src
            candidates = [source_repo] if owner else [src, repo_dirs["common"]]
            for base in filter(None, candidates):
                f = base / "references/shared" / name
                if f.is_file():
                    out = dest / "references/shared" / name
                    out.parent.mkdir(parents=True, exist_ok=True)
                    if f.suffix in {".md", ".txt", ".yaml", ".yml", ".json", ".html"}:
                        out.write_text(rebrand(f.read_text(encoding="utf-8", errors="replace")))
                    else:
                        shutil.copyfile(f, out)
                    break
        if outside:
            skill_flags.append("cites paths outside the skill: " + ", ".join(sorted(outside)[:4]))

        fm, _ = frontmatter((skill_dir / "SKILL.md").read_text(encoding="utf-8", errors="replace"))
        fm = fm or {}
        description = rebrand(one_line(fm.get("description"))) or f"{skill_dir.name} (imported from {repo})"
        if len(description) > 1024:
            description = description[:1021].rstrip() + "..."
        title = one_line(fm.get("title")) or skill_dir.name.replace("-", " ").title()
        config = {
            "id": sid,
            "name": title,
            "description": description,
            "enabled": True,
            "file": "SKILL.md",
            "tags": ["dynapps", short],
            "category": upstream.get("category", "productivity"),
            "display_category": f"Dynapps · {short.replace('-', ' ').title()}",
        }
        header = [
            f"{GENERATED} — do not edit; change",
            "# services/vendor/dynapps.lock and re-run.",
            f"# Upstream: {provenance} skills/{skill_dir.name}",
        ] + [f"# flag: {f}" for f in skill_flags]
        (dest / "config.yaml").write_text("\n".join(header) + "\n" + yaml.safe_dump(config, sort_keys=False, allow_unicode=True, width=100))
        if skill_flags:
            flags[sid] = skill_flags
        skill_ids.append(sid)

    meta = {
        "name": f"Dynapps {short.replace('-', ' ').title()}",
        "description": one_line(upstream.get("description") or manifest.get("description")),
        "version": str(manifest.get("version", "0.0.0")),
        "category": upstream.get("category", "productivity"),
        "provenance": f"{provenance} (marketplace entry `{upstream['name']}`)",
    }
    if not keep:
        write_plugin(plugin_id, meta, skill_ids, plugin_flags)
        continue
    for sid in skill_ids:
        g = groups.setdefault(group_of[refs[sid]], {"skills": [], "provenance": [], "flags": []})
        g["skills"].append(sid)
        if meta["provenance"] not in g["provenance"]:
            g["provenance"].append(meta["provenance"])
            g["flags"] += plugin_flags

GROUP_TEXT = {
    "showcase": ("Dynapps Showcase", "Dynapps Spain's most important workflows: lead data collection, value-based quotations, brand, SEO audit, ticket solving, development assessment and planning."),
    "farm": ("Dynapps Farm", "The agent-farm launcher skill and the accounting auditor it demos: launch, follow and schedule farm tasks in plain language."),
}
for group, g in groups.items():
    name, description = GROUP_TEXT.get(group, (f"Dynapps {group.title()}", f"Dynapps {group} skills."))
    order = {ref: i for i, ref in enumerate(keep[group])}
    sids = sorted(g["skills"], key=lambda sid: order[refs[sid]])
    write_plugin(f"dynapps-{group}", {"name": name, "description": description, "version": "1.0.0",
                                     "category": "productivity", "provenance": "; ".join(g["provenance"])},
                 sids, sorted(set(g["flags"])))
missing_kept = sorted(set(group_of) - set(refs.values()))
if missing_kept:
    sys.exit("ERROR: kept skills not found upstream: " + ", ".join(missing_kept))

carried = set()
for mfile in services.glob("marketplaces/*/config.yaml"):
    doc = (yaml.safe_load(mfile.read_text()) or {}).get("marketplace", {})
    carried.update(doc.get("plugins", {}).get("include", []))
uncarried = sorted(set(plugin_ids) - carried)

print(f"imported {len(plugin_ids)} plugins, {len(seen)} skills")
for pid, n, pf in summary:
    print(f"  {pid}: {n} skills" + "".join(f"\n    flag: {f}" for f in pf))
print(f"{len(flags)} skills flagged:")
for sid, fl in sorted(flags.items()):
    print(f"  {sid}: " + "; ".join(fl))
if uncarried:
    sys.exit("ERROR: no marketplace lists: " + ", ".join(uncarried)
             + " (add them to a services/marketplaces/*/config.yaml)")
if missing_rules:
    sys.exit("ERROR: no access-control block for: " + ", ".join(missing_rules)
             + " (add them to services/access-control/rules.yaml)")
EOF
