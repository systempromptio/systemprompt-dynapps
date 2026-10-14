# Scenarios — Dynapps Migrator

El orquestador resuelve, en el entry-point, **un único campo `scenario`** que gobierna
el track de código y el go-live. No se reimplementa nada: cada escenario **enruta** a
las piezas que ya existen (`git-manager`, `odoo-database-upgrader`, `dynapps-local-env-setup`).

## Los dos escenarios

### `onpremise_gitlab` — Bélgica dyncloud + España

- **Hosting código:** GitLab Dynapps (`gitlab.dynapps.be:customers/<v>/<cliente>/odoo`).
- **CI:** pipeline GitLab (inyectado desde `infra/odoo-gitlabci`), gate = **pipeline verde**.
- **Buildout:** `dyn-build.yaml` + `parts/oca_pull` (dyn-tools).
- **BD:** Upgrade Service on-premise (SSH al servidor de BD de producción del cliente —
  el path exacto lo resuelve `dynapps-db-upgrade-runner`; `test`/`production`).
- **Go-live:** subida manual del dump (fuera de alcance del hito local).
- **git-manager:** proveedor `gitlab` (`glab`).

### `odoosh_github` — Bélgica odoo.sh

- **Hosting código:** GitHub (`github.com:DynAppsNV/<cliente>` y workdir `indawsodoo/odoo-migkit-<cliente>`).
- **CI:** **sin pipeline GitLab**. Gate = **pre-commit + tests locales** (GitHub Action `pre-commit-ci` si existe, pero no bloquea el hito local).
- **Buildout:** addons OCA **vendorizados a mano** en `parts/oca` (NO `parts/oca_pull`, que depende de dyn-tools GitLab).
- **BD + go-live:** delegados a `odoo-database-upgrader` con `target_deployment = odoo.sh`
  (upgrade **nativo** de odoo.sh, **siempre `test`**, fixes vía glue module — odoo.sh no
  acepta dumps externos en prod). Ver `odoo-database-upgrader` FASE 0 y
  `playbooks/go_live/odoosh_cutover_commit.md`.
- **git-manager:** proveedor `github` (`gh`).

## Precedencia de detección (determinista — no preguntar de más)

Resolver `scenario` con la primera señal que aplique, en este orden:

1. **Declaración explícita** del usuario en el prompt ("odoo.sh", "on-premise", "dyncloud", "Combell").
2. **Host del `repo`:** `gitlab.dynapps.be` → `onpremise_gitlab`; `github.com` → `odoosh_github`.
3. **`.copier-answers.yml → ci_system`** si el buildout ya existe: `gitlab` → `onpremise_gitlab`
   (cualquier otro valor / ausencia con remote github → `odoosh_github`).
4. **Keyword `infraestructura`:** `SH`/`OdooSH` → `odoosh_github`; `Combell`/`on-premise`/`local` → `onpremise_gitlab`.
5. **Solo si sigue ambiguo** → AskUserQuestion (único STOP de detección; ver `stop-points.md`).

```bash
# Señal 2 — host del repo (la más fiable)
case "$REPO" in
  *gitlab.dynapps.be*) SCENARIO=onpremise_gitlab ;;
  *github.com*)        SCENARIO=odoosh_github ;;
esac

# Señal 3 — copier-answers si hay buildout
CI_SYSTEM=$(grep -E '^ci_system:' "$BUILDOUT_DIR/.copier-answers.yml" 2>/dev/null | awk '{print $2}')
[ "$CI_SYSTEM" = "gitlab" ] && SCENARIO=${SCENARIO:-onpremise_gitlab}
```

## Qué cambia por escenario (tabla de routing)

| Aspecto | `onpremise_gitlab` | `odoosh_github` |
|---|---|---|
| git-manager provider | `gitlab` (`glab`) | `github` (`gh`) |
| Prerrequisitos código | `dynapps-code-migration` STEP 0.5 (`gitlab-prerequisites.md`) | **se salta** `gitlab-prerequisites.md` |
| Gate de código (para el hito) | verde en local (CI está fuera del hito) | pre-commit + tests locales |
| Addons OCA | `parts/oca_pull` | `parts/oca` (vendorizado a mano) |
| BD / Upgrade Service | on-premise (SSH prod, test/prod) | odoo.sh nativo (siempre test) |
| local-env-setup | flujo GitLab | pasar `--odoo-sh` + responder repo GitHub |

> **Nota:** el port de módulos y el iterate `-u` (`odoo-module-upgrader`, iterate loop)
> son **idénticos** en ambos escenarios — son de Odoo, no de plataforma. Solo cambian el
> proveedor git, los prerrequisitos y la gestión de addons.

`scenario` se añade a `migration_context` (ver `input-detection.md`) y se propaga a cada
skill delegada.
