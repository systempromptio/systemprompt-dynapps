# GOTCHAS — `odoo-database-upgrader`

Patrones de fallo reales encontrados en migraciones. **Lee esto antes de cualquier tarea no trivial.**

## v18 cambios silenciosos que rompen SQL/templates

### 1. `arch_db` es `jsonb` en v18, era `text` antes

Operador `LIKE` no funciona en v18:
```sql
-- v15-17 (text)
UPDATE ir_ui_view SET arch_db = REPLACE(arch_db, 'a', 'b') WHERE arch_db LIKE '%a%';

-- v18 (jsonb)
UPDATE ir_ui_view
SET arch_db = (
    SELECT jsonb_object_agg(key, to_jsonb(REPLACE(value, 'a', 'b')))
    FROM jsonb_each_text(arch_db)
);
```

### 2. `account.account.code` movido a `code_store` jsonb en v18

Comparison SQL del kit usa `jsonb_extract_path_text(code_store::jsonb, 'es_ES')` automáticamente. Custom SQL del cliente que use `account_account.code` rompe silenciosamente.

### 3. `tax_totals` (dict) reemplaza `tax_totals_json` (str) en v18

Templates QWeb v15:
```xml
<t t-set="totals" t-value="json.loads(doc.tax_totals_json)"/>
```

Hay que reescribir a:
```xml
<t t-call="sale.document_tax_totals">
    <t t-set="currency" t-value="doc.currency_id"/>
</t>
```

### 4. `_search()` ya no acepta `access_rights_uid` en v18

Custom code v15:
```python
def _search(self, args, offset=0, limit=None, order=None, count=False,
            access_rights_uid=None):
```

v18:
```python
def _search(self, domain, offset=0, limit=None, order=None):
```

## Decisiones que cuestan caro

### 5. `aim=production` solo se usa **1 vez por contrato**

- Si `target_deployment = odoo.sh` → SIEMPRE `aim=test`. **Nunca `production` localmente.**
- Si `target_deployment = indaws | custom` → `aim=test` en iteraciones, `production` solo el día del go-live real

### 6. `migrated_modules/` NO va dentro de `migration_kit_*/`

`docker-compose.yml` monta `./migrated_modules` y `./stubs` como addons-path. Si están dentro del workdir oculto del kit, Odoo no los carga.

```
$PROJECTS_ROOT/Cliente_X/
├── docker-compose.yml
├── migrated_modules/        ← addons-path ✅
├── stubs/                   ← addons-path ✅
└── migration_kit_cliente_x/ ← workdir kit (NO contiene addons)
```

### 7. `Studio.validate_with_odoo: false` no detecta vistas rotas

El reactivate "basic" solo cambia `active=true` en SQL. No prueba render. Resultado: vistas con XPath roto siguen reactivadas y rompen en UI.

**Siempre usar `validate_with_odoo: true`** cuando hay container Odoo arriba para que `get_view()` valide.

**Reactivación de vistas: NUNCA bulk, SIEMPRE curado.** Un `UPDATE ir_ui_view SET active=true` masivo reactiva vistas que estaban inactivas LEGÍTIMAMENTE (p.ej. "Template Header" de `website`) → HTTP500 al arrancar el frontend. La reactivación se hace SOLO sobre una lista de ids previamente validados (uno a uno, render OK). Ver gotcha #30.

## Errores históricos de Pharmactive

### 8. `stock_quant_manual_assign` neutralizado por error

OCA presente en `migrated_modules/` que `NeutralizeOcaModule` neutralizó por accidente (no comprobaba el path). **Fix aplicado:** ahora `NeutralizeOcaModule` chequea `migrated_modules_path` antes de neutralizar. Sigue siendo gotcha: confirmar que el helper tiene el path correcto del cliente.

### 9. `account_statement_import` y su renombrado v18

Renombrado a `account_statement_import_file` (catálogo OCA). Pero el sucesor también fue **discontinuado** después. Decisión real: NEUTRALIZE legacy + no instalar nada.

### 10. Borrar `ir_model_data` de módulos uninstalled

**NUNCA borrar sin preguntar.** Causa: si el módulo se reinstala, falla porque sus xml_ids ya no existen y los registros físicos quedaron huérfanos. Usar `PreserveOrphanXmlid` en lugar de DELETE crudo.

## Convenciones del kit

### 11. `pg_dump -Fc` obligatorio para Upgrade Service

`.sql` plain y `-Fd` directory format **fallan en upload**. Solo custom format (`-Fc`).

### 12. `backup/original.dump` es inmutable

Una vez restaurado, **nunca modificar**. Si necesitas resetear, usa una copia. El backup es la referencia de "estado pre-migración".

### 13. Solo un fichero a la vez en `dumps/`

Borrar el anterior antes de generar el nuevo. El Upgrade Service no acepta múltiples uploads simultáneos.

### 14. Idempotencia obligatoria en fixes SQL

Todo SQL en `references/fixes/` o `<cliente>/sql/` debe poder re-ejecutarse sin romper. Usar:
```sql
ADD COLUMN IF NOT EXISTS ...
INSERT ... ON CONFLICT DO NOTHING
DO $$ BEGIN ... EXCEPTION WHEN duplicate_object THEN NULL; END $$;
```

## Gotchas descubiertos en vivo (Tecniman 2026-05)

Patrones reales que el `PostRestoreValidator` ahora detecta y repara automáticamente:

### 15. Crones inactivos huérfanos bloquean upgrade (P1)
`ir_cron` con `active=false` cuyo `ir_actions_server_id` apunta a una server_action de módulo `to upgrade` → al refrescar la action, peta `ir_cron_ir_actions_server_id_fkey`.
**Fix automático:** DELETE de crones inactivos huérfanos vía `validate_data` del validator.

### 16. company_dependent integer/varchar debe ser jsonb (P3)
Generalización del gotcha #1: cualquier columna `company_dependent` en `ir_model_fields` con tipo distinto a `jsonb` rompe `_auto_init` con `cannot cast type X to jsonb`.
**Fix automático:** registry estático + detección dinámica via `ir_model_fields.company_dependent=true`. Template del fix incluye `DROP CONSTRAINT IF EXISTS` para FKs previas.

### 17. Vistas cross-state to_upgrade↔installed (P4)
Hijo en módulo `to upgrade/uninstalled` con `inherit_id` apuntando a vista de módulo `installed` (o viceversa). El upgrade no puede borrar el padre por `ir_ui_view_inherit_id_fkey`.
**Fix automático:** ALTER FK → `ON DELETE CASCADE` (Odoo la regenera en `-u all`) + DELETE de las vistas extension huérfanas.

### 18. arch_db con patrones v10 obsoletos tras `-u all` (V1)
Vistas custom que sobrevivieron al upgrade pueden contener `attrs="{...}"`, `<tree>`, `<field name="customer">`, `states="..."`. Odoo no las recompila si su xml_id no se reprocesa.
**Detección:** `view_arch_validator` (capa 1 SQL + capa 2 `_check_xml` por JSON-RPC).
**Auto-fix:** `view_arch_fixer` delega a script `transform_arch.py` de la skill `odoo-module-upgrader`.

## Gotchas descubiertos validando iterate end-to-end (Tecniman 2026-05-26)

Estos surgieron al ejecutar el `iterate` COMPLETO desde el dump original (no solo dry-run). Son la diferencia entre "el kit compila" y "el kit reproduce la BD".

### 19. `iterate` necesita 3 fases, no 2 (update-list entre medias)
El flujo viejo `restore → apply(pre) → -u all → apply(post)` NO instala los módulos NUEVOS (sucesores de migrated_modules). `SuccessorMarker` no los encuentra en `ir_module_module` porque sus manifests nunca se registraron.
**Fix:** flujo de 4 fases — `restore → apply(pre_sql) → update-list → apply(pre_markers) → -u all → apply(post)`. `Step.phase` ahora soporta `pre | pre_markers | post`. Los markers/neutralize/replace_with/core_replace/studio van en `pre_markers` (post-registro).

### 20. Odoo 18 NO tiene `--update-list` en CLI server
`odoo --update-list` → `error: no such option`. La forma soportada de forzar `ir.module.module.update_list()` es `odoo -u base --stop-after-init --no-http` (re-upgrade de base lo dispara como efecto secundario). El kit lo hace en `run_update_list` de `iterate.py`.

### 21. KEEP que son módulos NUEVOS quedan uninstalled
Muchos KEEP son módulos creados PARA la migración (tecniman_partner, splits, etc.) que NO están installed en el dump. `-u all` solo upgradea installed → nunca los instala.
**Fix:** compiler emite `mark_keep_modules_to_install` (SuccessorMarker sobre todos los KEEP, skipea los ya installed).

### 22. NEUTRALIZE: el stub no basta, hace falta force-uninstall
El stub vacío en `stubs/` NO sustituye al OCA original si su código sigue en addons_path (orden de `addons_path` no garantizado aunque `ODOO_PRIORITY_ADDONS_PATH` apunte a stubs). Odoo carga el OCA real y revienta por vistas heredadas.
**Fix:** `NeutralizeOcaModule` ahora hace cleanup completo (4º sub-step): desactivar vistas + DELETE ir_model_data + state='uninstalled'. La lista de módulos a force-uninstall del cliente va en el SQL prep (PASO 2b Tecniman).

### 23. Módulos `installed` sin código rompen el `-u all` en staging
Tras Upgrade Service, módulos OCA/CE quedan `installed` pero su código no está en el addons-path del destino. Casos Tecniman:
- `web_responsive` / `web_favicon`: conflicto con Enterprise (`apps_menu_search_type`, `_get_favicon`).
- `account_add_gln`: módulo CE añadido 2025-12-02; imágenes anteriores no lo tienen (`global_location_number`).
**Fix:** cleanup completo (vistas + xmlids + state) en SQL prep. Identificar con: `SELECT m.name FROM ir_module_module m JOIN ir_model_fields f ON f.module=m.name WHERE f.name='COLUMNA_FALLIDA'`.

### 24. Campos compute non-store en res.partner + orden de carga
OCA como `mail_tracking` (`tracking_emails_count`), `partner_sector` (`sector_id`), `product_supplierinfo_discount` añaden campos compute non-store a res.partner. Si un módulo que extiende `view_partner_form` (ej. `web_map` Enterprise, `partner_external_map`) se carga ANTES → ParseError "campo X no existe". El orden no está garantizado porque no declaran depends.
**Fix:** NEUTRALIZE de los módulos que aportan los campos (si el cliente no los usa) o de los que rompen la vista. Cadena recursiva: cada uno revela el siguiente.

### 25. FKs legacy del dump rompen `check_foreign_keys` al instalar sucesores
Al instalar un sucesor, Odoo valida FKs. Columnas del dump con IDs viejos fallan:
- `res_partner.zip_id` → `res_city_zip` (recreado de 0 por base_location).
- `sale_order.type_id_hidden` → `indaws_sale_order_type` (IDs OCA viejos ≠ IDs Indaws).
**Fix:** `UPDATE ... SET col = NULL` en SQL prep (PASO 2d). El step post (`remap_type_id_hidden_values`) repobla desde `order_type_id`.

### 26. `res_partner.sale_type` es jsonb (company_dependent) en v18
Migrar `sale_type` (OCA sale_order_type) → `order_type_id` falla con `operator does not exist: jsonb = integer`. En v18 los campos company_dependent se almacenan jsonb `{'<cid>': value}`.
**Fix:** `(rp.sale_type->>'<company_id>')::int = oca_id` en el SQL del helper `sale_order_type_to_indaws`.

### Patrón de velocidad: clone TEMPLATE en vez de pg_restore
Para iterar rápido (debugging del pipeline), restaurar una vez a `<db>_pristine` y clonar con `CREATE DATABASE <db> TEMPLATE <db>_pristine` (~30s vs ~14min de pg_restore). Pre-poblar `iterations/.dump_sha` con el sha del dump para que `iterate` skipee el restore. Ver `clone_pristine.sh` del workdir Tecniman.

### on_verify_fail: continue en post_restore_validator
`PostRestoreValidator` puede detectar issues "naturalmente irreparables" (vistas extension de módulos uninstalled con padre installed, menús huérfanos). Con `on_verify_fail: continue` devuelve warning en vez de abortar — necesario para que el pipeline complete cuando quedan residuos no críticos.

## Gotchas descubiertos en odoo.sh con submódulos (Pharmactive 2026-06)

Setup real: prod (`master`) = MONOREPO (custom inline + `OCA/` submódulos v15); target (`staging18`) = SUBMÓDULOS (`addons-pharmactive` con todo el custom + `OCA_standalone/*` submódulos OCA 18.0). Top-level: 54 entradas vs 7.

### 27. Estructura de repo divergente prod↔target (commit-tree, no merge)
El "commit que dispara el upgrade" en odoo.sh debe avanzar `master`, pero `master` y `target` tienen estructuras top-level incompatibles (monorepo vs submódulos). `git merge target→master` da conflictos masivos y un árbol mezclado. **Solución:** sustituir el árbol de `master` por el de `target` con `git commit-tree`, conservando la historia de prod como primer padre:
```bash
TREE=$(git rev-parse origin/<target>^{tree})
NEW=$(git commit-tree "$TREE" -p origin/master -p origin/<target> -m "[MIGRATION] master -> v18 (= <target> validado)")
git diff --stat "$NEW" origin/<target>   # DEBE estar VACÍO (árbol idéntico)
[ "$(git rev-parse $NEW^1)" = "$(git rev-parse origin/master)" ] && echo OK  # avance directo
git push origin "$NEW":master            # PUNTO DE NO RETORNO: dispara backup→upgrade→restore→-u
```
Resultado: `master` queda byte a byte = target validado, sin conflictos. Receta completa en `playbooks/go_live/odoosh_cutover_commit.md`.

### 28. SHA desplegado ≠ SHA validado (GATE obligatorio)
La rama staging/prod puede apuntar al submódulo (`addons-pharmactive`) en un commit ANTERIOR (p.ej. `69e742e`, glue con bug HTTP500) mientras lo validado en el simulacro local es OTRO commit (p.ej. `4c5069c`, con el fix). Si no se cruza, se despliega la versión buggy.
**GATE pre-cutover:** el SHA del submódulo en la rama que se va a desplegar DEBE == el SHA marcado como validado. Verificar con `git ls-tree <rama> <ruta-submódulo>` y comparar contra el sha del simulacro. No avanzar `master` sin este check en verde.

### 29. OCA all-or-nothing en submódulos (todos en la versión destino)
En un setup de submódulos, TODOS los repos OCA deben subir a la rama de la versión DESTINO (`18.0`). No se puede dejar ningún OCA en versión origen: no arranca en la versión nueva. No es "cambio mínimo" ni opcional — es requisito de build. Verificar rama de cada submódulo OCA antes del cutover.

### 30. Reactivación de vistas en bloque = HTTP500 (refuerzo del #7)
`UPDATE ir_ui_view SET active=true` masivo reactivó las "Template Header" de `website` (inactivas a propósito) → HTTP500 al arrancar el frontend. **Regla:** reactivación SIEMPRE curada (lista de ids validados, render comprobado), NUNCA bulk. Aplica tanto en SQL prep como en los `migrate` scripts del glue (la reactivación de Studio del kit es curada por diseño).

---

**Para detalle adicional por version-pair**, ver:
- `gotchas/v10_v18_migration.md` — 10 gotchas G1-G10 documentados con SQL fix-recipes.
