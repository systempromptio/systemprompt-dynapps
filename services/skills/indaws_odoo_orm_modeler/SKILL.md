---
name: indaws-odoo-orm-modeler
version: 0.2.12
effort: high
model: opus
description: >-
  Diseña la capa modelo/ORM de un módulo Odoo (v17-v19): campos, computes,
  constraints, herencia (_inherit/_inherits/delegation), relaciones x2many,
  Command, company_dependent, Properties, precompute, depends_context, diffs v19.
  NOT vistas, controllers ni JS.
  Triggers: "diseña el modelo", "campo computed / related / inverse", "constraint
  SQL vs Python", "_inherit vs _inherits", "Command create/link/set",
  "que cambio del ORM en v19", "computed field", "api.depends", "api.constrains",
  "model inheritance", "properties field".
allowed-tools: [Read, Grep, Glob, Edit, Write]
---
<!-- verified: v18.0 @ 2026-06 -->

# Odoo ORM Modeler

Reference practico para disenar correctamente la capa de modelo/ORM de un modulo Odoo:
tipos de campo y atributos, computed fields, constraints, onchange, herencia y relaciones.
**Base v18**; los diffs v17 y v19 se documentan relativos a v18.

Estandares compartidos: `references/shared/coding-standards.md` (codigo SIEMPRE en ingles;
strings de UI traducibles; ortografia es_ES en texto generado).

## Workflow

1. **Identifica la intencion**: nuevo modelo, extension de uno existente, o cambio de un campo.
   Si es cambio de un campo ya en produccion, [delega] en `odoo-impact-analyzer` ANTES de tocar.
   **Antes de crear nada nuevo**, aplica los ejes de `references/shared/organic-module-design.md`:
   reutiliza antes de crear (estandar/Core/Studio) y monta lo nuevo sobre motores estandar
   (`_inherits`/mixins/componer via registros estandar) en vez de reimplementar.
2. **Fija la version** (17/18/19). Determina el fichero fuente correcto (en v19 el ORM esta en
   `odoo/orm/`, ver `references/version-diffs.md`).
3. **Elige tipo de herencia** antes de escribir campos: lee la tabla de
   `references/inheritance-and-relations.md` (`_inherit` vs `_inherits` vs `_name` nuevo vs Abstract/Transient).
4. **Declara campos**: tipos y atributos en `references/fields-and-attributes.md`. Verifica el
   nombre exacto del atributo en la version objetivo (p.ej. `aggregator` v18+ vs `group_operator` v17).
   Si necesitas un **tipo de campo/columna PROPIO** (p.ej. geometria PostGIS), NO configures un
   campo estandar: ve a `references/custom-field-types.md` (v19: `_column_type`, indices GIST fuera
   de banda, override `_insert_cache`).
5. **Computes / constraints / onchange**: patrones en `references/compute-constraints-onchange.md`
   (stored vs no-stored, `inverse`/`search`, `precompute`, `@api.depends(_context)`,
   `@api.constrains` vs SQL constraint, cuando onchange legacy).
6. **Relaciones x2many**: usa `Command.*` (no tuplas magicas legacy). Ver
   `references/inheritance-and-relations.md`.
   Si necesitas un **operador de dominio NO-estandar** (p.ej. `geo_dwithin`), el mecanismo v19
   cambio: `references/custom-domain-operators.md` (`@operator_optimization` + `Domain.custom`).
7. Para firmas exhaustivas de cualquier API/decorador, [delega] en `odoo-version-api-reference`.
   Para scaffolding completo del modulo, [delega] en `odoo-module-creator`.

## GOTCHAS

1. **`group_operator` -> `aggregator`**: deprecado desde v18 (warning en runtime). En v17 el atributo
   es `group_operator`; en v18/v19 es `aggregator`. Fuente: `odoo/fields.py:482`, `Field._get_attrs` (warning), `odoo17/odoo/fields.py:323`, `Field.group_operator`.
2. **`_sql_constraints` (lista) desaparece en v19**: en v17/v18 es `_sql_constraints = [(name, sql, msg)]`.
   En v19 se ignora con warning y se usa `models.Constraint(definition, message)` a nivel de clase.
   Fuente: `odoo/orm/model_classes.py:162`, `add_to_registry` (warning), `odoo/orm/table_objects.py:79`, `Constraint`.
3. **`api` -> `orm/decorators` en v19**: el codigo de los decoradores vive en `odoo/orm/decorators.py`.
   `odoo/api.py` ya no es un **fichero**, pero `odoo/api/__init__.py` SI existe: es el paquete que
   re-exporta la superficie publica. `from odoo import api` sigue igual; lo que cambia es donde citas.
4. **`precompute` NO es v17+**: existe ya en v16 (atributo de clase `odoo16/odoo/fields.py:312`, `Field.precompute`; docstring del parámetro en `:232`). Presente en v17/18/19.
   Solo tiene sentido en campos `compute` + `store=True`; en no-computed emite warning.
5. **Computed sin `store=True`: filtrarlo NO falla en v18, MIENTE.** Medido: sin `search=`, el ORM
   loguea un ERROR y **descarta la condicion** (`odoo/osv/expression.py:1179-1186`, `expression.parse`, *«Ignore it:
   generate a dummy leaf»*) → el `search` devuelve **todo**. **v19 lanza `ValueError`** y ya no
   descarta nada: si el campo no tiene `search=`, la condicion atraviesa el optimizador
   (`odoo/orm/domains.py:956-957`, `DomainCondition._optimize_step`, que solo desvia cuando `field.search` existe) y revienta al
   construir el SQL en `Field.to_sql` (`odoo/orm/fields.py:1216`, `Field.to_sql`). Y `search=`
   habilita **buscar, no ordenar**: el `order` revienta igual (**v18** `odoo/models.py:2981`, `BaseModel._field_to_sql`, **v19**
   `odoo/orm/fields.py:1216`, `Field.to_sql`, mismo mensaje *"Cannot convert ... to SQL because it is not stored"*; ojo,
   en v19 **`odoo/models.py` no existe como fichero** — es el paquete `odoo/models/`). Escribir sigue
   necesitando `inverse=`. Detalle: `references/compute-constraints-onchange.md`.
6. **Tuplas magicas `(0,0,vals)` legacy** siguen funcionando pero usa `Command.*` (`fields.Command` en
   v17/18; `odoo/orm/commands.py` en v19). Mas legible y a prueba de cambios.
7. **`_inherits` (delegation) NO es `_inherit`**: `_inherits = {'model': 'fk_field'}` crea registros
   linkados (composicion). `_inherit = 'model'` extiende in-place. Confundirlos rompe el esquema.
8. **`company_dependent=True`: en v18+ SI hay columna, es `jsonb`** con `company_id` como clave
   (`odoo/fields.py:180`, `Field` docstring, `:280` "Company-dependent fields are stored as jsonb").
   `ir.property` fue **eliminado en v18** (existe en v17, no en v18/v19). Afecta a search e index. No
   te fies del comentario `# (property field)` de `odoo/fields.py:314`, `Field.company_dependent`: esta obsoleto en la propia fuente.
9. **Tipo de campo custom v19 — el read path NO llama `convert_to_cache`**: `_fetch_query` mete el
   valor CRUDO de BD en cache via `field._insert_cache(...)` (`models.py:~3931`). Si el crudo no es
   usable (EWKB hex de PostGIS, etc.) hay que override `_insert_cache`, no `convert_to_cache`. Y
   `index='gist'` NO existe: `check_indexes` ASEVERA `index in (btree, btree_not_null, trigram, ...)`
   (`odoo/modules/registry.py:833`, `Registry.cache_invalidated`); los indices especiales van fuera de banda en `update_db` con nombre propio.
   Detalle: `references/custom-field-types.md`.
10. **Operador de dominio custom v19 NO llega a `Field.condition_to_sql`**: `STANDARD_CONDITION_OPERATORS`
    (`odoo/orm/domains.py:81`, `STANDARD_CONDITION_OPERATORS`) es frozenset y el optimizador FULL asevera `operator in STANDARD...` (`:981`,
    `:1089`). Registra con `@operator_optimization([...])` (hace `CONDITION_OPERATORS.update`, `:1114`)
    y reescribe a `Domain.custom(to_sql=lambda model, alias, query: SQL(...))` (`:289`); la columna via
    `model._field_to_sql(alias, fname, query)`. Detalle: `references/custom-domain-operators.md`.
11. **v19 cambia lo que recibe un metodo `search=`** (medido): `("=", "AAA")` pasa a ser
    `("in", OrderedSet(["AAA"]))` — el optimizador de dominios normaliza antes de llamarte. El
    `value.lower()` de toda la vida muere con `AttributeError`, y **al filtrar, no al instalar**.
12. **`_inherits`: borrar el hijo deja al padre VIVO y huerfano** (medido); borrar el padre si se
    lleva al hijo (`ondelete='cascade'` del m2o). Buscar y ordenar por campo delegado si funcionan.

## NEVER

- NUNCA documentes firmas de API de memoria: verifica en el `fields.py`/`models.py`/`decorators.py` de la version.
- NUNCA uses `group_operator` en codigo nuevo v18/v19.
- NUNCA pongas `_sql_constraints` lista en un modulo v19 nuevo: usa `models.Constraint`.
- NUNCA escribas a un computed no-stored sin `inverse=`, ni lo filtres en domain sin `store=True`/`search=`
  (en v18 no te avisa: se salta el filtro y devuelve de mas), ni lo pongas en `_order`/`order=`.
- NUNCA uses `@api.onchange` para logica que deba persistir en `create`/`write` por RPC: usa compute.

## ALWAYS

- SIEMPRE codigo en ingles (identificadores, comentarios, docstrings, external IDs).
- SIEMPRE declara `@api.depends(...)` completo y correcto en computes stored (recompute incorrecto = bug silencioso).
- SIEMPRE usa `Command.*` en escrituras x2many de codigo nuevo.
- SIEMPRE verifica el fichero fuente exacto segun version antes de afirmar un nombre de atributo.
- **Provenance marker (ALWAYS):** every generated file (except `.csv`/`.json` — no comment syntax) ends with `Made with odoo-orm-modeler · v<version del frontmatter de esta skill>`, wrapped in the file-type comment, idempotent. Full rule: `references/shared/skill-provenance-marker.md`.

## Delegates to

| Skill | Para que |
|-------|----------|
| odoo-version-api-reference | Firmas exhaustivas de campos/decoradores/metodos ORM |
| odoo-module-creator | Scaffolding del modulo completo (manifest, security, views) |
| odoo-impact-analyzer | Impacto antes de cambiar/renombrar/borrar un campo en produccion |
| odoo-deep-knowledge | Quirks de compute/store/recompute y semantica ORM oculta |

## Consumed by

| Skill | Para que |
|-------|----------|
| odoo-module-creator | Diseno profundo de modelos durante el scaffolding |
| odoo-wizard-builder (developer) | Definir los campos del wizard (tipos, m2m/o2m, parametros) |

## REFERENCES

| Fichero | Contenido | Cargar cuando |
|---------|-----------|---------------|
| references/fields-and-attributes.md | Tipos de campo, atributos comunes, Properties, company_dependent, aggregator | Declarar/revisar campos |
| references/custom-field-types.md | Crear un tipo de campo/columna PROPIO v19 (`_column_type`, ddl verbatim, indices GIST fuera de banda, `_insert_cache`, compute store) | Necesitas un tipo de columna Postgres no estandar (PostGIS, etc.) |
| references/custom-domain-operators.md | Operador de dominio NO-estandar v19 (`@operator_optimization` + `Domain.custom`, `_field_to_sql`) | Necesitas un operador de busqueda propio (geo, fulltext custom, etc.) |
| references/compute-constraints-onchange.md | computes (stored/inverse/search/precompute/depends_context), constraints Py vs SQL, onchange | Logica de campos y validaciones |
| references/inheritance-and-relations.md | _inherit/_inherits/_name/Abstract/Transient, relaciones, Command.* | Elegir herencia y x2many |
| references/version-diffs.md | Diffs ORM v17->v18->v19 (incl. reestructura a odoo/orm/) con fichero:linea | Trabajar cross-version |
| `references/shared/organic-module-design.md` | Diseno organico: reutiliza antes de crear, monta sobre motores estandar, enchufa a flujos del ERP | Modelo/modulo nuevo (antes de declarar campos) |
