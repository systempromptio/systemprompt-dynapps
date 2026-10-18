#!/usr/bin/env python3
"""Refuse a deploy whose profile pins a services bundle the registry cannot serve.

A bundle that fails to fetch at boot is not an error the operator sees: core
falls back to the last-good composition (`on_fetch_failure: use_last_good`),
which is the tree from BEFORE this release. Migrations still run, so the
database moves forward while the services tree moves back — a renamed group
or marketplace then matches nothing and its members silently lose access.
Every `oci.reference` in the deploy profile is therefore resolved
against GHCR with the profile's `ghcr_pull_token` before anything ships, and
every `owner: bundle:<name>` in rules.yaml must name a source the profile
pins, and every pinned source must be owned by one: a release that renames a
kit's source must neither ship on the old pin nor keep serving it beside the new.

Usage: check-deploy-sources.py <profile.yaml>   (secrets.json beside it)
"""

import base64
import json
import re
import sys
import urllib.error
import urllib.request
from pathlib import Path


def references(profile: Path) -> list[tuple[str, str]]:
    found, name = [], None
    for line in profile.read_text().splitlines():
        m = re.match(r"\s*- name:\s*(\S+)", line)
        if m:
            name = m.group(1)
        m = re.match(r"\s*reference:\s*(\S+)", line)
        if m and name:
            found.append((name, m.group(1)))
    return found


def resolves(reference: str, token: str | None) -> str | None:
    """Returns None when the reference resolves, else the reason it does not."""
    m = re.match(r"ghcr\.io/([^:@]+)[:@](.+)$", reference)
    if not m:
        return None  # not GHCR; core resolves other registries itself
    repo, ref = m.groups()
    auth = {}
    if token:
        auth["Authorization"] = "Basic " + base64.b64encode(token.encode()).decode()
    try:
        req = urllib.request.Request(
            f"https://ghcr.io/token?scope=repository:{repo}:pull", headers=auth
        )
        bearer = json.load(urllib.request.urlopen(req, timeout=20))["token"]
        req = urllib.request.Request(
            f"https://ghcr.io/v2/{repo}/manifests/{ref}",
            method="HEAD",
            headers={
                "Authorization": f"Bearer {bearer}",
                "Accept": "application/vnd.oci.image.manifest.v1+json,"
                "application/vnd.oci.image.index.v1+json",
            },
        )
        urllib.request.urlopen(req, timeout=20)
        return None
    except urllib.error.HTTPError as e:
        return f"HTTP {e.code}"
    except (urllib.error.URLError, KeyError, ValueError) as e:
        return str(e)


def owned_bundles() -> set[str]:
    rules = Path(__file__).resolve().parent.parent / "services/access-control/rules.yaml"
    return set(re.findall(r"^\s*owner:\s*bundle:(\S+)", rules.read_text(), re.M))


def main() -> int:
    profile = Path(sys.argv[1])
    secrets_path = profile.with_name("secrets.json")
    token = None
    if secrets_path.is_file():
        token = json.loads(secrets_path.read_text()).get("ghcr_pull_token")
    failed = 0
    pinned = {name for name, _ in references(profile)}
    for bundle in sorted(owned_bundles() - pinned):
        failed += 1
        print(
            f"✗ rules.yaml grants a marketplace owned by bundle:{bundle}, "
            f"but the profile pins no source named {bundle}",
            file=sys.stderr,
        )
    for name in sorted(pinned - owned_bundles()):
        failed += 1
        print(
            f"✗ the profile pins source {name}, which no rules.yaml entity owns — "
            f"a retired or renamed kit; remove its block from the profile",
            file=sys.stderr,
        )
    for name, ref in references(profile):
        reason = resolves(ref, token)
        if reason:
            failed += 1
            print(f"✗ source {name}: {ref} does not resolve ({reason})", file=sys.stderr)
        else:
            print(f"✓ source {name}: {ref}")
    if failed:
        print(
            "Refusing to deploy: the server would boot on the last-good services tree "
            "while migrations move the database forward. Publish the kit (or re-pin "
            "with `just services-pin <kit> <ref>`) and deploy again.",
            file=sys.stderr,
        )
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
