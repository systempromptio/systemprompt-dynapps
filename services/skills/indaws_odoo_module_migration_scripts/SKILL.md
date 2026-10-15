---
name: indaws-odoo-module-migration-scripts
version: 0.1.10
effort: medium
model: opus
description: >-
  Escribe scripts de migracion intra-modulo de Odoo (migrations/, pre- y post-migrate, migrate(cr, version)). Usala cuando renombres o recomputes campos al actualizar. NOT portar código entre majors (→ odoo-module-upgrader) ni upgrade de BD (→ odoo-database-upgrader).
allowed-tools: [Read, Grep, Glob, Bash, Edit, Write]
---

# Odoo Module Migration Scripts

Scripts de migracion **intra-modulo**: el codigo Python que Odoo ejecuta cuando
detecta que la version del `__manifest__.py` es MAYOR que la `installed_version`
de un modulo durante un `-u <modulo>`. Sirven para transformar datos/schema que
el ORM no migra solo (renames, recomputos, limpiezas). Lenguaje del codigo:
ingles SIEMPRE. Ver `references/shared/coding-standards.md` y
`references/shared/odoo-connection.md`.

## Frontera (leer antes de actuar)

| Tarea | Skill correcto |
|-------|----------------|
| Transformar datos/schema al subir la version del MODULO (`-u`) | **esta skill** |
| Portar el codigo del modulo de un major de Odoo a otro (XML/Py/JS/OWL) | `odoo-module-upgrader` |
| Subir la BASE DE DATOS entre versiones via Upgrade Service | `odoo-database-upgrader` |

Si la peticion es "migrar de v17 a v18" sin mas, NO es esta skill: clarifica.

## Workflow

1. **Identifica fase.** Cambio de schema/columna -> `pre-`. Datos que dependen de
   la nueva estructura/campos/data del modulo -> `post-`. Algo que depende de que
   TODOS los modulos esten cargados -> `end-`. Detalle y orden exacto en
   [references/mechanics.md](references/mechanics.md).
2. **Decide naming de carpeta.** `migrations/<module_version>/` donde
   `<module_version>` coincide con (o es <=) la version del manifest. Caso `0.0.0`
   = corre en CUALQUIER cambio de version. Reglas en `references/mechanics.md`.
3. **Escribe `migrate(cr, version)`.** Recibes el CURSOR, no env. Para ORM en
   `post`/`end` construye `env = api.Environment(cr, SUPERUSER_ID, {})`. SQL vs ORM,
   idempotencia y recetas concretas en [references/recipes.md](references/recipes.md).
4. **Usa helpers reales.** No inventes nombres: usa `odoo.tools.sql`
   (`column_exists`, `rename_column`, `table_exists`...). Inventario por version y
   nota sobre `openupgradelib` (externo) en
   [references/core-util-and-openupgradelib.md](references/core-util-and-openupgradelib.md).
5. **Verifica diffs de version** si el modulo es v17 o v19 en
   [references/version-diffs.md](references/version-diffs.md).
6. **Sube el manifest.** El script NO corre si no incrementas `version` en
   `__manifest__.py` (salvo `0.0.0`, que igualmente exige version distinta).
7. **Valida el script [VALIDADOR determinista, ~0 tokens LLM].**
   `scripts/check_migration_script.py <fichero|migrations_dir> --sql-source <odoo>/odoo/tools/sql.py --md`
   → lintea las reglas NEVER/ALWAYS (firma `migrate(cr, version)`, SQL parametrizado, sin `cr.commit()`,
   helpers de `odoo.tools.sql` reales, idempotencia, `flush_all`). Arregla lo CRÍTICO antes de entregar.
   Escribir el script es juicio del LLM; validarlo es determinista. **Fallback:** revisar a mano con `references/`.

## GOTCHAS

1. **`migrate` recibe `cr`, NO `env`.** En `pre` el registry aun no esta completo
   para ese modulo; construir env ahi es fragil. En `post`/`end` si:
   `from odoo import api, SUPERUSER_ID; env = api.Environment(cr, SUPERUSER_ID, {})`.
2. **`pre-` corre ANTES de cargar la data del modulo y ANTES del update de schema
   ORM** del propio modulo (loading.py:182, antes de `init_models` en :206). Los
   campos/columnas nuevos del manifest aun NO existen en `pre`.
3. **`post-` corre DESPUES de `init_models` y DESPUES de cargar la data** (xml/csv)
   del modulo (loading.py:228 luego :233). Ahi ya tienes columnas y registros nuevos.
4. **`end-` corre cuando TODOS los modulos ya estan actualizados y data cargada**
   (loading.py STEP 3.5, :520-523), tras el bucle de carga completo. Unico sitio
   seguro para tocar datos de OTROS modulos.
5. **El script NO corre si no subes `version` del manifest.** El gate es
   `installed_version < version_manifest` (migration.py:216).
6. **`0.0.0` corre en cada cambio de version** (migration.py:203-204): en `pre` el
   PRIMERO, en `post`/`end` el ULTIMO. Para invariantes idempotentes.
7. **`noupdate="1"`:** la data noupdate NO se recarga en `-u`; si un `post` depende
   de un xmlid que solo se crea con noupdate, puede no existir aun en re-ejecucion.
8. **No asumas orden de otros modulos en `pre`/`post`.** Solo el grafo de
   dependencias garantiza orden; cruzar modulos => usa `end`.

9. **Antes de escribir la migracion, comprueba que alguien tiene la version vieja.**
   Un modulo recien construido que **ninguna base tiene instalado** no necesita script: se cambian
   los campos y una instalacion limpia ya crea el esquema bueno. Mantener el script no es gratis
   -- suele ser lo mas arriesgado del PR (renombres a pelo, `DELETE FROM ir_model_fields`).
   La comprobacion es una linea, por cada base candidata **incluida staging**:
   ```sql
   SELECT name, latest_version, state FROM ir_module_module WHERE name = 'mi_modulo';
   ```
   Si sale `uninstalled` en todas, el script sobra. Caso medido (2026-08-12): un modulo con dos
   scripts de migracion cuya unica base con datos era una copia de test.
10. **Renombrar un campo SIN migracion no falla: pierde los datos en silencio.**
   Odoo nunca borra la columna vieja ni copia nada: crea la nueva **vacia** y abandona la anterior.
   El `-u` sale en verde y la clasificacion, las marcas y las referencias quedan a NULL. Por eso la
   pregunta del punto anterior se contesta ANTES de decidir, no despues: si hay una sola base con
   el modulo puesto, el script pasa de sobrar a ser imprescindible.

## NEVER

- NUNCA construyas SQL con f-strings/`%`-format de valores; usa `cr.execute(query, params)`
  parametrizado. (`%s` con tupla de params es seguro.)
- NUNCA asumas que un campo/columna nuevo del manifest existe en `pre-migrate`.
- NUNCA inventes funciones util (`openupgrade.rename_*` no existe en core);
  verifica el nombre en `odoo.tools.sql` o en `openupgradelib` (externo).
- NUNCA dejes un script no idempotente: un `-u` puede re-ejecutarse; guarda con
  `column_exists`/`if not env...`/`ON CONFLICT`.
- NUNCA toques datos de otros modulos en `pre`/`post`; usa `end`.
- NUNCA hagas `cr.commit()` dentro de un script de migracion (rompe la transaccion
  del upgrade).

## ALWAYS

- SIEMPRE firma exacta `def migrate(cr, version):` (en v18/v19 una firma distinta
  LEVANTA TypeError; ver version-diffs).
- SIEMPRE haz el script re-ejecutable (idempotente).
- SIEMPRE `env.flush_all()` tras `add_to_compute`/writes ORM en `post`/`end`.
- SIEMPRE comprueba existencia antes de rename/drop (`column_exists`, `table_exists`).
- SIEMPRE sube el numero de `version` en el manifest a la version de la carpeta.
- **Provenance marker (ALWAYS):** every generated file (except `.csv`/`.json` — no comment syntax) ends with `Made with odoo-module-migration-scripts · v<version del frontmatter de esta skill>`, wrapped in the file-type comment, idempotent. Full rule: `references/shared/skill-provenance-marker.md`.

## Delegates to

| Skill | Cuando |
|-------|--------|
| odoo-version-api-reference | Firmas/cambios de API por version (no inline a ciegas) |
| odoo-shell-scripting | Transformacion de datos puntual FUERA del flujo de migracion |
| odoo-impact-analyzer | Medir el impacto/dependencias de un rename antes de escribirlo |
| odoo-deep-knowledge | Comportamiento de datos cross-version al migrar |

## Consumed by

| Skill | Para que |
|-------|----------|
| odoo-module-upgrader | Cuando una migracion de major necesita scripts intra-modulo |
| odoo-database-upgrader | Scripts que acompanan el upgrade de BD |

## REFERENCES

| Fichero | Contenido |
|---------|-----------|
| [scripts/check_migration_script.py](scripts/check_migration_script.py) | **Validador determinista** del `migrate()` contra las reglas NEVER/ALWAYS (JSON/`--md`) |
| [references/mechanics.md](references/mechanics.md) | MigrationManager, descubrimiento, fases y orden exacto, naming de version, caso 0.0.0, firma |
| [references/recipes.md](references/recipes.md) | Recetas idempotentes: rename columna/campo/modelo, mover datos, recomputar stored, cleanup, set masivo, SQL vs ORM |
| [references/core-util-and-openupgradelib.md](references/core-util-and-openupgradelib.md) | Helpers reales de `odoo.tools.sql` por version + nota openupgradelib |
| [references/version-diffs.md](references/version-diffs.md) | Diffs v17 / v18 / v19 del mecanismo y de los helpers |
| references/shared/coding-standards.md | Estandar de codigo (ingles obligatorio) |
| references/shared/odoo-connection.md | Conexion a Odoo |
