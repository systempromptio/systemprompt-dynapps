# Developer Workflow (Steps 3a–3e)

## 3a — GitHub SSH authentication

Before cloning private repos, verify SSH access:

```bash
ssh -T git@github.com 2>&1
```

If it **fails**, guide the user through SSH key generation and GitHub registration. Use `scripts/setup_ssh.sh` as reference. Do NOT proceed until SSH auth works — Enterprise and Indaws Core repos require it.

## 3b — Resolve source paths

Cascade:

1. **Workspace context file** — Read the project context file from the current workspace, walking up to the repo root (`AGENTS.md`, then `CLAUDE.md`); fall back to the user-global one if the workspace has none. Look for `## Rutas Locales Odoo`
2. **Default paths** — See `references/git-repos.md` → Default paths table
3. **Ask the user** — If defaults don't exist, offer:
   - "I'll provide custom paths" → ask for each (Community required, rest optional)
   - "I don't have source code yet" → create standard structure with `mkdir -p`

## 3c — Clone or update repos

See `references/git-repos.md` for URLs, auth methods, and clone commands.

Rules:
- All clones use `--depth 1` (reading source, not history)
- HTTPS for public repos (CE, OCA), SSH for private (Enterprise, Indaws Core)
- If repo exists → `git pull`, don't re-clone
- OCA: run `scripts/get_oca.sh {oca_path} {version}` (handles 117 core repos)
- Report each result: cloned / updated / skipped / error

## 3d — Create symlinks

| Symlink | Target |
|---|---|
| `base` | `{odoo_base}/odoo{major_version}/addons` |
| `enterprise` | `{enterprise_path}/{major_version}` |
| `OCA` | `{oca_path}/v{major_version}` |
| `indaws-core` | `{indaws_path}/{major_version}` |

> **`{version}` (rama, `18.0`) ≠ nombre del directorio.** Cada raíz lo nombra distinto:
> `odoo18` / `18` / `v18` / `18`. Esta tabla usaba `{version}` en las tres primeras hasta
> 2026-08-07, así que los symlinks apuntaban a directorios que no existen (o, peor, a un árbol
> paralelo que solo esta skill creaba). Ver `references/shared/odoo-source.md`.

Before creating:
1. Target must exist — skip with note if not
2. Symlink already correct → skip
3. Symlink points elsewhere → warn and ask before overwriting

## 3e — Save paths to global config

If paths were resolved by asking (not read from a context file), save them to the user-global context file of the running agent under `## Rutas Locales Odoo` so future projects don't ask again. Ask the user where that is rather than assuming a path.
