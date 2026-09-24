"""Render every run's metrics.json for a plugin into one interactive HTML page.

    report.py <evals/plugin dir> <out.html>

The page embeds the data (no fetches), so it can be published as-is and
republished after each run to keep one URL for the plugin's history.
"""
import json
import pathlib
import sys

evals, out = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
runs = []
for m in sorted(evals.glob("*/metrics.json"), key=lambda p: json.load(open(p))["started_at"]):
    d = json.load(open(m))
    d.pop("skill_hashes", None)
    d.pop("delta", None)
    notes = m.parent / "notes.md"
    d["notes"] = notes.read_text() if notes.exists() else ""
    runs.append(d)
gateway = "https://sp-dev.astound.digital"
html = (pathlib.Path(__file__).parent / "report.template.html").read_text()
html = html.replace("__DATA__", json.dumps({"runs": runs, "gateway": gateway}).replace("</", "<\\/"))
out.write_text(html)
print(f"{out}: {len(runs)} runs")
