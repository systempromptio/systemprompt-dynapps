#!/usr/bin/env python3
"""Add the agent-farm kit source to a local profile before its first boot.

The dynapps-agent-farm marketplace is published as a signed OCI services
bundle, the same one production composes. A local instance needs the source
in `services.sources` and a GHCR pull token in its secrets before the first
boot, because the first boot seeds the access-control entities, and the vet
demo grants rules on that marketplace.

The token comes from GHCR_PULL_TOKEN, else `gh auth token`. Without one the
source is left out and the instance boots without the kit.

Usage: setup-local-kit-source.py <profile.yaml> <secrets.json>
"""

import json
import os
import re
import subprocess
import sys

SOURCE_NAME = "dynapps-agent-farm"
SOURCE_BLOCK = f"""  sources:
    - name: {SOURCE_NAME}
      oci:
        reference: ghcr.io/systempromptio/systemprompt-agent-farm:stable
        auth_secret: ghcr_pull_token
        verify:
          ed25519_public_keys: ["B4UBK6UgOqiVwFETibS+a65y4Yf7ALC68CSosD1+RKE="]
"""


def pull_token() -> str:
    token = os.environ.get("GHCR_PULL_TOKEN", "").strip()
    if token:
        return token
    try:
        out = subprocess.run(
            ["gh", "auth", "token"], capture_output=True, text=True, check=True
        )
    except (OSError, subprocess.CalledProcessError):
        return ""
    return out.stdout.strip()


def add_source(profile_path: str) -> bool:
    text = open(profile_path, encoding="utf-8").read()
    if f"name: {SOURCE_NAME}" in text:
        return False
    services = re.search(r"^services:\n((?:  .*\n|\n)*)", text, flags=re.M)
    if services is None:
        sys.exit(f"setup-local-kit-source: no services: block in {profile_path}")
    body = re.sub(r"^  sources: \[\]\n", "", services.group(1), flags=re.M)
    if re.search(r"^  sources:", body, flags=re.M):
        sys.exit(
            f"setup-local-kit-source: {profile_path} already declares other "
            "services.sources; add the agent-farm source by hand"
        )
    text = text[: services.start(1)] + SOURCE_BLOCK + body + text[services.end(1) :]
    open(profile_path, "w", encoding="utf-8").write(text)
    return True


def store_token(secrets_path: str, token: str) -> None:
    secrets = json.load(open(secrets_path, encoding="utf-8"))
    secrets["ghcr_pull_token"] = token
    fd = os.open(secrets_path, os.O_WRONLY | os.O_TRUNC | os.O_CREAT, 0o600)
    with os.fdopen(fd, "w", encoding="utf-8") as out:
        json.dump(secrets, out, indent=2)
        out.write("\n")


def main() -> None:
    if len(sys.argv) != 3:
        sys.exit("usage: setup-local-kit-source.py <profile.yaml> <secrets.json>")
    profile_path, secrets_path = sys.argv[1], sys.argv[2]
    token = pull_token()
    if not token:
        print(
            "No GHCR pull token (set GHCR_PULL_TOKEN or run `gh auth login`): "
            "the agent-farm kit is not composed and `just demo-vet-seed` "
            "cannot grant its marketplace."
        )
        return
    store_token(secrets_path, token)
    added = add_source(profile_path)
    print(
        "Added the agent-farm kit source to the local profile."
        if added
        else "The agent-farm kit source is already in the local profile."
    )


if __name__ == "__main__":
    main()
