---
name: indaws-odoo-shell-scripting
version: 0.1.10
effort: medium
model: opus
description: >-
  Escribe scripts one-off seguros para odoo shell: correccion de datos, fix masivo, recompute, huerfanos, con dry-run. Usala cuando haya que tocar datos de produccion. Siempre con dry-run e idempotencia. NOT import masivo (→ indaws-data-import); NOT acciones de servidor reutilizables (→ odoo-server-action-creator).
allowed-tools: [Read, Write, Edit, Bash]
---

# Odoo Shell Scripting

Skill para escribir scripts seguros, idempotentes y reproducibles para ejecutar vía
`odoo shell`. Cubre dry-run, savepoints, batching, logging y control explícito de commit.

**Distinto de:**
- `indaws-data-import` → importación masiva desde ficheros externos
- `odoo-server-action-creator` → acciones automatizadas persistentes en UI
- `odoo-pilot` → operaciones via API externa (cuando no hay acceso a shell)

Referencias compartidas: `references/shared/odoo-connection.md`, `references/shared/batch-limits.md`

## Workflow

1. **Definir el cambio** — Preguntar: modelo afectado, dominio de filtro, campo(s) a modificar,
   versión de Odoo, entorno (staging/prod), si hay backup reciente.

2. **Leer patrones de seguridad** — Cargar [`references/safe-script-patterns.md`](references/safe-script-patterns.md)
   antes de escribir el script. Cargar [`references/shell-usage.md`](references/shell-usage.md)
   para saber cómo lanzar en la versión correcta.

3. **Escribir el script** — Usar el patrón obligatorio: flag `DRY_RUN=True`, savepoint,
   batching con commit periódico, log de conteos. Consultar
   [`references/recipes.md`](references/recipes.md) si el caso encaja en una receta conocida.

4. **Dry-run** — [STOP] Entregar script, instruir al usuario para ejecutarlo con `DRY_RUN=True`
   primero. Revisar el log de conteos antes de continuar.

5. **Ejecución real** — [STOP] Con `DRY_RUN=False`. Si el script usa `env.cr.commit()`,
   recordar que el shell NO hace auto-commit al salir; confirmación manual es obligatoria.

6. **Verificar** — Proveer consulta de verificación post-ejecución (count + sample de registros
   afectados) para que el usuario confirme el resultado.

## GOTCHAS

1. **`odoo shell` no hace commit automático** — Si el script no llama `env.cr.commit()`, los
   cambios se pierden al cerrar. Incluirlo explícitamente; el dry-run NUNCA debe llamarlo.

2. **`env` está disponible en el shell en TODAS las versiones (v14-v19)** — no hay frontera de
   versión. Misma línea `local_vars['env'] = env` en v14 (`odoo/cli/shell.py:116`, `Shell.ptpython` @cc0060e8`), v15
   (`:115 @3a28e5b0`), v16 (`:114 @b2f10d357`), v17 (`:116 @3e1233ed1`), v18 (`:117 @99f5a33c64`)
   y v19 (`:142 @d1d161033`). Y `self` es el registro del usuario (`local_vars['self'] = env.user`),
   así que `self.env` también funciona en todas. Ver `shell-usage.md`.

3. **Savepoint no protege contra OOM en batches grandes** — Si el dataset supera ~50k registros,
   dividir en sub-batches de ≤5k y hacer commit periódico, no un solo savepoint global.
   Los umbrales ~50k/≤5k son orientativos, no un límite del ORM.

4. **Invalidar la caché hace falta tras `cr.execute()`, NO tras un `write()` del ORM** — el
   `write()` del ORM actualiza la caché él mismo: *"This method must update the cache and prepare
   database updates"* (`odoo/orm/fields.py:1501-1503`, `Field.write`, v19 @d1d161033`). Leer justo después de un
   `write()` da el valor correcto. Lo que sí deja la caché mintiendo es el SQL crudo, que escribe
   por debajo del ORM: ahí `records.invalidate_recordset()` es obligatorio (ver gotcha 5).

5. **`_compute_*` no se dispara con SQL directo** — Si se usa `cr.execute()` para velocidad,
   los campos computados NO se recalculan automáticamente; añadir `recompute` explícito.

6. **Dominio vacío `[]` afecta TODOS los registros** — Verificar siempre que el dominio
   no sea accidentalmente vacío; añadir assertions de count antes del write.

7. **Producción sin backup** — Si el usuario no confirma backup, emitir advertencia bloqueante
   antes de generar el script ejecutable.

## NEVER

- NUNCA generar un script sin bloque `DRY_RUN` como primera protección.
- NUNCA usar `cr.execute()` con UPDATE directo sin documentar que bypasea ORM
  (ni triggers, ni recompute, ni mail tracking).
- NUNCA hacer `env.cr.commit()` dentro del bloque dry-run.
- NUNCA escribir sobre producción sin preguntar por backup.
- NUNCA ignorar errores de `write()` en batch — usar try/except con log por registro fallido.

## ALWAYS

- Incluir conteo previo y posterior en el log (`Before: N, After: M`).
- Parametrizar el dominio como variable al inicio del script (fácil revisión).
- Incluir cabecera de script con: modelo, propósito, autor, fecha, versión Odoo target.
- Usar `with env.cr.savepoint():` para operaciones atómicas dentro de un batch.
- Entregar siempre la query de verificación junto al script.

## Delegates to

| Skill | Suite | Cuándo |
|-------|-------|--------|
| odoo-pilot | common | Cuando no hay acceso a shell; operación vía API REST/JSON-RPC |
| odoo-version-api-reference | common | Verificar API ORM disponible en la versión target |
| indaws-data-import | developer | Si el caso es importación masiva desde CSV/Excel externo |

## Consumed by

| Skill | Suite | Para qué |
|-------|-------|----------|
| guion | — | Orquestador general de tareas de desarrollo |
| odoo-module-migration-scripts | developer | Transformación de datos puntual fuera del flujo de migración |

## REFERENCES

| Fichero | Contenido | Cargar cuando |
|---------|-----------|---------------|
| [references/shell-usage.md](references/shell-usage.md) | Cómo lanzar `odoo shell` por versión, flags, `env`/`self`, contexto, lang | Paso 2 siempre |
| [references/safe-script-patterns.md](references/safe-script-patterns.md) | Patrón dry-run, savepoint, batching, commit periódico, logging, idempotencia | Paso 2 siempre |
| [references/recipes.md](references/recipes.md) | Recetas: reasignar registros, recompute, corregir estados, set masivo, limpiar huérfanos | Paso 3 si el caso es reconocible |
