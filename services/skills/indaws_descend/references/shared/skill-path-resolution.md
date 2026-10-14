# Skill Path Resolution

Single source of truth for locating **sibling skill scripts** and **per-user state** from
inside a skill. **Do NOT hardcode agent-specific paths in your skill — reference this
document.**

A skill must not assume which agent is running it. `~/.claude/` exists only under Claude
Code; opencode uses `~/.config/opencode/`, and a developer checkout uses neither. Hardcoding
any single one of them makes the skill silently non-portable.

## Locating sibling skill scripts

Resolve in this order, first hit wins. Every entry is a *candidate root*; keep them all.

| Order | Root | Why |
|-------|------|-----|
| 1 | `$<SKILL>_SCRIPTS` (explicit override, e.g. `$ODOO_PILOT_SCRIPTS`) | Lets the caller pin an exact path. Always honour it first. |
| 2 | `$DYNAPPS_SKILLS_ROOT` | Agent-agnostic override for the whole suite tree. |
| 3 | `$SKILLS_ROOT` | Pre-existing convention in developer checkouts. |
| 4 | `$HOME/.claude/plugins/cache/*/*/[*/]skills/<skill>/scripts` | **Claude Code.** Keep it: removing it breaks every current user. |
| 5 | `$HOME/.config/opencode/skills/<skill>/scripts` | **opencode**, skills installed globally. |
| 6 | `$HOME/.dyn-skills/*/suites/*/skills/<skill>/scripts` | **opencode** via the Dynapps marketplace installer. |

### Bash

```bash
# resolve_skill_scripts <skill-name> -> absolute path on stdout, empty if not found
resolve_skill_scripts() {
    local skill="$1" candidate
    for candidate in \
        ${DYNAPPS_SKILLS_ROOT:+"$DYNAPPS_SKILLS_ROOT"/*/skills/"$skill"/scripts} \
        ${DYNAPPS_SKILLS_ROOT:+"$DYNAPPS_SKILLS_ROOT"/skills/"$skill"/scripts} \
        ${SKILLS_ROOT:+"$SKILLS_ROOT"/*/skills/"$skill"/scripts} \
        "$HOME/.claude/plugins/cache"/*/*/*/skills/"$skill"/scripts \
        "$HOME/.claude/plugins/cache"/*/*/skills/"$skill"/scripts \
        "$HOME/.config/opencode/skills/$skill/scripts" \
        "$HOME/.dyn-skills"/*/suites/*/skills/"$skill"/scripts; do
        [ -d "$candidate" ] && { printf '%s\n' "$candidate"; return 0; }
    done
    return 1
}
```

Usage, with the explicit override honoured first:

```bash
PILOT="${ODOO_PILOT_SCRIPTS:-$(resolve_skill_scripts odoo-pilot)}"
[ -n "$PILOT" ] || { echo "ERROR: odoo-pilot scripts not found. Install common-skills:odoo-pilot, or export ODOO_PILOT_SCRIPTS=/path/to/scripts" >&2; exit 1; }
```

### Python

```python
import os, glob

def resolve_skill_scripts(skill: str) -> str | None:
    home = os.path.expanduser("~")
    patterns = []
    for var in ("DYNAPPS_SKILLS_ROOT", "SKILLS_ROOT"):
        if os.environ.get(var):
            patterns += [f"{os.environ[var]}/*/skills/{skill}/scripts",
                         f"{os.environ[var]}/skills/{skill}/scripts"]
    patterns += [
        f"{home}/.claude/plugins/cache/*/*/*/skills/{skill}/scripts",
        f"{home}/.claude/plugins/cache/*/*/skills/{skill}/scripts",
        f"{home}/.config/opencode/skills/{skill}/scripts",
        f"{home}/.dyn-skills/*/suites/*/skills/{skill}/scripts",
    ]
    for pattern in patterns:
        for hit in sorted(glob.glob(pattern)):
            if os.path.isdir(hit):
                return hit
    return None
```

**Never** pin a version in the path (`common-skills/1.1.0/...`). Versions change on every
release and the pinned path goes stale silently — use the glob.

## Credentials

**Never** read credentials from an agent's config directory, and **never** walk parent
directories looking for a `.env`: a client repo may contain one and you would pick it up
without anyone noticing.

Resolve in this order:

1. An explicit environment variable already exported (`ODOO_URL`, `ODOO_DB`, `ODOO_USER`, `ODOO_KEY`).
2. A `.env` file **in the current workspace root**, if the user points at one.
3. **Ask the user.** Stop and request the path or the values.

Legacy note: `$HOME/.claude/.env.odoo` is still honoured as a fallback for users onboarded
before this change, but it must never be the only path a skill knows.

## Per-user state

State a skill writes (preferences, slots, progress) goes under an agent-agnostic root:

```bash
DYNAPPS_STATE="${DYNAPPS_HOME:-$HOME/.dynapps}/state/<skill-name>"
```

When a legacy location exists under `$HOME/.claude/`, read it as a fallback and prefer the
new root for writes, so existing users keep their state.

## Project context

Do not hardcode `~/.claude/CLAUDE.md`. Read the project context file from the **current
workspace**, walking up to the repository root: `AGENTS.md` first, then `CLAUDE.md`. Both
Claude Code and opencode follow that convention, opencode accepting `CLAUDE.md` as a
fallback. Only fall back to the user-global file if the workspace has none.
