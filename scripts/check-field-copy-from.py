#!/usr/bin/env python3
"""Report `From` impls that only copy fields across, name for name.

A `From` that reads

    impl From<&Wire> for Runtime {
        fn from(c: &Wire) -> Self {
            Self { a: c.a, b: c.b, ... }
        }
    }

converts nothing. It says the two types are the same type, written twice, and
the compiler will not tell you when they drift apart. `config::RateLimitConfig`
was exactly this against `profile::RateLimitsConfig` -- 15 fields, identical
names and types, plus a second set of hardcoded defaults that nothing kept in
step. It cost three sessions a wrong diagnosis in one afternoon, because both
types were plausible answers to `grep RateLimit`, and it was invisible to
`check-duplicate-types.sh`, which reads names within one crate and so sees
neither the different names nor the different crates.

The rule: a `From` earns its place by doing something -- renaming, narrowing,
defaulting, tightening an invariant, changing a representation. One that only
restates the same fields should be a single type, or a borrow.

A projection that drops or renames fields is real work and is not reported;
so is anything under the threshold, where two types sharing a few field names
is coincidence rather than duplication.

Exemption: `// lint-ok: field-copy-from` on the line above the impl, with a
reason.
"""
import re
import subprocess
import sys
from pathlib import Path

MIN_FIELDS = 5
SRC_DIRS = ["extensions", "src", "bridge/src"]

IMPL_RE = re.compile(
    r"^[ \t]*impl(?:<[^>]*>)?\s+From\s*<\s*"
    r"(?:&\s*)?(?:'[\w]+\s+)?(?:&\s*)?"      # &, &'a, &'a mut
    r"([\w:]+)"                                # the source type
    r"(?:<[^>]*>)?\s*>\s+for\s+([\w:]+)",
    re.M,
)
# `name: src.name`, tolerating .clone(), .to_owned(), .into(), and a leading &
FIELD_RE = re.compile(r"^\s*(\w+)\s*:\s*&?(?:\w+)\.(\w+)(?:\s*\.\s*(?:clone|to_owned|to_string|into)\s*\(\s*\))?\s*,\s*$")


def tracked_files():
    out = subprocess.run(
        ["git", "ls-files", "-co", "--exclude-standard", *SRC_DIRS],
        capture_output=True, text=True, check=False,
    ).stdout.split()
    return [Path(f) for f in out if f.endswith(".rs")]


def block_at(lines, start):
    """Return the lines of the brace-balanced block beginning at `start`."""
    depth, body, started = 0, [], False
    for line in lines[start:]:
        depth += line.count("{") - line.count("}")
        body.append(line)
        if "{" in line:
            started = True
        if started and depth <= 0:
            break
    return body


def main():
    findings = []
    for path in tracked_files():
        try:
            text = path.read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError):
            continue
        if "From<" not in text:
            continue
        lines = text.splitlines()
        for match in IMPL_RE.finditer(text):
            lineno = text[: match.start()].count("\n")
            # Walk back through the contiguous comment block above the impl, so
            # the marker may head a multi-line reason rather than share its line.
            exempt, cursor = False, lineno - 1
            while cursor >= 0 and lines[cursor].strip().startswith("//"):
                if "lint-ok: field-copy-from" in lines[cursor]:
                    exempt = True
                    break
                cursor -= 1
            if exempt:
                continue
            body = block_at(lines, lineno)
            copied, other = 0, 0
            for line in body[1:]:
                stripped = line.strip()
                if not stripped or stripped.startswith("//"):
                    continue
                # The `fn from` signature and any block opener are scaffolding,
                # not conversion work.
                if stripped.startswith(("fn ", "pub fn ")) or stripped.endswith("{"):
                    continue
                field = FIELD_RE.match(line)
                if field:
                    if field.group(1) == field.group(2):
                        copied += 1
                    else:
                        other += 1  # a rename is real work
                elif stripped not in ("}", "});", "Self {", "}, ") and ":" in stripped:
                    other += 1
            if copied >= MIN_FIELDS and other == 0:
                findings.append(
                    f"{path}:{lineno + 1}: From<{match.group(1)}> for {match.group(2)} "
                    f"copies {copied} fields and changes nothing"
                )

    if findings:
        print("Field-for-field `From` impls (these two types should be one):\n")
        for f in findings:
            print(f"  {f}")
        print(
            "\nCollapse the pair, or borrow instead of converting. If the duplication is\n"
            "deliberate, annotate the impl with `// lint-ok: field-copy-from` and say why."
        )
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
