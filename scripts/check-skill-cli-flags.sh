#!/usr/bin/env bash
# Check that every `systemprompt` invocation an admin skill documents is one
# the CLI accepts.
#
# The skills are the only place a model learns which flags exist. In a
# production session an administrator's model spent fifteen tool calls on
# `--help`, invented `--until` and `--export` on `infra logs request list`,
# and never found the `--user` flag that was there — because the skill
# listed domains, not flags, and nothing tied its prose to the clap
# definitions. Core is a registry dependency, so the two drift silently on
# every version bump.
#
# The gate extracts each `systemprompt <domain> <sub...> [--flag ...]` line
# from a fenced code block or inline code in services/skills/*/SKILL.md, and
# each `| \`<domain> <sub...>\` | <flags> |` row of a flags table, then runs
# `<command path> --help` against a built binary and requires every long
# flag to appear in that help text. It needs the binary:
# with none built it prints a notice and passes, so a clone without a build
# is not red for the wrong reason — CI builds before the gates run.
#
# Exemption: a code line ending in `# skill-flags: skip` is not checked (for
# an illustrative command that is not meant to run as written).
set -uo pipefail

SKILL_ROOT="${SKILL_ROOT:-services/skills}"
CLI="${CLI:-}"
if [ -z "$CLI" ]; then
    for candidate in target/release/systemprompt target/debug/systemprompt; do
        if [ -x "$candidate" ]; then CLI="$candidate"; break; fi
    done
fi
if [ -z "$CLI" ] || [ ! -x "$CLI" ]; then
    echo "check-skill-cli-flags: no built systemprompt binary (target/{release,debug}); skipping"
    exit 0
fi

export CLI
python3 - "$SKILL_ROOT" <<'PY'
import os, re, subprocess, sys, pathlib

root = pathlib.Path(sys.argv[1])
cli = os.environ["CLI"]
help_cache = {}
violations = []

def help_text(words):
    key = tuple(words)
    if key not in help_cache:
        try:
            out = subprocess.run(
                [cli, *words, "--help"],
                capture_output=True, text=True, timeout=30,
                env={**os.environ, "SYSTEMPROMPT_NON_INTERACTIVE": "1"},
            )
            help_cache[key] = out.stdout + out.stderr if out.returncode == 0 else None
        except (OSError, subprocess.TimeoutExpired):
            help_cache[key] = None
    return help_cache[key]

def command_path(tokens):
    # Words up to the first flag or placeholder; the CLI resolves the deepest
    # subcommand that exists, so probe from longest to shortest.
    words = []
    for tok in tokens:
        if tok.startswith("-") or tok.startswith("<") or tok.startswith("{") or tok.startswith("'") or tok.startswith('"'):
            break
        words.append(tok)
    for n in range(len(words), 0, -1):
        text = help_text(words[:n])
        if text is not None:
            return words[:n], text
    return None, None

LINE = re.compile(r"(?:^|`|\$ )systemprompt ((?:[a-z][a-z0-9-]*)(?:[^`\n|]*))")
# `| `infra logs audit <id>` | `--messages/-m`, `--offset` ... |` — the first
# cell names one or more commands (`a` / `b` share the flags), the rest of
# the row carries the flags.
TABLE_ROW = re.compile(r"^\|\s*`([a-z][^`]*)`((?:\s*/\s*`[^`]*`)*)\s*\|(.*)\|\s*$")
DOMAINS = ("core", "infra", "admin", "cloud", "analytics", "web", "plugins", "build")

def check_flags(skill, lineno, words, text, flags):
    for tok in flags:
        flag = tok.split("=")[0].split("/")[0]
        if flag == "--help":
            continue
        if not re.search(rf"(^|\s){re.escape(flag)}(\s|,|$|\[|<)", text, re.M):
            violations.append(
                f"{skill}:{lineno}: `systemprompt {' '.join(words)}` has no `{flag}`"
            )

def check_table_row(skill, lineno, line):
    row = TABLE_ROW.match(line)
    if not row:
        return
    first = row.group(1).split()
    if not first or first[0] not in DOMAINS:
        return
    base = [t for t in first if not t.startswith("<")]
    siblings = [base]
    for extra in re.findall(r"`([^`]*)`", row.group(2)):
        siblings.append(base[:-1] + extra.split())
    flags = [f.split("\\|")[0] for f in re.findall(r"`(--[a-z-]+(?:/-[a-z])?)", row.group(3))]
    for words_in in siblings:
        words, text = command_path(words_in)
        if words is None:
            violations.append(f"{skill}:{lineno}: `systemprompt {' '.join(words_in)}` — no such command")
            continue
        check_flags(skill, lineno, words, text, flags)

for skill in sorted(root.glob("*/SKILL.md")):
    for lineno, line in enumerate(skill.read_text().splitlines(), 1):
        if "skill-flags: skip" in line:
            continue
        check_table_row(skill, lineno, line)
        for match in LINE.finditer(line):
            body = match.group(1).strip()
            body = re.sub(r"\s+#.*$", "", body)
            # Alternatives like `promote|demote` and `model\|provider` are
            # documentation notation, not argv; take the first alternative.
            body = body.replace("\\|", "|")
            tokens = [t.split("|")[0] for t in body.split()]
            if not tokens or tokens[0] in ("<domain>",):
                continue
            words, text = command_path(tokens)
            if words is None:
                violations.append(f"{skill}:{lineno}: `systemprompt {body}` — no such command")
                continue
            check_flags(skill, lineno, words, text, [t for t in tokens[len(words):] if t.startswith("--")])

if violations:
    print("check-skill-cli-flags: skills document flags the CLI does not accept:")
    for v in violations:
        print(f"  {v}")
    sys.exit(1)
print(f"check-skill-cli-flags: OK ({len(help_cache)} commands probed with {cli})")
PY
