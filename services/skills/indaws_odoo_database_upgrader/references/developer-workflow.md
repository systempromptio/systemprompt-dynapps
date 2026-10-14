# Flujo del desarrollador — de migración recibida a "1-step reproducible"

Cómo trabajar en una migración real con la skill `odoo-database-upgrader` +
el motor `indaws_migration_kit`, de forma que cualquier dev (incluido tú
mismo dentro de 3 meses) pueda re-ejecutar el pipeline en un solo paso.

> **Premisa**: este flujo asume que ya tienes el resultado del Upgrade Service
> de Odoo (un `upgraded.zip` o `dump.sql` post-migración técnica). El
> trabajo del desarrollador empieza **después** de que la migración técnica
> entre versiones ya se completó. Lo que hace el dev es **cerrar la brecha
> funcional**: re-implementar módulos custom, restaurar UX v10, limpiar
> artefactos y dejar la BD operativa.

## Sub-flujos (lectura on-demand)

| Tema | Archivo |
|---|---|
| Auto-feedback (`suggest`, PlaybookHook) | `developer/auto-feedback.md` |
| Tracking de módulos (status, blockers, orden topológico) | `developer/module-tracking.md` |
| Entrega del workdir a otro dev | `developer/handoff.md` |
| Subir mejoras al kit o a la skill | `developer/extend-kit.md` |

## Regla de oro

**Todo descubrimiento debe acabar reflejado en `migration.yaml` o en `sql/`
o `python/` del workdir del cliente. Si lo arreglas a mano fuera del kit,
el siguiente dev tendrá que descubrirlo de nuevo.**

---

## Fase 0 — Setup inicial (1 vez por equipo)

Requisitos para el entorno del dev:

```bash
# 1. Repos código (instalar en el orden)
git clone github.com/indawsodoo/ai-claude-developer  # contiene la skill
git clone github.com/indawsodoo/indaws_migration_kit # motor Python

# 2. Mirrors locales (read-only)
$ODOO_SRC/odoo18                  # Odoo CE
$ODOO_ENTERPRISE_SRC/18           # Odoo EE
$OCA_SRC/v18                      # OCA modules
$PROJECTS_ROOT/indawsodoo/18       # Indaws Core

# 3. Setup venv del kit
cd $PROJECTS_ROOT/indaws_migration_kit
python3 -m venv venv && venv/bin/pip install -e .
venv/bin/playwright install chromium  # opcional, smoke UI
```

Si el cliente nuevo (o `cd` a su proyecto):

```bash
indaws-migrate init --client TuCliente --database tucliente_v18 \
    --pg-host localhost --pg-port 5432 ...
```

Esto crea `migration_kit_<cliente>/` con la estructura canónica:

```
migration_kit_<cliente>/      # versionable (git)
├── migration.yaml            # ← centro de gravedad de la migración
├── sql/                      # scripts SQL del cliente (sql/01_*.sql, ...)
├── python/                   # scripts Python ad-hoc del cliente
├── docs/                     # estudio, comparison, post-mortem
├── playbook.md               # narrativa viva de la migración
├── gates.json                # acks go-live
├── logs/                     # gitignored (credenciales, outputs)
├── iterations/               # gitignored (historia dry-run)
└── audit.sqlite              # gitignored (auto-gen por runner)

$DUMP_PATH/                   # FUERA del workdir (NAS/S3/path local)
├── original.dump             # dump inmutable, input del kit
└── to_upload.dump            # temporal, regenera cada iteración
```

> Los dumps NO viven en el workdir. Apunta `$DUMP_PATH` donde te interese
> (NAS, carpeta hermana, S3 montado), y pasa rutas absolutas al kit.

---

## Fase 1 — Primera pasada exploratoria

### 1.1. Restaurar el dump post-upgrade

```bash
docker compose up -d db
docker exec -i <client>_db psql -U odoo -c "CREATE DATABASE <db> OWNER odoo;"
docker exec -i <client>_db pg_restore -U odoo -d <db> < $DUMP_PATH/original.dump
```

### 1.2. Inicializar contexto del plan

```bash
# Detecta lo instalado en la BD restaurada y rellena migration.yaml
indaws-migrate detect
indaws-migrate refresh-context
```

### 1.3. Compilar pipeline inicial

```bash
indaws-migrate compile
indaws-migrate inspect --plan migration.yaml
```

`inspect` lista todos los steps que el runner emitiría. Antes de aplicar
nada, revisa que no aparezca:

- `studio_modules` con notas tipo "decisión consultor pendiente"
- Módulos con `successor: TBD`
- `manifest_preflight` con paths inexistentes

### 1.4. Primera ejecución dry-run

```bash
indaws-migrate run --plan migration.yaml          # SIN --apply
```

Revisa los logs en `logs/`. Cada step reporta `WARN`/`BLOCK` con SQL
sugerido que aplicaría en modo `--apply`.

---

## Fase 2.A — Port de código custom (paralelo al ciclo iterativo)

**Cuándo**: cualquier módulo custom v_anterior (v10, v15, v17...) que el
cliente quiera mantener (`KEEP`) o que sea sucesor de un `REPLACE_WITH`
necesita su código portado a v18 antes de que el kit lo encuentre.

**No es parte del pipeline del kit** (`indaws_migration_kit` asume que
`migrated_modules/` ya tiene los módulos v18 listos). Es trabajo
pre-pipeline del dev.

### Quién hace el port

Skill hermana **`odoo-module-upgrader`** (developer-skills). Se invoca
desde aquí cuando detectes:

- Módulo custom v10 que sigue siendo necesario funcionalmente
- Módulo OCA renombrado que no tiene equivalente directo v18
- Módulo Indaws Core con incompatibilidades

Cubre:
- Python: `@api.depends`, `_inherit`, signatures de métodos cambiadas
- XML: `attrs`/`states` → inline, `<tree>` → `<list>`, `customer` →
  `customer_rank`, etc.
- JavaScript/OWL: cambios de framework v17+
- SCSS/themes: variables Bootstrap renombradas
- `__manifest__.py`: `version` v18 + dependencias actualizadas

### Flujo concreto

```bash
# 1. Identificar módulo a portar (ej. desde estudio o detect)
# 2. Invocar skill `odoo-module-upgrader`
#    (en chat: "porta el módulo XYZ de v10 a v18")
# 3. La skill genera el módulo v18 en migrated_modules/<name>/
# 4. Smoke local: cd migrated_modules/<name> && (test manual o helper RPC)
# 5. Declarar en migration.yaml:
#    modules.<old_name>:
#      action: REPLACE_WITH
#      successor: <new_name>      # o KEEP si no reemplaza
# 6. (Si hay migración de datos legacy → nuevo campo):
#      data_sql: sql/NN_migrate_<feature>.sql
```

### Cuándo NO portar (alternativas)

| Alternativa | Cuándo |
|---|---|
| `NEUTRALIZE` (stub) | El módulo definía un campo que ya no se usa, pero hay datos en BD que conservar |
| `STUDIO` | La funcionalidad cabe en campos manuales + vistas configurables |
| `CORE_REPLACE` (Indaws Core) | Indaws Core tiene un módulo equivalente (`indaws_*`) |
| `REPLACE_WITH` OCA portado | OCA ya portó el módulo (con o sin rename) |
| `REMOVE` / `TO_DROP` | Sin uso real, sin valor de mantener |

Elige la opción según la **complejidad de la lógica**: para lo trivial, las alternativas de arriba (NEUTRALIZE / STUDIO / CORE_REPLACE / REPLACE_WITH / REMOVE) suelen bastar; para lógica no trivial, portar el módulo (código gobernado) es lo correcto. Con IA, portar ya no es la opción "cara" que era. Criterio en `odoo-classification-guide`.

### Estimación

Tiempo de port por módulo (con `odoo-module-upgrader`):
- Trivial (manifest + 1-2 archivos): 15-30 min
- Standard (modelo + vistas + python): 1-3 h
- Complejo (JS/OWL, reports, wizards): 4-8 h
- Con tests + validación: +30-50%

### Salida esperada

`migrated_modules/<name>/` listo para `auto_install` y consumible por el
kit. Tests propios opcionales pero recomendados (al menos un smoke
manual).

---

## Fase 2 — Iterar (el grueso del trabajo)

Aquí descubres los problemas reales. Tu trabajo es **convertir cada
descubrimiento en código declarativo del plan**.

### Ciclo iterativo

```
[1] indaws-migrate run --apply             # ejecuta lo que sabe el kit
[2] docker compose up -d odoo              # arranca Odoo
[3] docker exec X bash -c 'odoo -c ... -u all --stop-after-init'
[4] Abrir UI / smoke / probar flujos       # → encontrar problemas
[5] Diagnosticar 1 problema (root cause)
[6] Reflejar el fix en migration.yaml o sql/  ← CRÍTICO
[7] Volver al paso 1
```

> **Atajo**: si `iterate` falla con patrones conocidos, `indaws-migrate
> suggest --workdir . [--apply --commit]` propone/aplica fixes al yaml
> automáticamente. Ver `developer/auto-feedback.md`.

### Dónde reflejar cada tipo de fix

| Tipo de problema | Dónde declararlo |
|---|---|
| Módulo legacy → módulo sucesor | `modules.<name>: { action: REPLACE_WITH/CORE_REPLACE, successor: ... }` |
| Migración de datos legacy → nuevo campo | `modules.<name>.data_sql: sql/NN_*.sql` (válido para KEEP/REPLACE_WITH/CORE_REPLACE) |
| Campo Studio custom v10 → v18 | `modules.<name>.studio_spec.fields[*]` con `legacy_column` |
| Vista heredada inactiva tras upgrade | `fixes.post_restore_validator.reactivate_disabled_views: true` |
| Vista huérfana (sin xmlid) | `fixes.post_restore_validator.delete_unowned_views: true` |
| `ir.model.fields` apuntando a módulos uninstalled | `migrate_orphan_fields_to: {old_mod: new_mod}` |
| FK hacia tablas de módulos uninstalled | `drop_orphan_fks: true` |
| Whitespace en `<attribute name="groups">` filtrando elementos | `sanitize_attribute_whitespace: true` |
| Fix técnico v10→v18 no cubierto por el kit | `steps_custom: [{type: RunSqlFile, params: {file: sql/NN_*.sql}}]` |
| Módulo nuevo (auto_install) que crea funcionalidad v18 | crearlo en `migrated_modules/` + declarar en `modules: KEEP` |
| Bug en override Indaws/OCA detectado | Issue upstream + módulo custom de fix en `migrated_modules/` |

### Antipatrones (NO hacer)

- ❌ Aplicar fix vía RPC/UI sin reflejarlo en el plan
- ❌ Editar `migrated_modules/` sin commit ni declarar el módulo en `modules:`
- ❌ Borrar registros (`ir_model_data`, `ir_ui_view`) sin antes ver si hay
      un helper genérico (P5, P6, P7) que lo cubre
- ❌ Crear `sql/NN_*.sql` con valores hardcoded sin docstring del problema
- ❌ Saltarse `indaws-migrate inspect` antes de `run --apply`

### Validación de cada iteración

Tras `run --apply`, valida:

```bash
# Smoke RPC (capa 1+2)
indaws-migrate run --apply  # incluye smoke_rpc auto-inyectado si odoo_url

# Smoke UI exhaustivo (opcional, 5-6 min)
venv/bin/python migration_kit_<cliente>/python/playwright_smoke.py
```

Cualquier FAIL real → vuelta al paso 5 del ciclo.

---

## Fase 3 — Cierre "1-step"

Cuando el flujo iterativo converge (smoke verde, validación funcional
del consultor OK), el plan está listo para ejecutarse 1-step. **Checklist
final** antes de cerrar:

- [ ] `indaws-migrate inspect --plan migration.yaml` → sin issues
      estructurales
- [ ] Todos los módulos custom de `migrated_modules/` están declarados en
      `modules:` (no se confía en `auto_install=True` solo)
- [ ] Todos los SQL en `sql/` referenciados desde `migration.yaml` (en
      `data_sql:` o `steps_custom:`)
- [ ] Studio specs completos (no `studio_spec: null`)
- [ ] Memoria del proyecto actualizada (`~/.claude/projects/<cliente>/memory/`)
- [ ] `README_MIGRACION.md` con anexo de iteración (qué se hizo + casos
      reales detectados)
- [ ] Tests de los helpers nuevos pasando (en el kit, si fueron añadidos)

### Test de reproducibilidad (último gate)

Desde un dump limpio y otra máquina o BD fresca:

```bash
docker exec -i <client>_db pg_restore -U odoo -d <db> < $DUMP_PATH/original.dump
indaws-migrate run --apply
docker exec <client>_odoo bash -c 'odoo -c ... -u all --stop-after-init'
indaws-migrate inspect --plan migration.yaml  # estado final OK
```

Si esto deja la BD en el mismo estado que la tuya tras la iteración
manual, **la migración es 1-step reproducible**. La entrega completa
(qué viaja con el workdir, qué no, y comandos del dev receptor) se
documenta en `developer/handoff.md`.

---

## Resume parcial (re-ejecutar tras cambios pequeños)

`indaws-migrate run --resume` consulta `audit.sqlite` y salta steps con
`status=completed` con el mismo `params_hash`. Si cambias un param de un
step, se re-ejecuta ese step + dependientes downstream.

```bash
# Cambias `sanitize_attribute_whitespace: false → true` en migration.yaml
# Para re-aplicar solo lo afectado:
indaws-migrate run --apply --resume
```

---

## Skills hermanas que el flujo invoca

| Skill | Cuándo | Para qué |
|---|---|---|
| `common-skills:docker-deployment` | Fase 1.1 | Levantar el entorno local del cliente |
| `common-skills:odoo-pilot` | Fase 2.4 | Probar via RPC tras cada `-u all` |
| `odoo-module-upgrader` | Fase 2 (custom v_X → v18) | Port código Python/XML/JS de módulos custom |
| `odoo-studio-migration` | Fase 2 (custom → Studio) | Decidir qué módulos pasan a Studio |
| `common-skills:odoo-database-analyzer` | Fase 0 | Análisis pre-migración (módulos instalados, deuda, etc.) |
| `common-skills:migration-assistant` | Cualquier fase | Compañero conversacional para guiar por fases + gates |

---

## Memoria del proyecto

Mantener actualizado `~/.claude/projects/-opt-PROYECTS-<cliente>/memory/`:

- `MEMORY.md` — índice
- `migration-kit.md` — configuración del kit
- `excel-errores-pendientes.md` (o equivalente) — tracking del informe
  del consultor: qué se resolvió y qué quedó como decisión funcional
- `skill-odoo-database-upgrader-improvements.md` — mejoras al kit
  detectadas durante la migración (P1, P2, ...)

Cada vez que detectes un patrón **genérico** (no específico del cliente),
documéntalo aquí + plantéate si merece un helper en el kit. El flujo
completo de "subir mejora al kit vs queda en workdir vs amplía la skill"
está en `developer/extend-kit.md`.

---

## TL;DR — workflow en 1 frase

> Recibes dump → `restore` → `compile` → `inspect` → `run --apply` →
> iteras descubriendo issues y **reflejando cada uno en `migration.yaml`
> o `sql/`** → si un descubrimiento es genérico (afectará a otros
> clientes) **sube al kit con tests + cambia el workaround del cliente
> al helper** → cuando smoke verde y consultor OK → checklist de cierre
> → el siguiente dev ejecuta `run --apply` en ~15 min y obtiene tu estado.
