"""Install this tree's plugin skills straight into the Windows Claude Code plugin
cache, ahead of a marketplace deploy, so an eval can run the next version today.

    patch-bridge.py <plugin> [--win-home /mnt/c/Users/<you>] [--restore]

Writes each skill the plugin includes, rendered the way the bridge delivers
it (frontmatter name/description, a blank line, then SKILL.md, plus any
sibling files), into both the active cache and the marketplace copy, and
removes skills the plugin no longer includes. The first patch backs the
cache up to `current.pre-patch`; --restore puts it back. The next bridge sync
overwrites a patch, so check `.eval-patch` before a run.

The gateway still credits these runs to the marketplace version it serves:
the console's Versions page moves only when the new marketplace is deployed.
"""
import hashlib
import json
import pathlib
import shutil
import sys

import yaml

ROOT = pathlib.Path(__file__).resolve().parents[2]
args = sys.argv[1:]
plugin_id = args[0]
win = pathlib.Path(args[args.index("--win-home") + 1]) if "--win-home" in args else pathlib.Path("/mnt/c/Users/ejb50")
cache = win / f".claude/plugins/cache/{plugin_id}/{plugin_id}"
current, backup = cache / "current", cache / "current.pre-patch"
market = win / f".claude/plugins/marketplaces/{plugin_id}/plugins/{plugin_id}"

if "--restore" in args:
    if not backup.exists():
        sys.exit("no backup to restore")
    shutil.rmtree(current)
    backup.rename(current)
    print("restored", current)
    sys.exit(0)

plugin = yaml.safe_load(open(ROOT / f"services/plugins/{plugin_id}/config.yaml"))["plugin"]
skills = plugin["skills"]["include"]
if not backup.exists():
    shutil.copytree(current, backup)

digest = hashlib.sha256()
for target in (current, market):
    sk_dir = target / "skills"
    wanted = {s.replace("_", "-") for s in skills}
    for existing in sk_dir.iterdir():
        if existing.is_dir() and existing.name not in wanted:
            shutil.rmtree(existing)
    for s in skills:
        src = ROOT / "services/skills" / s
        cfg = yaml.safe_load(open(src / "config.yaml"))
        body = (src / cfg.get("file", "SKILL.md")).read_text()
        rendered = f"---\nname: {s.replace('_', '-')}\ndescription: {json.dumps(cfg['description'].strip(), ensure_ascii=False)}\n---\n\n{body}"
        dst = sk_dir / s.replace("_", "-")
        dst.mkdir(parents=True, exist_ok=True)
        (dst / "SKILL.md").write_text(rendered, encoding="utf-8")
        for extra in src.iterdir():
            if extra.name not in ("config.yaml", cfg.get("file", "SKILL.md")) and extra.is_file():
                shutil.copy(extra, dst / extra.name)
        if target == current:
            digest.update(rendered.encode())
    pj = target / ".claude-plugin/plugin.json"
    if pj.exists():
        meta = json.loads(pj.read_text())
        meta["version"] = f"{plugin['version']}+local"
        meta["description"] = plugin["description"]
        pj.write_text(json.dumps(meta, indent=2))

stamp = {"plugin_version": plugin["version"], "skills": len(skills), "content": digest.hexdigest()[:12]}
(current / ".eval-patch").write_text(json.dumps(stamp))
print(json.dumps(stamp))
