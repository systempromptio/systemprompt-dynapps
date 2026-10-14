# Git Repository URLs

Source repos for Odoo development. All clones use `--depth 1`.

> **La rama es `{version}` (`18.0`); el directorio NO.** `$ODOO_SRC/odoo18`, `$ODOO_ENTERPRISE_SRC/18`, `$OCA_SRC/v18`, `indawsodoo/18` — cuatro convenciones distintas, y confundirlas crea un árbol paralelo que ninguna otra skill lee. Fuente: `references/shared/odoo-source.md`.

## Odoo Community (CE)
- **URL:** `https://github.com/odoo/odoo.git`
- **Auth:** Public (HTTPS)
- **Branch:** `{version}` (e.g., `18.0`)
- **Target:** `{odoo_base}/odoo{major_version}`  ← `odoo18`, NO `18.0`
- **Symlink:** `base` → `{odoo_base}/odoo{major_version}/addons`

## Odoo Enterprise (EE)
- **URL:** `git@github.com:indawsodoo/enterprise.git`
- **Auth:** SSH (private) — requires GitHub SSH key
- **Branch:** `{version}` (e.g., `18.0`)
- **Target:** `{enterprise_path}/{major_version}`  ← `18`, sin decimal
- **Symlink:** `enterprise` → `{enterprise_path}/{major_version}`

## Indaws Core
- **URL:** `git@github.com:indawsodoo/indaws-odoo.git`
- **Auth:** SSH (private) — requires GitHub SSH key
- **Branch:** `{version}` (e.g., `18.0`)
- **Target:** `{indaws_path}/{major_version}`
- **Symlink:** `indaws-core` → `{indaws_path}/{major_version}`

## OCA
- **URL pattern:** `https://github.com/OCA/{repo}.git`
- **Auth:** Public (HTTPS)
- **Branch:** `{version}` (e.g., `18.0`)
- **Target:** `{oca_path}/v{major_version}/{repo}`  ← `v18`, con la `v`
- **Symlink:** `OCA` → `{oca_path}/v{major_version}`
- **Script:** `scripts/get_oca.sh {oca_path} {version}`
- **Core repos:** **109** (contados en el array `CORE_REPOS` de `scripts/get_oca.sh` y en la lista de `references/oca-repos.md`; decía 117 hasta 2026-08-07)
- **On-demand repos:** 142

## Default paths

> These are **defaults, not assumptions** — overridable per environment (env vars /
> project `CLAUDE.md`). Resolve the actual base via `references/shared/odoo-source.md`
> before using; never hardcode them downstream.

| Component | Default base path |
|---|---|
| Odoo CE | `$ODOO_SRC` |
| Odoo EE | `$ODOO_ENTERPRISE_SRC` |
| OCA | `$OCA_SRC` |
| Indaws Core | `$PROJECTS_ROOT/indawsodoo` |

## Clone commands

> ⚠️ **La rama y el directorio NO se llaman igual, y cada raíz nombra su directorio distinto.**
> La rama de git es `{version}` (`18.0`); el directorio destino es **`odoo{major}` / `{major}` /
> `v{major}` / `{major}`** según la raíz. Escribir `{odoo_base}/{version}` clona en
> un directorio con nombre de rama (`…/18.0`) mientras **todas las demás skills leen
> `$ODOO_SRC/odoo18`** — dos árboles, uno invisible. Corregido 2026-08-07: esta sección tenía `{version}` en CE, EE y OCA (solo Indaws
> Core usaba `{major_version}`, ya bien). Convención en
> `references/shared/odoo-source.md`.
>
> | Raíz | Directorio para 18.0 |
> |---|---|
> | `$ODOO_SRC` | `odoo18` |
> | `$ODOO_ENTERPRISE_SRC` | `18` |
> | `$OCA_SRC` | `v18` |
> | `$PROJECTS_ROOT/indawsodoo` | `18` |

### First time
```bash
# {version} = rama (18.0) · {major_version} = major sin decimal (18)

# CE (public) — directorio odoo{major}, NO {version}
git clone -b {version} --depth 1 https://github.com/odoo/odoo.git {odoo_base}/odoo{major_version}

# EE (SSH) — directorio {major}
git clone -b {version} --depth 1 git@github.com:indawsodoo/enterprise.git {enterprise_path}/{major_version}

# Indaws Core (SSH) — directorio {major}
git clone -b {version} --depth 1 git@github.com:indawsodoo/indaws-odoo.git {indaws_path}/{major_version}

# OCA (109 repos core via script) — el script crea v{major}; comprueba con `ls {oca_path}` al acabar
bash scripts/get_oca.sh {oca_path} {version}
```

Después de clonar, **verifica que el directorio es el que espera el resto del ecosistema**:
`ls -d {odoo_base}/odoo{major_version} {enterprise_path}/{major_version} {oca_path}/v{major_version}`.
Si alguno no está, la ruta se escribió con la convención equivocada y las demás skills no lo verán.

### Update existing — SAFE protocol (never `git pull` blind)

A blind `git pull` over a dirty tree or a feature branch **stomps work or fails**. Always
**fetch first** (ahead/behind counters are stale until you fetch) and inspect state per repo:

```bash
cd {path}
git fetch origin {version}                                   # safe: no working-tree change
DIRTY=$(git status --porcelain | head -1)
BRANCH=$(git rev-parse --abbrev-ref HEAD)
BEHIND=$(git rev-list --count HEAD..origin/{version} 2>/dev/null)
```

Decision table (STOP and ask the user on any ⚠️ — do not auto-resolve):

| Situación | Acción |
|-----------|--------|
| Limpio, en `{version}`, `BEHIND>0` | `git pull --ff-only origin {version}` (seguro) |
| Limpio, ya al día (`BEHIND=0`) | nada |
| ⚠️ Working tree **sucio** (`DIRTY`) | STOP: los cambios locales pueden ser intencionados. Preguntar: stash+pull+pop / descartar / no tocar. **Nunca** descartar sin confirmar. |
| ⚠️ Rama ≠ `{version}` (feature branch) + untracked WIP | STOP: es trabajo en curso. No hacer checkout ni pull sin preguntar. |
| ff-only falla (diverge) | STOP: no forzar; reportar y preguntar. |

**OCA = N repos independientes** (no un repo): actualizar todos es lento. Ofrecer scope
"solo los que usa el proyecto" (derivar de `depends` de `desarrollos/`) vs "todos". Reportar los
que fallen; nunca declarar "actualizado" en silencio si alguno quedó fuera.
