# Changelog

All notable changes to this skill will be documented in this file.

## [2.0.0] - 2026-04-02

### Added
- **Git clone/update workflow**: Automatic clone (`--depth 1`) or pull for all source repos
- **Repo definitions**: Odoo CE (HTTPS public), Odoo EE (SSH private), Indaws Core (SSH private)
- **OCA script** (`scripts/get_oca.sh`): Manages 117 core OCA repos — clone/pull/skip by version
- **OCA on-demand catalog**: 142 additional repos (tooling, l10n, verticals, connectors, Enterprise-covered) available for manual clone when needed
- **SSH authentication check**: Verifies `ssh -T git@github.com` before cloning private repos
- **SSH setup guidance**: Step-by-step instructions for non-technical users (key generation, GitHub settings link, verification)
- **SSH helper script** (`scripts/setup_ssh.sh`): Interactive SSH key setup for users who need it
- **References**: `references/git-repos.md` (repo URLs, auth, paths), `references/oca-repos.md` (full catalog with categories)
- Step 7 summary now includes source code clone/update stats

### Changed
- Version bumped to v2.0
- Step 3 expanded from "symlinks only" to full source code lifecycle (auth → resolve → clone/update → symlink → save)
- OCA symlink now points to `{oca_path}/{version}` (was `OCA-{v}`)
- Gotchas expanded: SSH auth failure, OCA branch missing, shallow clone limitations, network timeouts

### Removed
- Hardcoded path assumptions — all paths resolved dynamically via cascade

**Author:** Toni Hermosilla <toni@indaws.es>

## [1.2.0] - 2026-03-28

### Added
- "What is a project" section: defines project = client root folder
- Safety check: detects when user is inside a module or shared source
- Gotcha for "inside module instead of project root"

### Changed
- Step 0 safety check validates against project definition

**Author:** Toni Hermosilla <toni@indaws.es>

## [1.1.0] - 2026-03-28

### Added
- Smart path resolution cascade: CLAUDE.md global → defaults → ask user
- Standard folder structure proposal for developers
- Workspace structure proposal for consultants
- Auto-save resolved paths to ~/.claude/CLAUDE.md

### Changed
- Step 3 split into substeps (3a-3e) with role-aware logic

**Author:** Toni Hermosilla <toni@indaws.es>

## [1.0.0] - 2026-03-28

### Added
- Initial release (refactored from toni-project-setup v0.1)
- Role detection: Developer vs Consultant
- Developer workflow: version, symlinks, dev skill selection, CLAUDE.md
- Consultant workflow: version, consultant skill selection, CLAUDE.md
- 12 developer + 13 consultant skill mappings
- Dynamic path resolution from global CLAUDE.md
- Safety checks: cwd validation, symlink conflict, re-run detection

**Author:** Toni Hermosilla <toni@indaws.es>
