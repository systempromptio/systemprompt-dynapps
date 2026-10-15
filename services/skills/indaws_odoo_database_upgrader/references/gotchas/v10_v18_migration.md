# Gotchas conocidos — migración Odoo v10 → v18

Patrones reales que han bloqueado migraciones anteriores. Cuando aparezca un síntoma equivalente, aplicar el fix documentado aquí antes de seguir investigando.

Fuente: migración Tecniman 2026-02→2026-05 (Brenda) + observaciones de iteraciones del Upgrade Service.

---

## G1 · `cannot cast type X to jsonb` al arrancar Odoo

**Síntoma:** Odoo cae al cargar el registry con error de cast a `jsonb`.

**Causa:** un campo `company_dependent` quedó como tipo nativo (boolean/integer) cuando v18 espera `jsonb`.

**Fix idempotente:**
```sql
ALTER TABLE <tabla> RENAME COLUMN <columna> TO <columna>_old;
ALTER TABLE <tabla> ADD COLUMN <columna> jsonb;
UPDATE <tabla> SET <columna> = jsonb_build_object('1', <columna>_old) WHERE <columna>_old IS NOT NULL;
```

Helper: `helpers/columns.JsonbConvert` (ya en kit).

---

## G2 · `psycopg2.errors.UndefinedColumn` en login

**Síntoma:** error 500 al hacer login con `column res_partner.<x> does not exist`.

**Causa:** modelo Python declara campo nuevo pero la BD no tiene la columna (falta `-u` del módulo que lo añade).

**Fix:** desde shell (la UI está rota):
```bash
odoo -d <bd> -u <modulo> --stop-after-init
```

---

## G3 · `ForeignKeyViolation` en `account_account_tag_account_move_line_rel`

**Síntoma:** upgrade de módulo falla `add_foreign_key` con `ForeignKeyViolation`.

**Causa:** scripts SQL que usan `session_replication_role='replica'` desactivan `ON DELETE CASCADE` y dejan filas M2M de `tax_tag_ids` huérfanas.

**Fix:**
```sql
DELETE FROM account_account_tag_account_move_line_rel r
WHERE NOT EXISTS (
  SELECT 1 FROM account_move_line aml WHERE aml.id = r.account_move_line_id
);
```

**IMPORTANTE:** usar `NOT EXISTS`, NUNCA `NOT IN`. Sin índice en la subquery, `NOT IN` tarda >12 min vs <1 s con `NOT EXISTS`.

Aplicable a cualquier M2M huérfana post-migrate.

---

## G4 · `equity_unaffected` rompe UNIQUE constraint

**Síntoma:** paso de reclasificación de cuentas falla con `duplicate key value violates unique constraint`.

**Causa:** mal mapeo de tipos de cuenta. Solo `129` debe ser `equity_unaffected`, no `120-121`.

**Fix:** consolidar tipos de cuenta `100-119/120-121/130-139` en `equity` y dejar solo `129` como `equity_unaffected`.

---

## G5 · `auth.sh` rompe `eval`

**Síntoma:** `eval "$(./scripts/auth.sh)"` da errores de sintaxis.

**Causa:** scripts de autenticación escriben mensajes informativos a stderr.

**Fix:** redirigir stderr:
```bash
eval "$(./scripts/auth.sh 2>/dev/null)"
```

---

## G6 · Reiniciar Odoo tras eliminar campos zombie

**Síntoma:** tras borrar campos de `ir_model_fields` siguen apareciendo errores.

**Causa:** el registry de Odoo cachea los campos al arrancar.

**Fix:** **siempre reiniciar** el proceso Odoo (no es suficiente con `-u`).

---

## G7 · Vista heredada de stub bloquea instalación de módulo

**Síntoma:** al instalar un módulo, falla con `El campo X no existe`.

**Causa:** una vista heredada de un stub (módulo NEUTRALIZE) referencia un campo que ya no existe en el ORM.

**Fix:** localizar y desactivar la vista:
```sql
UPDATE ir_ui_view SET active = false WHERE id IN (<id_vista>);
```

Detectable con:
```sql
SELECT v.id, v.name, v.model FROM ir_ui_view v
JOIN ir_module_module m ON m.name = split_part(v.xml_id, '.', 1)
WHERE m.state IN ('uninstalled', 'to remove');
```

---

## G8 · `--init` vs `--update` en módulos nuevos

**Síntoma:** al añadir un módulo custom nuevo, las columnas no aparecen en BD.

**Causa:** recargar el código Python (e.g., `up web` de docker-compose) no aplica `-u`.

**Fix:** desde shell:
```bash
docker compose run --rm web \
  odoo -d <bd> -u <modulo> --stop-after-init --no-http
```

Para módulo **nuevo** (no instalado aún), usar `-i` en su lugar.

---

## G10 · `l10n_es_aeat_mod303` falla con "dates overlap with existing record"

**Síntoma:** durante `-u all`, módulos AEAT (mod303, mod347, etc.) fallan al cargar fixtures CSV con `Error! The dates of the record overlap with an existing record.`

**Causa:** los maps de impuestos AEAT (`l10n.es.aeat.map.tax`) cargan CSVs anuales con `date_from / date_to`. La BD post-upgrade tiene registros viejos con fechas solapadas con los nuevos. La constraint `unique daterange` peta.

**Fix obligatorio antes del `-u all`** (clientes españoles): ejecutar el helper `AeatMapTaxFix`:

```yaml
fixes:
  - id: aeat_map_tax
    params:
      close_at: '2022-12-31'   # ajustar al último día del año fiscal previo al upgrade
      models:
        - l10n.es.aeat.map.tax
        - l10n.es.aeat.mod303.map.tax.line
        - l10n.es.aeat.mod347.map.tax.line
        - l10n.es.aeat.mod349.map.tax.line
        - l10n.es.aeat.mod111.map.tax.line
        - l10n.es.aeat.mod115.map.tax.line
```

El helper cierra (`date_to = close_at`) los maps que tengan `date_to` NULL o posterior al close_at, dejando espacio para los nuevos.

**Cuándo aplica:** clientes con `countries: ['ES']` migrando de cualquier v_X a v18 (visto en Tecniman, Pharmactive, ARBE).

**Prevención automática:** el compiler debería auto-inyectar este fix si `context.countries` contiene `ES`. **Pendiente impl** (M15).

---

## G9 · Versiones core/enterprise/OCA desincronizadas

**Síntoma:** módulos nuevos rompen al cargar (ej: `spreadsheet_edition` falla).

**Causa:** se actualizó enterprise/OCA pero no el core. Los repos van encadenados.

**Fix:** revertir a versiones congeladas del set conocido. Para el set Tecniman:
- Core Odoo: `18.0-20251001`
- Enterprise: commit `70ba62406b1` (2025-10-01)
- OCA repos: commits previos a `2025-10-02`

**Prevención:** documentar en `migration.yaml.context.source_repos` el commit de cada repo y validar antes de cada iteración.
