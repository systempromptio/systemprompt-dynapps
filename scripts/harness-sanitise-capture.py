#!/usr/bin/env python3
"""Turn raw `just smoke-harness-local` captures into harness fixtures.

Reads every <client>-NNN.json (and its <client>-NNN.reply, when the proxy kept
one) from a CAPTURE_DIR and writes tests/fixtures/harness format
({client, inbound, headers, body}) into the output directory, named
<client>-<model>-NNN.json / .sse. Sanitising: the per-run gateway session id is
dropped, the throwaway work dir becomes /workspace and the home dir becomes
/home/user. Credentials never reach a capture (the proxy strips them); a
leftover key-shaped string aborts the run instead of being written.

    harness-sanitise-capture.py <capture-dir> <out-dir>
"""
import json
import os
import re
import sys

SRC, OUT = sys.argv[1], sys.argv[2]
INBOUND = {
    "/v1/messages": "anthropic.messages",
    "/v1/chat/completions": "openai.chat",
    "/v1/responses": "openai.responses",
}
DROP_HEADERS = {"x-session-id"}
KEY = re.compile(r"sp-live-[0-9a-f]{6,}|sk-[A-Za-z0-9_-]{20,}|Bearer [A-Za-z0-9._-]{20,}")
HOME = os.path.expanduser("~")


def scrub(text):
    text = re.sub(r"/tmp/tmp\.[A-Za-z0-9]+(/project)?", "/workspace", text)
    text = re.sub(r"sess_[0-9a-f-]{36}", "sess_00000000-0000-0000-0000-000000000000", text)
    text = text.replace(HOME, "/home/user")
    if KEY.search(text):
        sys.exit(f"refusing to write: key-shaped string survived sanitising ({KEY.search(text).group()[:12]}...)")
    return text


os.makedirs(OUT, exist_ok=True)
for name in sorted(os.listdir(SRC)):
    if not name.endswith(".json"):
        continue
    raw = json.load(open(os.path.join(SRC, name)))
    body = raw.get("body") or {}
    stem = f"{raw['client']}-{body.get('model', 'unknown')}-{name[-8:-5]}"
    fixture = {
        "client": raw["client"],
        "inbound": INBOUND.get(raw["path"], raw["path"]),
        "headers": {k: v for k, v in raw["headers"].items() if k not in DROP_HEADERS},
        "body": body,
    }
    with open(os.path.join(OUT, stem + ".json"), "w") as f:
        f.write(scrub(json.dumps(fixture, indent=1)) + "\n")
    reply = os.path.join(SRC, name[:-5] + ".reply")
    if os.path.exists(reply):
        with open(os.path.join(OUT, stem + ".sse"), "w") as f:
            f.write(scrub(open(reply, encoding="utf-8", errors="replace").read()))
    print(stem)
