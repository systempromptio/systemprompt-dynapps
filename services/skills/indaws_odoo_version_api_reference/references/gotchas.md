# Gotchas — Errores frecuentes de Claude con versiones de Odoo

Errores reales observados al generar codigo. Actualizar este fichero cada vez que se detecte uno nuevo.

---

## Python

### 1. Usar `name_get()` en v18+
**Error**: Claude genera `name_get()` por defecto porque aparece en mucho training data.
**Fix**: En v18+ SIEMPRE usar `_compute_display_name()`. En v16-17, preferir `_compute_display_name()` tambien.

### 2. Firma antigua de `_read_group()` en v16.3+
**Error**: Claude usa `fields=['field'], groupby=['field']` que es la firma antigua.
**Fix**: Desde v16.3 usar `_read_group(domain, ['field'], ['__count'])`.

### 3. Olvidar `aggregator` y usar `group_operator` en v17.2+
**Error**: Claude pone `group_operator='sum'` porque es mas comun en el training data.
**Fix**: Desde v17.2 usar `aggregator='sum'`.

### 4. `type='product'` en v18+ para productos almacenables
**Error**: Claude escribe `'type': 'product'` para storable products en v18+.
**Fix**: En v18+ usar `'is_storable': True`.

### 5. Tuplas en lugar de Command API en v16+
**Error**: Claude genera `[(0, 0, {...})]` en vez de `[Command.create({...})]`.
**Fix**: Desde v16 usar `from odoo import Command` y `Command.create()`.

### 6. Import de `slug` directo en v18+
**Error**: `from odoo.addons.http_routing.models.ir_http import slug` falla en v18+.
**Fix**: Crear wrapper: `slug = lambda v: request.env['ir.http']._slug(v)`.

---

## XML / Views

### 7. `<tree>` en v18 y v19 (no solo v19)
**Error**: Claude usa `<tree>` por defecto.
**Fix**: **Desde v18** OBLIGATORIO `<list>`; `<tree>` solo vale en v17 y anteriores. `ir.ui.view.type`
pasa de `('tree','Tree')` (`odoo17/odoo/addons/base/models/ir_ui_view.py:163`, `View.type`) a `('list','List')` (`odoo18/…:153`), sin
shim, y el propio core de v18 no tiene ni un `<tree ` en `addons/*/views/*.xml` (v17: 322 ficheros).
Verificado 2026-08-07.

### 8. `<group>` en search views de v19
**Error**: Claude envuelve filtros de agrupacion en `<group expand="0" string="Group By">`.
**Fix**: En v19 los filtros van directamente bajo `<search>`, sin `<group>`.

### 9. `active_id` en contextos de form en v19
**Error**: Claude usa `active_id` en expresiones de contexto dentro de vistas form.
**Fix**: En v19 usar `id` en su lugar.

### 10. `numbercall` en cron jobs de v18+ (NO es de v19)
**Error**: Claude incluye `<field name="numbercall">-1</field>` en definiciones de cron.
**Fix**: Eliminar la linea. El campo **desaparece en v18**, no en v19 (corregido 2026-08-17): 0
ocurrencias de `numbercall` y `doall` en CE18 **y** CE19, presentes en v17 (`odoo/addons/base/models/ir_cron.py:67`, `ir_cron`, v17). Un
`<field name="numbercall">` en un modulo v18 da `Invalid field 'numbercall'` — o sea que datarlo en
v19 deja sin aviso a quien migra v17->v18, que es donde rompe.
**Sustituto de `numbercall=1`** (una pasada y desactivar): `_notify_progress(..., deactivate=True)`
(`odoo/addons/base/models/ir_cron.py:734-749`, `ir_cron._notify_progress`, v18, aplicado en `odoo/addons/base/models/ir_cron.py:444-445`, `ir_cron._run_job`). Y v18 anade **auto-desactivacion por fallos**:
5 fallos + 7 dias (`:24-25`).

### 11. `kanban-box` en v19
**Error**: Claude genera `t-name="kanban-box"` para templates kanban.
**Fix**: En v19 usar `t-name="card"`.

---

## JavaScript

### 12. `useService("rpc")` en frontend de v19
**Error**: Claude usa el servicio RPC en componentes publicos de v19.
**Fix**: El RPC service NO esta disponible en frontend de v19. Usar `fetch()` con wrapper JSON-RPC. Ver `references/v18-base/javascript-services.md` (y `references/diffs/v19-diff.md`) para el patron completo.

### 13. `odoo.define()` en v17+
**Error**: Claude genera el patron legacy AMD para modulos JS.
**Fix**: Desde v17 usar `/** @odoo-module **/` con ES imports y `registry.category().add()` para tours.

### 14. QUnit en v18+
**Error**: Claude genera tests con `QUnit.test()` y `assert.ok()` para v18+.
**Fix**: En v18+ usar Hoot: `test()` con `expect().toBe()`, ficheros `*.test.js`, `mountWithCleanup()`.

---

## Tests

### 15. Import de Form class en v17+
**Error**: `from odoo.tests.common import Form` falla en v17+ (deprecated import).
**Fix**: Desde v17 usar `from odoo.tests import Form`.

### 16. `SavepointCase` en v17+
**Error**: Claude hereda de `SavepointCase`.
**Fix**: Deprecated en v17, eliminado en v18. Usar `TransactionCase`.

### 17. `freezegun` externo en v18+
**Error**: Claude importa `from freezegun import freeze_time` en v18+.
**Fix**: En v18+ usar `from odoo.tests import freeze_time` (built-in).

---

## SCSS / Themes

### 18. Sin `menu`/`footer`/`copyright` en paletas v19
**Error**: Claude define paletas de colores sin asignar que color va donde.
**Fix**: En v19 OBLIGATORIO incluir `'menu': N, 'footer': N, 'copyright': N` en la paleta.

### 19. Sin `properties` en font configs de v19
**Error**: Claude define fuentes sin la seccion `properties.base`.
**Fix**: En v19 cada fuente necesita `'properties': ('base': ('font-size-base': 1rem, ...))`.

### 20. `px` en lugar de `rem` para botones en v19
**Error**: Claude usa `'btn-padding-y': 16px`.
**Fix**: En v19 usar `rem`: `'btn-padding-y': 1rem`.

---

## ORM / Engine (version-independent)

### 21. M2O sin check_company — y `check_company` **solo** tampoco vale
**Error**: Claude crea `Many2one('account.journal')` sin `check_company=True`.
**Fix**: Siempre añadir `check_company=True` en M2O que referencian datos que pueden ser de otra compañía (journal, warehouse, partner, account). Excepciones: `company_id`, `currency_id`, datos globales (country, UoM).
**Y el modelo necesita `_check_company_auto = True`** (vale `False` por defecto: v18
`odoo/models.py:630`, `BaseModel._check_company_auto`, v18; v19 `odoo/orm/models.py:451`, `BaseModel._check_company_auto`) — es la **única** condición de las llamadas a
`_check_company()` en `create`/`write`. **Medido en v18 y v19**: dos modelos idénticos salvo esa
línea; el que no la tiene **acepta en silencio** un registro de la empresa A apuntando a un recurso
de la B. Poner solo una de las dos deja el módulo igual de desprotegido, pero con aspecto de
protegido. Sonda: `ai-claude-developer/tools/bench-orm` → `tests/test_company.py`.

### 22. Computed store=True con depends incompleto
**Error**: Claude declara `@api.depends('price_unit')` pero el compute también lee `discount` — el campo no se recalcula al cambiar descuento.
**Fix**: Listar TODOS los campos que afectan al cómputo, incluyendo relaciones. Si no estás seguro, usa `store=False`.

### 23. Deep depends path
**Error**: `@api.depends('partner_id.child_ids.invoice_ids.amount_total')` — triggers recomputación masiva en cascada.
**Fix**: Mantener profundidad de depends <= 2. Si necesitas más, usa campo intermedio store=True o store=False.

### 24. Monetary: el orden de declaración NO importa — MEDIDO, esta gotcha era falsa
**Lo que decía**: que declarar `amount = fields.Monetary(...)` antes de `currency_id` rompe el
`write`. **Medido en v18 y v19** con tres modelos (divisa después, divisa antes, `currency_field`
con otro nombre declarado después): los tres crean y **redondean igual** a los decimales de la
divisa. El `write_sequence` ordena la escritura en el ORM, no obliga a nada en el fuente.
**Lo que sí pasa, y es lo que hay que vigilar**: **sin divisa el `write` tampoco falla** — el valor
entra tal cual, sin redondear (medido: `12.3456789` se guarda entero). Es decir, el fallo real es
silencioso y de datos, no un error al escribir.
**Fix**: asegurar que el `currency_field` **existe y tiene valor**; el orden en la clase da igual.
Sonda: `ai-claude-developer/tools/bench-orm` → `tests/test_money.py::TestMonetaryOrder`.

### 25. N+1 en compute loop
**Error**: `for rec in self: rec.x = rec.partner_id.name` — un query por registro.
**Fix**: Pre-cargar la relación antes del loop: `self.partner_id` (fuerza prefetch). Luego iterar.

### 26. sudo() en compute vs compute_sudo
**Error**: Claude usa `self.sudo().field` dentro de un `_compute_*` para acceder a datos restringidos.
**Fix**: Usar atributo `compute_sudo=True` en la declaración del campo. `sudo()` dentro del compute es un antipatrón que bypassea TODAS las reglas.

### 27. Record rules AND vs OR
**Error**: Claude crea una record rule con `groups` cuando debería ser global (sin groups).
**Fix**: Reglas sin `groups` = globales (AND). Reglas con `groups` = grupo (OR entre sí). Error común: crear rule de compañía con grupo cuando debería ser global.

### 28. store=False sin search method
**Error**: Campo computed `store=False` sin método `search` — invisible en filtros de búsqueda.
**Fix**: Añadir `search='_search_field'` si los usuarios necesitan filtrar por ese campo.

### 29. No precompute en campos elegibles
**Error**: Campo computed `store=True` cuyo `depends` solo tiene campos del mismo modelo, pero sin `precompute=True`.
**Fix**: Si el compute depende SOLO de campos presentes en `create vals` → añadir `precompute=True` para evitar un UPDATE extra tras el INSERT.

### 30. Olvidar flush antes de SQL directo
**Error**: Claude hace `self.write({'field': val})` seguido de `self.env.cr.execute("SELECT field FROM ...")` — lee valor antiguo.
**Fix**: Llamar `self.flush_recordset(['field'])` entre write ORM y SQL directo. El write ORM actualiza cache pero NO la BD inmediatamente.

### 31. with_context() NO modifica el recordset actual
**Error**: Claude escribe `self.with_context(key=val)` esperando que `self` cambie.
**Fix**: `with_context()` retorna un NUEVO recordset. Siempre: `new_self = self.with_context(key=val)` y usar `new_self` despues.

### 32. invalidate_recordset() necesario tras UPDATE SQL
**Error**: Claude ejecuta `UPDATE table SET field = X` via SQL y luego lee `record.field` — cache devuelve valor antiguo.
**Fix**: Llamar `self.invalidate_recordset(['field'])` despues de SQL UPDATE para limpiar la cache ORM.

### 33. sudo() NO bypasea `_check_company()`
**Error**: Claude usa `record.sudo().write(...)` para escribir datos cross-company, pero falla.
**Fix**: `sudo()` bypasea ACL y record rules, pero NO `_check_company()` — **medido en v18 y v19**,
con `sudo()` y con `with_company()`: los dos siguen lanzando `UserError`. No es una constraint SQL
sino una comprobación Python en `create`/`write`, y compara `record.company_id` (no `env.company`),
por eso cambiar la empresa activa no la esquiva. Para cross-company de verdad hay que cambiar la
empresa **del registro**, no el contexto.

### 34. _order con campos no-stored = sort en Python
**Error**: Claude define `_order = 'computed_field desc'` donde `computed_field` es `store=False`.
**Fix**: Campos no-stored no tienen columna SQL → `search()` con order hace sorting en Python tras cargar TODOS los registros. Extremadamente lento con muchos registros. Usar `store=True` o repensar el ordering.

### 35. company_dependent: el coste depende de la version, y en v18+ NO es ir.property
**Error**: Claude razona sobre `company_dependent=True` con el modelo de almacenamiento equivocado para la version objetivo.
**Fix**: cambia el mecanismo en v18.
- **v17 y anteriores**: cada valor unico crea un registro `ir.property` (`addons/base/models/ir_property.py`, existe en v17). Ahi si: 10000 productos × 5 companies = 50000 filas, y con alta cardinalidad conviene plantear un modelo relacional.
- **v18/v19**: `ir.property` **fue eliminado** (el fichero no existe en v18 ni v19). El valor vive en una columna **`jsonb`** de la propia tabla del modelo, con `company_id` como clave (`odoo/fields.py:178-180`, `Field` — la docstring de `company_dependent`; el tipo de columna, `odoo/fields.py:278-280`, `Field._column_type`). No hay filas extra, asi que el argumento de cardinalidad NO aplica; lo que si aplica es que buscar/indexar sobre ese campo es distinto a una columna normal.

Verificado 2026-08-07.

### 36. recursive=True en computed fields dispara cascada
**Error**: Claude anade `recursive=True` sin entender el impacto. Campo computed con `recursive=True` re-evalua TODA la jerarquia cuando cualquier nodo cambia.
**Fix**: Usar solo cuando realmente necesitas propagacion jerarquica (ej: `parent_path` dependence). Cada write en un nodo triggera recompute de todos los descendientes/ancestros.

### 37. Olvidar @api.depends en campo related sobreescrito
**Error**: Claude sobreescribe un campo `related` con compute custom pero olvida poner `@api.depends`.
**Fix**: Campos `related` tienen depends automatico. Al cambiar a `compute`, los depends desaparecen y el campo nunca se recalcula. Siempre declarar `@api.depends` explicito.

### 38. Properties field (JSON) sin indice = full scan
**Error**: Claude usa `lot_properties` o `task_properties` en filtros de busqueda sin considerar rendimiento.
**Fix**: Los campos Properties son JSONB. Sin indice GIN, cada busqueda hace full table scan. Para filtros frecuentes sobre properties, considerar campos reales o crear indice GIN manual.

### 39. v19: expression.AND/OR reemplazado por Domain()
**Error**: Claude usa `from odoo.osv import expression; expression.AND([dom1, dom2])` en v19.
**Fix**: En v19 usar `from odoo.fields import Domain; domain = Domain(dom1) & Domain(dom2)`. `expression` module reestructurado. `NEGATIVE_TERM_OPERATORS` → `Domain.NEGATIVE_OPERATORS`.

### 40. v19: models.Constraint/Index reemplaza _sql_constraints
**Error**: Claude genera `_sql_constraints = [('name', 'CHECK(...)', 'message')]` en v19.
**Fix**: En v19 declarar `models.Constraint` como atributo de clase con prefijo `_` (ej: `_name_uniq = models.Constraint('unique(name)', 'msg')`). ⚠ La lista legacy `_sql_constraints` en v19 **ya NO se aplica**: solo emite un warning ("no longer supported") y la constraint NO se crea (verificado `odoo/orm/model_classes.py:162-164`, `add_to_registry`, v19). No es "deprecada pero funciona" — deja de tener efecto.

### 41. Inherited view sin groups= dispara AccessError
**Error**: Claude hereda `res.partner` form para anadir stat button o page con One2many a un modelo restringido (ej: `rappel.partner`) sin anadir `groups=` al elemento XML. Usuarios sin el grupo ven el elemento → Odoo lee el modelo → AccessError.
**Fix**: Anadir `groups="my_module.group_xxx"` en buttons, pages y fields que referencian modelos con ACL restringido. El `groups=` debe coincidir con el grupo que tiene `perm_read` en `ir.model.access.csv`.

### 42. Group rule restrictiva sin complementaria para base.group_user
**Error**: Claude crea una `ir.rule` de grupo restrictiva (ej: `[('carrier_id.user_id', '=', user.id)]` para grupo portal) en `stock.move` sin anadir regla complementaria `[(1,'=',1)]` para `base.group_user`. Resultado: usuarios internos que tambien estan en el grupo portal pierden acceso a los demas moves.
**Fix**: Siempre que crees una regla de grupo restrictiva en un modelo amplio (stock.move, stock.picking, account.move), anadir regla complementaria `[(1,'=',1)]` para `base.group_user` en el mismo modelo.

### 43. `precompute=True` obligatorio para required + computed + store — **no es de v19: ya en v18**
**Error**: campo `required=True, compute='_compute_x', store=True` sin `precompute=True`. El INSERT
sale sin esa columna y Postgres lo rechaza con `NotNullViolation`.
**Corregido por medición (v18 y v19, mismo modelo)**: esta gotcha decía «en v19», y se leía como un
riesgo que aparece al migrar. **Falla igual en 18.0**: un compute almacenado normal corre DESPUÉS
del INSERT en las dos. Si un módulo v18 no lo sufre no es por la versión, es porque nadie creó un
registro sin pasar el campo.
**Fix**: `precompute=True`. Pasar el valor a mano en el `create` también evita el reventón (el
INSERT ya lleva columna), pero **el compute lo pisa justo después** — no sirve como valor.
Sonda: `ai-claude-developer/tools/bench-orm` → `tests/test_money.py::TestRequiredComputed`.

### 44. `@api.constrains` con dotted path ignorado silenciosamente (v19)
**Error**: Claude usa `@api.constrains('child_ids', 'child_ids.template_id')` — el path dotted `child_ids.template_id` se ignora silenciosamente en v19. La constraint solo se triggerea cuando cambia `child_ids`, no cuando cambia `template_id` de un child.
**Fix**: Mover la validacion al modelo hijo con `@api.constrains('field_id')` que referencie campos directos.

### 45. `pool.cursor()` crea transaccion separada — invisible en tests y mismo request (v19)
**Error**: Claude usa `with self.pool.cursor() as cr: cr.execute(...)` para logging o actualizacion de estado. Esto abre un cursor separado con su propia transaccion. Los datos escritos son invisibles para el cursor principal (y viceversa). Ademas, auto-commiteea al salir del context manager, lo cual esta prohibido en tests.
**Fix**: Usar `self.env.cr` para SQL dentro del mismo request. Solo usar `pool.cursor()` si necesitas persistencia independiente (ej: logging que sobreviva a rollback), nunca para lectura/escritura de datos de negocio.

### 46. `_inherit` con `_name` diferente = tabla separada, FK incompatible
**Error**: Claude asume que `class MyModel(models.Model): _name='my.model'; _inherit='parent.model'` comparte tabla con `parent.model`. En realidad crea tabla separada `my_model`. Un FK a `parent_model` no acepta IDs de `my_model`.
**Fix**: No referenciar IDs de modelos hijos en FK que apuntan al padre. Si necesitas relacion, usar campo `Reference` o buscar el registro por nombre.

### 47. `ir.cron` create/write necesita sudo() desde usuarios sin privilegios admin
**Error**: Claude crea crons dentro de `create()` o `_update_cron()` sin `sudo()`. Usuarios con grupo admin del modulo pero sin acceso directo a `ir.cron` reciben `AccessError`.
**Fix**: Siempre usar `self.env['ir.cron'].sudo().create(...)` y `.sudo().write(...)` en metodos que gestionan crons automaticamente.

### 48. `fields.Json` compute devuelve `False` para valores vacios (v19)
**Error**: Claude asume que un campo `fields.Json(compute=...)` devuelve `{}` cuando el valor es vacio. En v19, devuelve `False` (falsy). Test con `assertEqual(record.json_field, {})` falla.
**Fix**: Usar `assertFalse(record.json_field)` o comparar con `not record.json_field`.

### 49. `missing-return` en compute/onchange/hook overrides — falso positivo de pylint_odoo
**Error**: pylint_odoo señala como warning cualquier metodo `_compute_*`, `_onchange_*` o hook override (`copy`, `unlink`, `create`, `write`) que llame a `super()` sin retornar. La intuicion Python sugiere "siempre retornar lo que devuelve super()".
**Fix**: En Odoo, los metodos `_compute_*` y `_onchange_*` por convencion NO retornan valor — mutan `self` directamente y la engine ignora cualquier return. Hook overrides como `create()` o `copy()` SI deben retornar lo que devuelve `super()`. La regla del linter no distingue contexto. Auditoria cross-repo en Indaws Core (2026-04): 103 violaciones, todas falsos positivos por convencion. La regla queda OFF en `.pylintrc-mandatory` con rationale documentado. Si tu compute realmente "deberia" retornar algo, repensa el diseño porque esta roto.

### 50. `F823` — tupla `_, _ = ...` shadowa `_` i18n como local
**Error**: Patron Python `_, _ = some_function()` para descartar valores reasigna `_` como variable local en el scope. Si en ese mismo metodo se usa `_("text")` para traduccion, `_` ya no es la funcion translation sino el ultimo valor descartado. `UnboundLocalError` o traduccion silenciosamente rota. Bug latente real detectado en `indaws_report_docx/ir_actions_report.py` durante el workstream Core 2026-04.
**Fix**: Usar `__` (doble underscore) para descartar:
```python
# WRONG — rompe _() i18n en el mismo metodo
_, _ = generate_pair()
raise UserError(_("Failed"))  # _ es el valor descartado, no la funcion

# CORRECT — usar __
__, __ = generate_pair()
raise UserError(_("Failed"))  # _ sigue siendo translation
```
Detectable con ruff `F823` activado.

### 51. v19: `_(...)` legacy en lugar de `self.env._(...)`
**Error**: Claude usa `from odoo import _` + `_("texto")` por inercia del training data en codigo v19.
**Fix**: En v19 el patron canonico es `self.env._("texto")` (lazy + lang-aware respecto al contexto del request). En controllers usar `request.env._("texto")`. El `_()` legacy sigue funcionando pero pylint_odoo lo flaggea via la regla `prefer-env-translation`. Migrar progresivamente en codigo existente; usar `self.env._()` en codigo nuevo.

**Disponibilidad por version**: `env._` existe desde **v18** (no en v16/v17 — ahi solo el `_()` de modulo). **Caveat de CI** que muerde en superficies expuestas: en contextos `auth="none"` / sin frame de request (el Environment se construye a mano y Odoo no inyecta `lang`), el `_()` de modulo emite el warning *"no translation language detected"* que tumba el gate `checklog-odoo`. En v18+ resuelvelo construyendo un env que lleve lang (`env(context={**env.context, "lang": <lang>})`) y traduciendo con `env._(...)`; en v16/v17 (sin `env._`) usa `_()` pero asegurate de que el env tenga lang. Patron tipico en MCP/JSON-RPC: resolver el idioma del usuario (`user.lang or "en_US"`) e inyectarlo antes de cualquier `_()`.

---

## Regla general

Cuando dudes de que patron usar, consulta la tabla resumen en `SKILL.md` con la version exacta. Si el usuario no especifica version, **preguntar antes de generar codigo**.

Para internals del motor (cache, prefetch, flush, security layers), consulta `references/engine-internals.md`.
Para patrones de codigo engine-aware, consulta `references/engine-patterns.md`.
