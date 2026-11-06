#!/usr/bin/env bash
# The third-party frontend allowlist: nothing vendored reaches a page unless
# services/web/vendor.yaml names it, byte for byte.
#
# Fails when:
#   1. a file under storage/files/js/vendor/ is missing from the manifest, or
#      the manifest lists a file that is absent;
#   2. a file's sha256 or sha384 SRI differs from the manifest;
#   3. a package's (or an inlined dependency's) licence is not in deny.toml's
#      [licenses] allow list;
#   4. non-vendor JS imports a bare specifier the import map does not define,
#      an external URL (http:, https:, //) or a /js/vendor/ path directly;
#   5. a template other than partials/vendor-head.hbs names /js/vendor/.
#   6. an entry has an empty approved_by/approved_on: `just vendor-add` clears
#      them, and nothing ships until a reviewer fills them in.
#
# tests/unit/web/src/vendor_allowlist.rs asserts the same rules from cargo,
# plus the registration in admin-core's generated vendor/assets.rs.
set -uo pipefail
cd "$(dirname "$0")/.."

python3 -I - <<'EOF'
import base64
import hashlib
import pathlib
import re
import sys

import yaml

root = pathlib.Path(".")
vendor_dir = root / "storage/files/js/vendor"
manifest = yaml.safe_load((root / "services/web/vendor.yaml").read_text()) or {}
deny = (root / "deny.toml").read_text()
allow_block = re.search(r"\[licenses\].*?^allow\s*=\s*\[(.*?)^\]", deny, re.S | re.M).group(1)
allowed = set(re.findall(r'^\s*"([^"]+)"', allow_block, re.M))
errors, unreviewed = [], []

listed, specifiers = {}, set()
for pkg in manifest.get("packages", []):
    name = pkg["name"]
    if pkg["license"] not in allowed:
        errors.append(f"{name}: licence {pkg['license']} not in deny.toml allow list")
    for dep in pkg.get("bundled", []):
        if dep["license"] not in allowed:
            errors.append(f"{name}: inlined {dep['name']} licence {dep['license']} not allowed")
    if not pkg.get("approved_by") or not pkg.get("approved_on"):
        unreviewed.append(f"{name}@{pkg['version']}")
    for f in pkg["files"]:
        if not f["path"].startswith(f"{name}@{pkg['version']}/"):
            errors.append(f"{f['path']}: not under {name}@{pkg['version']}/")
        listed[f["path"]] = f
        if f["kind"] == "module":
            specifiers.add(f.get("specifier") or name)

on_disk = {p.relative_to(vendor_dir).as_posix() for p in vendor_dir.rglob("*") if p.is_file()} \
    if vendor_dir.is_dir() else set()
for path in sorted(on_disk - listed.keys()):
    errors.append(f"vendor/{path}: on disk but not in vendor.yaml")
for path in sorted(listed.keys() - on_disk):
    errors.append(f"vendor/{path}: in vendor.yaml but missing on disk")
for path in sorted(on_disk & listed.keys()):
    data = (vendor_dir / path).read_bytes()
    want = listed[path]
    if hashlib.sha256(data).hexdigest() != want["sha256"]:
        errors.append(f"vendor/{path}: sha256 differs from vendor.yaml")
    sri = "sha384-" + base64.b64encode(hashlib.sha384(data).digest()).decode()
    if sri != want["sri"]:
        errors.append(f"vendor/{path}: sri differs from vendor.yaml")

IMPORT = re.compile(
    r"""^\s*(?:(?:import|export)\b[^'"\n]*?|[^'"\n]*\}\s*)\bfrom\s*['"]([^'"\n]+)['"]"""
    r"""|^\s*import\s*['"]([^'"\n]+)['"]"""
    r"""|\bimport\(\s*['"]([^'"\n]+)['"]\s*\)""",
    re.M,
)
for js in sorted((root / "storage/files/js").rglob("*.js")):
    if vendor_dir in js.parents:
        continue
    for groups in IMPORT.findall(js.read_text(encoding="utf-8", errors="replace")):
        spec = next(g for g in groups if g)
        if re.match(r"^(https?:)?//", spec):
            errors.append(f"{js}: imports external URL {spec}")
        elif spec.startswith("/js/vendor/"):
            errors.append(f"{js}: imports {spec} directly; use its import-map specifier")
        elif not spec.startswith(("/", ".")) and spec not in specifiers:
            errors.append(f"{js}: bare specifier {spec!r} is not in vendor.yaml")

for tpl in sorted(list((root / "storage/files/admin").rglob("*.hbs"))
                  + list((root / "services/web/templates").rglob("*.html"))):
    if tpl.as_posix().endswith("partials/vendor-head.hbs"):
        continue
    if "/js/vendor/" in tpl.read_text(encoding="utf-8", errors="replace"):
        errors.append(f"{tpl}: names /js/vendor/; vendor code loads only via the workspace import map")

for u in unreviewed:
    errors.append(f"{u}: awaits review (approved_by/approved_on empty)")
if errors:
    print("check-vendor: FAIL")
    for e in errors:
        print(f"  {e}")
    sys.exit(1)
print(f"check-vendor: OK ({len(listed)} vendored files, {len(specifiers)} modules)")
EOF
