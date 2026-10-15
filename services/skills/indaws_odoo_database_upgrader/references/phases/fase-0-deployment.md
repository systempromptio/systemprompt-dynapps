# FASE 0 — Deployment strategy + credenciales

(El detalle por destino —odoo.sh, indaws, custom— se cubre en este mismo archivo.)

## Pasos

1. **[STOP] Preguntar destino:** `odoo.sh` | `indaws.cloud` | `custom`
2. `upgrade_aim`:
   - `odoo.sh` → SIEMPRE `test` (go-live lo hace odoo.sh nativo)
   - `indaws | custom` → `test` durante iteraciones, `production` para go-live
3. Credenciales en `migration_kit_<cliente>/logs/upgrade_credentials.json` (gitignored)
4. Docker compartido: `odoo-upgrade-analyzer` puerto 5434 — nunca recrear
5. Imagen Odoo: `odoo:XX.0` (odoo.sh) | `indaws/odoo-slim:XX.0` (indaws)

## Intake interactivo (credenciales) — [STOP] AskUserQuestion

Cuando no hay config (o el workdir cambió), **preguntar al usuario** y persistir en
`upgrade_credentials.json` (lo gestiona `scripts/init_config.py` — `load`/`save`/`validate`/`docker`):

| # | Pregunta | Notas |
|---|----------|-------|
| 1 | Nº de contrato Enterprise | ej. `M20102320384678`; si la API da `INVALID_CONTRACT` → borrar config y re-preguntar |
| 2 | Email del contrato | |
| 3 | Nombre de BD en el contenedor | solo `[a-z0-9_]` |
| 4 | Ruta absoluta al dump | `.dump` / `.sql` / `.gz` |
| 5 | `version_from` | **NUNCA inferir** (no está en la BD post-upgrade) → bloquea hasta input humano |
| 6 | `version_to` | solo versiones `> version_from` |
| 7 | `pg_version` del dump origen | ej. 12/14/15/16 → nombra el analyzer `odoo-upgrade-analyzer-pg{N}` |
| 8 | `upgrade_aim` | `test` (default, ilimitado) vs `production` (1 por contrato+BD; solo tras iterar OK) |

Tras `load`/`save`, mostrar siempre el **resumen de config activa** (contrato, BD, dump+tamaño, `from→to`, aim, pg, workdir).

## Dos modelos de migración (decide TODO el flujo de go-live)

La diferencia clave del destino no es solo `aim`: son **dos modelos mentales distintos** de cómo llegan los fixes a producción. Decidirlo en FASE 0 condiciona FASE 4 y 5.

| Eje | **on-premise** (`indaws.cloud` / `custom`) | **odoo.sh** |
|---|---|---|
| ¿Quién corre el Upgrade Service? | Tú, vía CLI del kit (`iterate`/`run`) sobre el analyzer | odoo.sh nativo (botón **Upgrade** de la rama) |
| ¿Se pueden importar dumps a producción? | **Sí** — restauras la BD migrada+fixed donde quieras | **No** — odoo.sh no acepta dumps externos en prod |
| ¿Dónde se aplican los fixes del kit? | Pipeline SQL del kit (`run --apply`) sobre la BD migrada | **2 canales**: (a) `prod_live.sql` a mano en el Shell de prod **antes** del backup automático; (b) módulo **glue** con `migrations/<ver>/{pre,post,end}-migration.py` que corre en el `-u` |
| `upgrade_aim` | `test` al iterar, **`production`** para go-live | **SIEMPRE `test`** (el go-live lo hace odoo.sh) |
| Cutover / go-live | Restaurar la BD migrada+fixed en el servidor productivo | Un **commit** en la rama de producción (sale del modo "Waiting for upgrade code to be merged" → dispara backup→upgrade→restore→`-u`) |
| ¿Estructura de repo? | Indiferente (addons-path local) | Puede ser monorepo o **submódulos**; si prod↔destino divergen → cutover con **`commit-tree`**, no merge/bump |
| ¿Ensayo del proceso? | El propio pipeline ya es reproducible | El **simulacro local** (restore v_prev → prod_live → upgrade test → restore → `-u`). El staging de odoo.sh **no** sirve para ensayar `prod_live` (su botón Upgrade coge copia fresca de prod) |
| Rollback | Restaurar dump previo | Backup manual pre-`prod_live` (FASE 0) en odoo.sh |

**Regla:** en on-premise el kit **ES** el vehículo de los fixes. En odoo.sh el vehículo es el **glue module + prod_live** (el kit no toca producción directamente; ver `references/post-upgrade-custom-fixes.md` patrón "glue como vehículo del kit en odoo.sh"). Flujo odoo.sh detallado: kit `docs/migrar-en-odoo-sh.md` + playbook `references/playbooks/go_live/odoosh_cutover_commit.md`.

## FASE 0.6 — Restore dump migrado

Leer `references/restore.md` para detalle del procedimiento.

1. Detectar tipo de dump (`upgrade_zip` / `single_custom` / `cluster` / `single_plain`) — el script soporta zip del Upgrade Service y lo extrae automáticamente
2. Restaurar en el analyzer compartido (`odoo-upgrade-analyzer-pgN`)
3. **`verify` NO borra BDs ajenas por defecto.** Si el analyzer es mono-cliente y quieres limpiar las demás, pasar `--clean-others`

> El **backup inmutable** (`backup/original.dump`) se crea en **FASE 1.5** una vez que `indaws-migrate init` ha creado el workdir `migration_kit_<cliente>/`. Hasta entonces no hay carpeta de destino para el backup.
