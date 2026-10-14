#!/usr/bin/env bash
# Fake 1Password CLI for the secrets-resolver tests (tests/common/src/onepassword_fake_cli.rs).
#
# Answers from the JSON store at $FAKE_OP_STORE:
#   {"token": "...", "fail": null | {"exit_code": N, "stderr": "..."},
#    "vaults": [{"id","name","items": [{"id","title","vault":{"id"},"fields":[{"id","label","type","value"}]}]}]}
# Supported: op --version | op whoami | op read [-n|--no-newline] op://vault/item/field
#            op item get <item> --vault <vault> [--format json]
# Vaults and items match by name/title or id. Needs bash and python3 (Linux and macOS).
set -euo pipefail
if [ -z "${FAKE_OP_STORE:-}" ] || [ ! -f "$FAKE_OP_STORE" ]; then
  echo "[ERROR] fake op: FAKE_OP_STORE is not set to a store file" >&2
  exit 1
fi
exec python3 -I -c '
import json, os, sys
store = json.load(open(os.environ["FAKE_OP_STORE"]))
args = sys.argv[1:]
def die(msg, code=1):
    sys.stderr.write("[ERROR] " + msg + "\n"); sys.exit(code)
if args[:1] == ["--version"]:
    print("2.30.0"); sys.exit(0)
fail = store.get("fail")
if fail:
    sys.stderr.write(fail.get("stderr", "")); sys.exit(int(fail.get("exit_code", 1)))
if os.environ.get("OP_SERVICE_ACCOUNT_TOKEN", "") != store.get("token"):
    die("authorization error: invalid service account token")
def vault(ref):
    for v in store.get("vaults", []):
        if ref in (v["name"], v["id"]): return v
    die("\"%s\" isn'"'"'t a vault in this account." % ref)
def item(v, ref):
    for i in v["items"]:
        if ref in (i["title"], i["id"]): return i
    die("\"%s\" isn'"'"'t an item in the \"%s\" vault." % (ref, v["name"]))
if args[:1] == ["whoami"]:
    print(json.dumps({"user_type": "SERVICE_ACCOUNT"})); sys.exit(0)
if args[:1] == ["read"]:
    rest = [a for a in args[1:] if a not in ("-n", "--no-newline")]
    newline = len(rest) == len(args) - 1
    if len(rest) != 1 or not rest[0].startswith("op://"):
        die("usage: op read op://vault/item/field")
    parts = rest[0][len("op://"):].split("/")
    if len(parts) != 3: die("invalid secret reference \"%s\"" % rest[0])
    v = vault(parts[0]); i = item(v, parts[1])
    for f in i["fields"]:
        if parts[2] in (f["label"], f["id"]):
            sys.stdout.write(f["value"] + ("\n" if newline else "")); sys.exit(0)
    die("could not find field \"%s\" on item \"%s\"" % (parts[2], i["title"]))
if args[:2] == ["item", "get"] and len(args) >= 3:
    rest = args[3:]
    if "--vault" not in rest or rest.index("--vault") + 1 >= len(rest):
        die("--vault is required by the fake")
    v = vault(rest[rest.index("--vault") + 1])
    print(json.dumps(item(v, args[2]))); sys.exit(0)
die("fake op: unsupported command: " + " ".join(args))
' "$@"
