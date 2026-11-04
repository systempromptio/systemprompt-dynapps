#!/usr/bin/env python3
"""Vendor one npm package into storage/files/js/vendor/ and record it.

    scripts/vendor-add.py <name> <version> --why "<reason>" [--entry <subpath>]
                          [--external <pkg>[,<pkg>…]] [--owner <team>]

The only sanctioned way third-party frontend code enters the repo (see the
header of services/web/vendor.yaml). Steps:

  1. `npm install --ignore-scripts` of exactly <name>@<version> into a fresh
     temporary project; npm verifies every tarball against the registry's
     integrity, and the manifest records the top-level one.
  2. esbuild (pinned below) bundles the package's ESM entry into one module.
     Every package already in the manifest stays an external bare specifier,
     resolved by the import map; anything else the bundle pulls in is inlined
     and recorded under `bundled` with its own version, licence and integrity.
  3. Files are written under vendor/<name>@<version>/ with a content hash in
     the name (so the static handler serves them `immutable`), and their
     sha256 + sha384 SRI land in the manifest.
  4. `approved_by` / `approved_on` are cleared: the diff needs review.
  5. extensions/web/shared/admin-core/src/vendor/assets.rs is regenerated.

Run through `just vendor-add`; `just vendor-check` validates the result.
"""
import argparse
import base64
import hashlib
import json
import pathlib
import shutil
import subprocess
import sys
import tempfile

import yaml

ROOT = pathlib.Path(__file__).resolve().parent.parent
MANIFEST = ROOT / "services/web/vendor.yaml"
VENDOR_DIR = ROOT / "storage/files/js/vendor"
ASSETS_RS = ROOT / "extensions/web/shared/admin-core/src/vendor/assets.rs"
ESBUILD = "esbuild@0.28.2"
WASM_SUFFIX = ".wasm"

HEADER = """\
# The third-party frontend allowlist: the ONLY way JS, WASM, CSS or fonts
# written outside this repo reach a page.
#
# Every file under storage/files/js/vendor/ is listed here with its sha256
# and sha384 SRI, and every listed file exists. Pages that opt into the
# workspace layout get an import map generated from this file; no other page
# loads vendor code. `scripts/check-vendor.sh` and
# tests/unit/web/src/vendor_allowlist.rs enforce it.
#
# Add or upgrade only with `just vendor-add <name> <version> --why "…"`,
# never by hand. The tool clears approved_by/approved_on; a reviewer fills
# them in the same PR. systemprompt-core never ships third-party frontend
# code: this file and the vendor/ directory exist only in astound.
"""


def die(msg):
    print(f"vendor-add: {msg}", file=sys.stderr)
    sys.exit(1)


def run(cmd, cwd):
    proc = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True)
    if proc.returncode != 0:
        die(f"{' '.join(cmd)} failed:\n{proc.stdout}{proc.stderr}")
    return proc.stdout


def digests(data):
    sha256 = hashlib.sha256(data).hexdigest()
    sri = "sha384-" + base64.b64encode(hashlib.sha384(data).digest()).decode()
    return sha256, sri


def load_manifest():
    if not MANIFEST.exists():
        return {"packages": []}
    return yaml.safe_load(MANIFEST.read_text()) or {"packages": []}


def canonical_tarball(name, version):
    # A configured mirror resolves the same integrity; the manifest names the
    # canonical registry so a reviewer can fetch and check it.
    base = name.split("/")[-1]
    return f"https://registry.npmjs.org/{name}/-/{base}-{version}.tgz"


def lock_entry(lock, pkg):
    entry = lock.get("packages", {}).get(f"node_modules/{pkg}")
    if entry is None:
        die(f"{pkg} missing from package-lock.json")
    return entry


def inlined_packages(meta):
    names = set()
    for path in meta["inputs"]:
        parts = path.split("node_modules/")
        if len(parts) < 2:
            continue
        seg = parts[-1].split("/")
        names.add("/".join(seg[:2]) if seg[0].startswith("@") else seg[0])
    return names


def build(args, externals, work):
    (work / "package.json").write_text('{"private":true}')
    run(["npm", "install", "--ignore-scripts", "--no-audit", "--no-fund",
         f"{args.name}@{args.version}", ESBUILD], work)
    lock = json.loads((work / "package-lock.json").read_text())
    entry = args.name + (f"/{args.entry}" if args.entry else "")
    (work / "entry.mjs").write_text(f'export * from "{entry}";\n'
                                    f'export {{ default }} from "{entry}";\n'
                                    if args.default_export else
                                    f'export * from "{entry}";\n')
    cmd = ["npx", "--no-install", "esbuild", "entry.mjs", "--bundle",
           "--format=esm", "--target=es2022", "--charset=utf8",
           "--legal-comments=inline", "--outfile=out/index.js",
           "--metafile=meta.json", "--log-level=warning"]
    cmd += [f"--external:{e}" for e in sorted(externals)]
    run(cmd, work)
    meta = json.loads((work / "meta.json").read_text())
    return lock, meta, (work / "out/index.js").read_text()


def main():
    p = argparse.ArgumentParser()
    p.add_argument("name")
    p.add_argument("version")
    p.add_argument("--why", required=True)
    p.add_argument("--owner", default="drive-workspace")
    p.add_argument("--entry", default="")
    p.add_argument("--wasm", default="", help="wasm file inside the entry dir")
    p.add_argument("--css", default="", help="stylesheet inside the package")
    p.add_argument("--external", default="")
    p.add_argument("--default-export", action="store_true")
    args = p.parse_args()
    manifest = load_manifest()
    others = [e for e in manifest["packages"] if e["name"] != args.name]
    externals = {e["name"] for e in others}
    externals |= {x for x in args.external.split(",") if x}

    with tempfile.TemporaryDirectory() as tmp:
        work = pathlib.Path(tmp)
        lock, meta, js = build(args, externals, work)
        top = lock_entry(lock, args.name)
        if top.get("version") != args.version:
            die(f"npm resolved {top.get('version')}, not {args.version}")
        out_dir = VENDOR_DIR / f"{args.name}@{args.version}"
        for old in VENDOR_DIR.glob(f"{args.name}@*"):
            shutil.rmtree(old)
        out_dir.mkdir(parents=True)
        files = []
        if args.wasm:
            src_dir = work / "node_modules" / args.name / pathlib.Path(args.entry or ".")
            data = (src_dir / args.wasm).read_bytes()
            sha, sri = digests(data)
            stem = args.wasm[: -len(WASM_SUFFIX)]
            hashed = f"{stem}.{sha[:12]}{WASM_SUFFIX}"
            quoted = f'"{args.wasm}"'
            if quoted not in js:
                die(f"bundle does not reference {quoted}")
            js = js.replace(quoted, f'"{hashed}"')
            (out_dir / hashed).write_bytes(data)
            files.append({"path": f"{out_dir.relative_to(VENDOR_DIR).as_posix()}/{hashed}", "kind": "wasm",
                          "sha256": sha, "sri": sri})
        if args.css:
            data = (work / "node_modules" / args.name / args.css).read_bytes()
            sha, sri = digests(data)
            hashed = f"{pathlib.Path(args.css).stem}.{sha[:12]}.css"
            (out_dir / hashed).write_bytes(data)
            files.append({"path": f"{out_dir.relative_to(VENDOR_DIR).as_posix()}/{hashed}", "kind": "style",
                          "sha256": sha, "sri": sri})
        data = js.encode()
        sha, sri = digests(data)
        name = f"index.{sha[:12]}.js"
        (out_dir / name).write_bytes(data)
        files.insert(0, {"path": f"{out_dir.relative_to(VENDOR_DIR).as_posix()}/{name}", "kind": "module",
                         "specifier": args.name, "sha256": sha, "sri": sri})
        bundled = []
        for dep in sorted(inlined_packages(meta) - {args.name}):
            e = lock_entry(lock, dep)
            bundled.append({"name": dep, "version": e["version"],
                            "license": e.get("license", "UNKNOWN"),
                            "integrity": e["integrity"]})

    prev = next((e for e in manifest["packages"] if e["name"] == args.name), {})
    entry = {
        "name": args.name,
        "version": args.version,
        "license": top.get("license", "UNKNOWN"),
        "source": {"tarball": canonical_tarball(args.name, args.version), "integrity": top["integrity"],
                   "entry": args.entry or ".", "bundler": ESBUILD},
        "why": args.why or prev.get("why", ""),
        "owner": args.owner or prev.get("owner", ""),
        "approved_by": "",
        "approved_on": "",
        "bundled": bundled,
        "files": files,
    }
    manifest["packages"] = sorted(others + [entry], key=lambda e: e["name"])
    body = yaml.safe_dump(manifest, sort_keys=False, width=100)
    MANIFEST.write_text(HEADER + "\n" + body)
    subprocess.run([sys.executable, "-I", str(ROOT / "scripts/vendor-assets-rs.py")], check=True)
    print(f"vendored {args.name}@{args.version}: {len(files)} file(s), "
          f"{len(bundled)} inlined dependency(ies); review vendor.yaml")


if __name__ == "__main__":
    main()
