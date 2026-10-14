# Helpers del motor `indaws_migration_kit`

Catálogo completo de helpers invocables como `fixes` o `steps_custom` desde
`migration.yaml`. El compiler auto-emite la mayoría a partir de
`modules` + `context`; los que requieren parámetros específicos van como
`fixes:` o `steps_custom:` explícitos.

> **Convención**: `id` = identificador único en el pipeline. `type` = ruta
> Python al MigrationStep. Todos son idempotentes y compatibles con
> `runner.resume` (saltar steps ya completados).

## 1. Validation (5)

### `manifest_preflight` — `helpers.validation.manifest_preflight.ManifestPreflightValidator`

Pre-flight scan del addons_path detectando manifests con `version` inválida
que romperían el entrypoint Odoo (HTTP 500 en `/web/webclient/load_menus`,
`/web/assets/*`, `/web/webclient/translations`).

**Auto-inyectado** al inicio si `context.source_repos` no vacío.

Caso real (Tecniman 2026-05): OCA `maintainer-quality-tools/tests/test_repo*/`
con versions `18.0.0.1.0.0` (4 partes tras `18.0.`) rompía la UI completa.

Params:
- `paths: list[str]` — paths a escanear
- `apply: bool = False` — si True, renombra manifests bad a `.disabled`
- `exclude: list[str]` — substrings a excluir
- `max_depth: int = 4`

### `smoke_rpc` — `helpers.validation.smoke_rpc.SmokeRpcValidator`

Smoke test post-deploy en 2 capas vía JSON-RPC:
1. **Read profundo**: por cada modelo crítico, lee el primer registro con
   TODOS los campos del form view (ejerce computes, One2many con `order=`).
2. **Create draft + read + unlink**: account.move ×4 move_types,
   sale.order, purchase.order, res.partner (ejerce defaults + create
   override + `_compute_needed_terms`).

**Auto-inyectado** al final si `plan.odoo_url` definido.

Detecta casos reales: `apply_payment_days` firma rota, campo Studio
referenciado pero inexistente, override compute incompatible.

Params:
- `models: list[str]` — amplía `CRITICAL_MODELS` (40+ por defecto)
- `drafts: list[dict]` — amplía `DRAFT_SPECS` (7 por defecto)
- `auto_discover: bool = True` — añade top-N modelos por nº de registros
- `top_n: int = 30`, `min_records_for_discovery: int = 100`

### `playwright_smoke` — `helpers.validation.playwright_smoke.PlaywrightSmokeValidator`

Smoke UI headless (Chromium) navegando `/odoo/action-<id>` por cada menú con
acción. Captura: console errors, JS errors, HTTP 5xx, RPC errors en
`/web/dataset/call_kw`.

Clasificador de noise integrado (descarta `Unexpected token '<' "<!doctype"`,
`/web/image/*` cuando no hay filestore, `beforeunload`, marketplace 404).

Params:
- `url, db, user, password` — sobreescriben `plan.*` por defecto
- `limit: int = 0` — 0 = todos los menús
- `timeout_secs: int = 15`
- `filestore_available: bool = False` — si False, `/web/image 500` = WARN

Requiere `playwright` + `playwright install chromium`. `precondition` salta
silenciosamente si no está disponible.

### `odoo_readiness` — `helpers.validation.odoo_readiness.OdooReadinessProbe`

Polling HTTP a `<odoo_url>/web/login` hasta 2xx/3xx. Usado como gate antes
de `smoke_rpc`/`playwright_smoke` tras restart de contenedor.

Params: `url`, `endpoint='/web/login'`, `timeout: int = 120`,
`interval: float = 2.0`, `expected_status=[200, 303]`.

### `override_linter` — `helpers.validation.override_linter.OverrideCompatibilityLinter`

AST scan de overrides Python con firma incompatible entre módulos hermanos.
Detecta caso real Tecniman: `indaws_account.apply_payment_days(self, days, date)`
vs OCA `account_payment_term_extension.apply_payment_days(self, line, date)`.

Heurística: para cada `(model, method)` con ≥2 implementaciones, compara
nº de parámetros posicionales + presencia de `*args`/`**kwargs`.

Params: `paths`, `target_model_filter`, `severity_threshold`, `exclude`.

## 2. Odoo lifecycle (6)

### `successor_marker` — `helpers.odoo.successor_marker.SuccessorMarker`

Marca módulos como `to install` en `ir_module_module` para que el próximo
`-u all` los instale.

**Auto-emitido** por el compiler antes de `replace_with_successors` y
`core_replace_modules` con la lista de sucesores extraída de `modules.action=REPLACE_WITH/CORE_REPLACE`.

Params: `modules: list[str]`, `apply: bool = True`,
`fail_if_uninstallable: bool = False`.

### `replace_with_successors` / `core_replace_modules` — `ReplaceWithSuccessor`

Archive legacy + opcional install sucesor multi-source. Mismo helper, distinta
etiqueta semántica para el log.

Param clave **M17** (Tecniman 2026-05): `finalize_uninstall: "auto"|"force"|"never"`:
- `"auto"` (default REPLACE_WITH): uninstall si el legacy NO tiene código en
  addons_path. Caso típico NEUTRALIZE→stub: preservar.
- `"force"` (default CORE_REPLACE): uninstall siempre, aunque el legacy tenga
  código. Imprescindible para CORE_REPLACE puro (OCA legacy + sucesor Indaws Core),
  o queda duplicado.
- `"never"`: solo archive.

Compat: `True` → `"auto"`, `False` → `"never"`.

### `post_restore_validator` — `helpers.odoo.post_restore_validator.PostRestoreValidator`

**18+ patrones** de saneamiento BD post-upgrade (auto-fix opcional):
P1 (ir_cron FK), P2 (zip_id ghost), P3 (jsonb company_dependent), P4
(`ir_ui_view_inherit_id_fkey`), **P5 unowned_views** (Tecniman 2026-05),
**P6 disabled_inherited_views** (vistas oficiales OCA/Odoo/Indaws
inactivas tras Upgrade Service — invisible en UI pese a estar todo
"correcto"), **P7 orphan_ir_model_fields** (Tecniman 2026-05-18:
`ir.model.fields` apuntando a módulos uninstalled o stub bloquean
re-definición en sucesores), **P8 whitespace_in_groups_attribute**
(Tecniman 2026-05-18: bug Odoo v18 al evaluar `<attribute name="groups">`
con whitespace alrededor del valor — filtra elementos cuyo user SÍ tiene
el grupo) + 10 más (phantom modules, legacy artifacts, AEAT dates,
customer→customer_rank, asset cache, phantom menus, obsolete models,
FK orphans, etc.).

Params clave:
- **P5** `delete_unowned_views: bool = False` (Studio puede crear vistas
  legítimas sin xmlid).
- **P6** `reactivate_disabled_views: bool = False` (algunas vistas se
  desactivan legítimamente — caso "Neutralize Banner").
- **P7** `migrate_orphan_fields_to: {old_module: new_module}` — migra
  `ir_model_data` al nuevo módulo (preserva campo) o sin mapeo: DELETE.
- **P8** `sanitize_attribute_whitespace: bool = False` — limpia
  `<attribute name="groups">\n  X\n  </attribute>` que Odoo v18 no
  evalúa correctamente. Caso real Tecniman: pestaña Accounting invisible
  pese a admin en el grupo correcto.
- **FK** `drop_orphan_fks: bool = False` — DROP CONSTRAINT de FKs hacia
  tablas de módulos uninstalled (post-CORE_REPLACE con campos related
  store=True).

**Caso real Tecniman 2026-05-15** (P6): 22 vistas OCA/Odoo/Indaws estaban
`active=FALSE` en v18 pese a estar `active=TRUE` en v10 (verificado con
diff v10↔v18). `arch_updated=FALSE` en todas → no eran custom. Afectadas:
`account_credit_control` (followup), `account_payment_order` (5 vistas),
`account_due_list`, `account_chart_update` wizard, `auditlog` (4 vistas),
`mis_builder`/`mis_builder_budget`, `board` (My Dashboard),
`l10n_es_aeat` (Compare BOE), `l10n_es_aeat_mod349`,
`account_tax_balance` (2 vistas), `account_invoice_supplierinfo_update`,
`mrp_bom_current_stock`, `indaws_stock_valued_picking`. Reactivar en
batch + smoke `get_views` por modelo → 0 fallos.

Recomendación: **NO** activar `reactivate_disabled_views` en bloque (riesgo
HTTP500 al reactivar banners/website legítimamente inactivos, caso
Pharmactive). P6 queda como diagnóstico WARN. La reactivación va siempre
CURADA vía `reactivate_root_qweb_by_module` (P9) o el
`end-reactivate_views.py` del track de código.

**B1 — `deactivated_views::<categoría>` (ZNU-720, gate duro, GLOBAL).** Además
de P6, la fase `menus` clasifica **TODAS** las vistas de módulos installed que
quedaron `active=FALSE` tras el Upgrade Service, por **dueño** (no por un prefijo
fijo — no todos los proyectos usan `xx_`):
- **custom** (inventario `custom_modules` del migration.yaml / `indaws-migrate
  modules list`, + prefijos + heurística de autor) → **BLOCK**: regresión casi
  segura; el `-u` no las reactiva (~120 en ZNU-720).
- **studio** (`studio_customization`) → **BLOCK**: el usuario final vería
  funcionalidad Studio desaparecida.
- **oca / core / vendor** → **WARN**: se reportan SIEMPRE (con lista y módulos)
  para revisión curada, para que el usuario final no descubra vistas base/OCA que
  faltan. Las inactivas legítimas (website/banner/snippet/header/footer) se eximen
  solo en estas 3 categorías.

Params: `custom_modules` (inventario, vía global), `custom_view_module_prefixes`
(fallback, def `['xx_']`), `deactivated_view_severity` (override por categoría),
`legit_inactive_view_patterns`. No auto-repara: el fix es siempre CURADO vía P9
`reactivate_root_qweb_by_module` / `end-reactivate_views.py`.

**B2 — fase `integrity` / `required_field_integrity` (ZNU-727, gate duro).**
Nueva sub-fase que detecta campos **`required` vaciados** (`''` o NULL) por el
Upgrade Service cuando el registro referenciaba un campo que ya no carga (custom
ausente en core+EE). Vaciar un required rompe el flujo que lo usa **sin error en
el `-u`**. Registry por defecto: los required estándar de `ir.filters`
(`domain`/`context`/`sort`) con `restore_default` auto-reparables en `apply=True`.
Ampliable por cliente vía param `required_field_integrity` para campos custom como
`ir.filters.partner_domain` (report-only → restaurar con post-migrate dedicado,
que es como se resolvió ZNU-727).

Params adicionales: `apply: bool = False`, `phases: list[str]`,
registries extensibles vía `params.*_registry`.

### `drop_modules` — `helpers.odoo.drop_module.DropModules`

TO_DROP completo: pre-checks (deps + FKs + M2M huérfanas) + uninstall +
DROP TABLE. `confirm_destructive: bool = False` por defecto.

### `view_arch_validator` (capa 1+2) — `helpers.odoo.view_arch_validator.ViewArchValidator`

Detecta vistas v10 obsoletas (SQL estático + `_check_xml` dinámico). Modes:
`report_only`, `deactivate`, `delete_orphan`.

### `view_arch_fixer` (capa 3) — `helpers.odoo.view_arch_fixer.ViewArchFixer`

Auto-fix `arch_db` delegando a la skill `odoo-module-upgrader` (transforms:
attrs→inline, states→inline, tree→list, customer/supplier→_rank).

### `neutralize_extras` — `helpers.odoo.neutralize_extras.NeutralizeExtras`

Completa la neutralización tras `odoo-bin neutralize`. Cubre lo que el
neutralize estándar de Odoo v17+ NO cubre:

- Reset password de todos los users a un valor común (default `admin`)
- Limpiar TOTP secrets (2FA)
- Regenerar `database.uuid` (evita colisiones con prod en IAP)
- Regenerar `database.secret`

Idempotente. Params: `apply: bool`, `reset_passwords_to: str`,
`clear_totp: bool`, `regenerate_database_uuid: bool`,
`regenerate_database_secret: bool`, `exclude_users: list[int]`.

Caso real (origen Pedak v16→v19, migration-playbook compañero):
patrón estable para BDs locales de desarrollo. 9 tests específicos.

## 3. Studio (3)

### `studio_modules` — `helpers.studio.studio_marker.StudioMigrationMarker`

Genera log + opcional creación de campos manuales + vistas heredadas.
Soporta `studio_spec` por módulo con `fields` y `views`.

Param clave **Gap 4** (Tecniman 2026-05): `mode: "auto"|"rpc_direct"|"sdk"`:
- `"auto"` (default): si `ctx.odoo` accesible → `rpc_direct`, si no → `sdk`.
- `"rpc_direct"`: usa `ctx.odoo.execute_kw` para `ir.model.fields.create`
  (state=manual) + `ir.ui.view.create` + `ir.model.data.create`. Sin
  dependencia del SDK Studio externo.
- `"sdk"`: delega a `BatchStudioFieldCreator` (requiere `studio_client` con
  protocolo Studio).

### `legacy_to_studio_copy` — `helpers.studio.legacy_to_studio_copy.LegacyToStudioCopy`

UPDATE `legacy_column` → `studio_column` por cada field declarado. Idempotente
con `WHERE legacy IS NOT NULL AND studio IS DISTINCT FROM legacy`.

**Auto-emitido** por el compiler tras `studio_modules`, poblando `fields`
desde `studio_modules[*].studio_spec.fields[*]` con `legacy_column` no nulo.

Caso real Tecniman: 33 144 `supplier_default` + 10 `label_tax` rescatados
de campos legacy a x_studio_* tras crearlos.

### `batch_field_creator` — `helpers.studio.batch_field_creator.BatchStudioFieldCreator`

Patrón abstracto sobre protocolo Studio (`studio_client`). Usado por
`studio_modules` modo `sdk`. Casi nunca invocado directamente.

## 4. Data / Spain (2)

### `field_renames_v10_v18` — `helpers.data.field_mapper.FieldRenameStep`

17 renames v10→v18 (customer→customer_rank, etc.) desde registry.

### `aeat_map_tax` — `helpers.spain.aeat.AeatMapTaxFix`

Cierra `l10n_es_aeat_map_tax` abiertos/parciales antes de `-u all`.
**Auto-inyectado** si `context.countries` contains `'ES'`.

## 5. Execution (3)

### `run_sql_file` — `helpers.sql.run_file.RunSqlFile`

Ejecuta fichero `.sql` arbitrario del workdir. **Auto-emitido** por el compiler
por cada módulo REPLACE_WITH/CORE_REPLACE con `data_sql` declarado.

Caso real Tecniman: `sql/02_migrate_sale_order_type_data_v2.sql` migrando
167 452 SOs de `type_id` (OCA) → `order_type_id` (Indaws Core).

### `run_script` — `helpers.exec.run_script.RunScript`

Invoca scripts Python del cliente en `<workdir>/python/` o `steps/`.

### `filestore_extractor` — `helpers.exec.filestore_extractor.FilestoreExtractor`

Extrae `filestore/` del zip del Upgrade Service al volumen Docker
(`/var/lib/odoo/filestore/<db>/`). Soporta zips con/sin `db_subdir` (prepend
auto). Idempotente con `overwrite: bool = False`.

Params: `zip_path`, `target_dir`, `db_name`, `chown: str | None`.

## 6. Orchestration (1)

### `phased_group` + `pause` — `helpers.orchestration.*`

Orquestación con pausas manuales (Studio ZIP import, decisiones consultor, etc.).

## 7. Reporting (1)

### `migration_report` — `helpers.reporting.migration_report.MigrationReportGenerator`

Autogenera `docs/iteration_YYYY-MM-DD.md` con métricas (`ir_module_module`
counts), historial de audit, warnings detectados. Opcionalmente appended al
README del cliente con `append_to_readme: 'README.md'`.

## Modelo de extensión declarable

Dos campos en `ModuleDecision` (yaml) habilitan extensión sin tocar el compiler:

- `studio_spec: {fields: [{model, name, ttype, label, legacy_column?}],
                  views: [{parent, xml_id, model, arch | modifications}]}`
  Soportado solo para `action: STUDIO`. Si presente, el compiler emite
  `studio_modules` (rpc_direct) + `legacy_to_studio_copy` automáticamente.

- `data_sql: str` (path relativo a workdir) — soportado para `REPLACE_WITH`
  y `CORE_REPLACE`. El compiler emite `RunSqlFile` tras el archive del
  legacy con ese fichero.

Ambos campos son opcionales; sin ellos el flujo es el clásico.
