# Mantener `migration_kit` y la skill cuando descubres casos no cubiertos

Cada migración real descubre patrones nuevos. La diferencia entre "fix
puntual" y "mejora del ecosistema" la marca esta pregunta:

> **¿Este caso pasará en al menos 1 cliente más?**

Si **sí** → la lógica del fix sube al kit. Si **no** → queda como
`sql/NN_*.sql` o `python/*.py` en el workdir del cliente.

## Decisión: ¿genérico o cliente-específico?

| Indicador | Genérico (sube al kit) | Cliente-específico (queda en workdir) |
|---|---|---|
| Patrón Odoo / bug Odoo | ✓ | |
| Patrón OCA (afecta a varios módulos OCA) | ✓ | |
| Patrón Indaws Core (afecta a `indaws_*`) | ✓ | |
| Convención v10 común (Tecniman, otros clientes v10) | ✓ | |
| Bug específico del custom del cliente | | ✓ |
| Datos del cliente (mapeo de IDs, valores, etc.) | | ✓ |
| Decisión funcional del consultor | | ✓ |
| Configuración multi-company particular | | ✓ |

> **Contrato de autoría de un fix SQL** (clasificación obligatoria GENERIC / OCA-CONDITIONAL
> / CUSTOM-CONDITIONAL, cabecera estándar `-- FIX-NNN`, idempotencia, y la regla "solo
> GENERIC y OCA en `references/fixes/`"): ver **`../fix-guidelines.md`**. Síguelo siempre
> que crees un fix nuevo.

## Flujo cuando subes mejora al kit

1. **Apunta el patrón en memoria del proyecto** antes de tocar código:
   `memory/skill-odoo-database-upgrader-improvements.md`
   Una fila por hallazgo: **causa raíz** + **caso real** + **propuesta de fix**.

2. **Elige el helper objetivo** en `indaws_migration_kit`:
   - Patrón de saneamiento BD post-restore → nueva sub-fase en
     `helpers/odoo/post_restore_validator.py` (P9, P10, ...)
   - Patrón de manejo de módulos → ampliar `helpers/odoo/*` existente
     o crear `helpers/odoo/<patron>.py` nuevo
   - Patrón Studio → `helpers/studio/`
   - Patrón validación → `helpers/validation/`
   - Patrón SQL pre/post → `helpers/sql/`

3. **Implementa con opt-in por defecto**. Detecta SIEMPRE (modo report),
   aplica SOLO con flag explícito (`apply_fix=True` + `param_específico=True`).
   Caso real: P5 detecta pero `delete_unowned_views: True` opt-in
   porque Studio puede crear vistas legítimas sin xmlid.

4. **Tests en `indaws_migration_kit/tests/helpers/`**. Mínimo:
   - 1 test detección (caso positivo)
   - 1 test no-detección (caso negativo / falso positivo conocido)
   - 1 test idempotencia (re-ejecución no rompe)
   - 1 test apply=False (dry-run no muta)

5. **Compiler**: si el helper se beneficia de auto-emisión por contexto,
   añadir lógica en `migration/compiler.py`. Ej:
   - `manifest_preflight` auto-inject si `context.source_repos` no vacío
   - `smoke_rpc` auto-inject si `plan.odoo_url` definido
   - `data_migration_<name>` auto-emit si `modules.<name>.data_sql`

6. **Documenta en `../helpers-catalog.md`** con caso real
   (cliente + fecha + cifras). Sin caso real → no documentar
   (evita ruido).

7. **Actualiza el `migration.yaml` del cliente actual** para usar el
   helper nuevo en lugar del workaround manual. Re-test que produce el
   mismo resultado.

8. **Versión del kit**: bump menor si añade helper sin breaking,
   bump major si rompe interfaz de algún helper. Tag git + changelog.

## Flujo cuando ampliar la skill (no el motor)

Ampliar la skill `odoo-database-upgrader` cuando:

- El conocimiento es **declarativo** (no código ejecutable): gotcha
  nuevo, patrón a evitar, decisión arquitectural, recipe SQL
- Va a `../gotchas/index.md` (lista de gotchas) o
  `../gotchas/v10_v18_migration.md` (específicos v10→v18)
- O nueva fase del workflow → `../phases/fase-*.md`
- Si es un playbook (paso a paso): `../playbooks/<grupo>/<nombre>.md`

Cambios en la skill:
1. Editar/añadir el `.md` en `references/`
2. Si añade fase: actualizar SKILL.md (tabla "FASES")
3. Si añade playbook: añadirlo a "Tabla situación → playbook"
4. Verificar SKILL.md < 150 líneas (mover detalle a references si excede)
5. PR a `ai-claude-developer` repo

## Flujo cuando la mejora cruza ambos

Caso típico: nuevo patrón requiere helper en kit **+** documentación en
skill **+** entrada en `migration.yaml` ejemplo:

1. Implementa helper en kit (+ tests)
2. Documenta en `../helpers-catalog.md` (skill)
3. Si afecta a un workflow concreto, actualiza `../phases/` o
   `../playbooks/`
4. Migración del cliente actual: aplica el helper desde el plan
5. 2 PRs paralelos (kit + skill)

## Antipatrones

- ❌ Hardcodear valores cliente en helper genérico (módulos, IDs, etc.)
- ❌ Helper que rompe si `apply=False` (debe ser SIEMPRE dry-run safe)
- ❌ Helper sin tests
- ❌ Patrón documentado sin caso real verificado
- ❌ Cambios al kit sin bump de versión + changelog
- ❌ Dejar `migrate_orphan_fields_to`/`drop_orphan_fks`/etc. activos por
     defecto (siempre opt-in para que el dev decida)

## Plantilla de patrón al apuntar en memoria

```markdown
| ID | Patrón | Estado |
|---|---|---|
| P<n> | [Nombre corto del patrón] | ⚠ Pendiente impl / ✅ Aplicado <fecha>. <Caso real>: <cliente> <cifras>. Implementación: <helper o ruta>. Opt-in con `<param>: True`. <n> tests. |
```

Ej real:
```
| P8 | sanitizar whitespace en <attribute name="groups"> | ✅ Aplicado
2026-05-18. Bug Odoo v18: el evaluador NO strip los \n al evaluar
groups → filtra elementos cuyo user SÍ tiene el grupo. Caso real
Tecniman: vista 3872 tecniman_security → pestaña Accounting invisible.
helpers/odoo/post_restore_validator.py sub-fase. Opt-in
`sanitize_attribute_whitespace: True`. 3 tests.
```

## Casos del port de código que merecen helper en el kit

Cuando portas código v_X → v18 (Fase 2.A), suelen aparecer patrones cuya
**transformación equivalente en BD** debería estar en el kit, no en el
port manual del módulo. Distinción:

- **`odoo-module-upgrader`** transforma **archivos** del módulo (Python/
  XML/SCSS en disco) según patrones de port v_X→v18
- **`indaws_migration_kit`** transforma **datos en BD** del cliente
  migrado (vistas almacenadas, dominios, registros, formatos, etc.)

**Regla de selección**: si la transformación requiere recorrer registros
de BD masivamente y es declarativa (registry de mappings o regex),
sube al kit. Si es ad-hoc para 1 fichero o caso único, queda en el port
manual.

Casos típicos del port y su contraparte en BD:

| Caso del port | Helper kit (estado) |
|---|---|
| Field renames v_X→v18 (`customer`→`customer_rank`, etc.) en BD: vistas custom guardadas, dominios de `ir.act.window`, server actions con código, `ir.cron`, `ir.filters` | ✅ `FieldRenameStep` con registry |
| Cambios en `arch_db` de vistas custom guardadas (attrs→inline, tree→list, etc.) | ✅ `ViewArchValidator` + `ViewArchFixer` (delega a `odoo-module-upgrader/transform_arch.py`) |
| Columnas con cambio de tipo entre versiones (boolean→jsonb company_dependent) | ✅ P3 `jsonb_company_dependent` |
| `arch_db` text→jsonb en v17 | ✅ Manejado en `post_restore_validator` |
| Modelos eliminados/renombrados (`procurement.order` v11+, `stock.pack.operation`) | ✅ `OBSOLETE_MODELS_REGISTRY` |
| Renames OCA entre versiones (`sale_order_type` → equivalente Indaws/OCA) | ⚠ Parcial vía CORE_REPLACE; falta registry OCA-renames |
| `account.move.state` v13 (`draft/open/paid`) → v17+ (`draft/posted`) | ❌ NO cubierto. Útil para v13/v14/v15→v17+ |
| `ir.translation` eliminado en v17 → columnas `*_jsonb` por idioma | ❌ NO cubierto (lo hace Upgrade Service pero deja restos) |
| `tax_totals_json` (str) → `tax_totals` (dict) en v18 | ⚠ Parcial — afecta templates QWeb más que datos |
| Studio dumps con modelos cambiados entre versiones | ❌ NO cubierto |
| Server actions con `safe_eval` de código incompatible | ❌ NO cubierto (linter pre-deploy sería útil) |
| `ir.cron` apuntando a métodos renombrados | ❌ Parcial — P1 detecta FK rota a `ir.act.server` pero no auditoría métodos |
| `ir.filters.sort`/`domain` con campos renombrados o `action_id` huérfano | ❌ NO cubierto (caso real Tecniman: 9 filtros `Productos` apuntaban a action 131 cuando el menú v18 abre 824) |
| Reports QWeb con sintaxis vieja (rutas `t-call`, t-options) | ⚠ Cubre parcial `view_arch_fixer` |
| `ir.default` con valores legacy (campos eliminados) | ❌ NO cubierto |
| Workflows v10 (`workflow.workflow` eliminado v11+) — `wkf_id` references rotos | ❌ NO cubierto (relevante solo en migraciones desde v10) |
| `mail.message.subtype` huérfanos tras eliminación de modelos | ❌ NO cubierto |
| `ir.attachment` con `res_model` apuntando a modelos eliminados | ❌ Parcial |

## Cómo identificar candidatos a helper durante el port

Durante el port de un módulo, cuando edites un archivo, pregúntate:

> Este cambio que estoy haciendo en el archivo, ¿también tiene que
> aplicarse a registros equivalentes en la BD del cliente?

Ejemplos:

1. Renombras `customer` → `customer_rank` en una vista del módulo →
   ¿hay vistas custom (en BD del cliente, no en disco) con `customer`?
   → SÍ → `FieldRenameStep` ya lo cubre, asegurar registry tiene la entry
2. Cambias `<tree>` → `<list>` en `views/*.xml` → ¿hay `arch_db` en BD
   del cliente con `<tree>`? → SÍ → `ViewArchFixer` lo cubre
3. Cambias `_inherit = "foo.bar"` porque foo.bar se renombró → ¿hay
   datos en `ir.model_data` apuntando a `foo.bar`? → registro nuevo
   `OBSOLETE_MODELS_REGISTRY`
4. Eliminas un `@api.depends('campo_x')` porque `campo_x` ya no existe
   → ¿hay computed fields en BD que referencian `campo_x`? → posible
   patrón nuevo en `post_restore_validator`

## Antes de implementar un helper nuevo del port

Verifica que:
- El patrón es **declarativo** (registry de mappings o regex aplicable a N registros)
- Aparece en al menos **2 clientes** o es un cambio API estándar de Odoo entre versiones documentado
- La detección puede hacerse **sin lanzar el módulo** (queries sobre tablas técnicas: `ir_ui_view`, `ir_model_fields`, `ir_act_window`, `ir_cron`, `ir_filters`, etc.)
- El fix es **idempotente** y **dry-run safe**

Si los 4 puntos no aplican, probablemente sea fix de cliente (`sql/NN_*.sql`), no helper.
