---
name: indaws-odoo-performance-profiler
version: 0.1.15
effort: high
model: opus
description: >-
  Encuentra y corrige lentitud en Odoo: N+1, browse en bucle, computed sin store, indices, informes lentos. Usala cuando algo vaya lento o haya que optimizar. Mide antes y después (query count, timing, py-spy): queries excesivas, prefetch, read_group, escrituras no batched, SQL lento.
allowed-tools: [Read, Grep, Glob, Bash, Edit, Write, Agent, Skill]
---

# Odoo Performance Profiler

Diagnostica y corrige cuellos de botella de ORM/runtime en Odoo v17-v19. Principio
rector: **medir, no adivinar**. Cada refactor se justifica con una medicion antes
y se valida con una medicion despues (query count o timing reproducible).

Refs compartidas: `references/shared/odoo-connection.md`,
`references/shared/batch-limits.md`.

## Workflow

1. **Identificar el hotspot.** Parte del sintoma (request lento, cron, informe,
   accion de servidor) o del log. Localiza el codigo concreto con Grep/Read. Si no
   hay sintoma claro, escanea estaticamente con [delega] `odoo-code-auditor`
   (focus Performance) para sacar candidatos.
2. **Medir el baseline.** Carga `references/measurement-tools.md`. Elige tecnica
   segun el caso: `--dev=all` + logging SQL, contador de queries, `assertQueryCount`,
   o py-spy para CPU. Para medir en vivo contra una instancia usa [delega]
   `odoo-pilot` (ejecutar metodo y cronometrar). Anota numeros: queries, ms, picos.
3. **Diagnosticar el patron.** Carga `references/orm-antipatterns.md` y clasifica
   el hallazgo (N+1, browse en bucle, computed sin store, dominio costoso, etc.).
   Confirma con el codigo; no asumas. Verifica la API de la version con [delega]
   `odoo-version-api-reference` antes de proponer cambios (firmas cambian v17-v19).
4. **Refactorizar.** Carga `references/optimization-patterns.md` y aplica el patron
   correcto (prefetch/`mapped`, `store`+`@api.depends`, `index=True`, batch
   read/write, `read_group`, vista SQL, control de `flush`/recompute). Edita el
   codigo directamente.
5. **Verificar la mejora.** Re-mide con la MISMA tecnica del paso 2 y compara. Si
   no hay mejora medible, revierte. Genera o actualiza un test de regresion con
   [delega] `odoo-test-generator` (`assertQueryCount` + `@warmup`) para fijar la
   ganancia.

## GOTCHAS

1. **No optimices sin medir.** Un refactor "obvio" puede no mover la aguja o
   empeorar. Sin baseline, no hay caso.
2. **El primer hit calienta caches.** Mide con `@warmup` o descarta la primera
   pasada; si no, cuentas queries de prefetch/registry que no se repiten.
3. **`store=True` tiene coste de escritura.** Almacenar un computed acelera lectura
   pero recalcula en cada write del depends; mide ambos lados antes de decidir.
4. **`index=True` no es gratis.** Acelera filtros pero ralentiza inserts y ocupa
   disco. Solo en campos realmente filtrados/ordenados con cardinalidad util.
5. **`read_group` vs sumar en Python.** Agregar en BD evita traer miles de records;
   pero para pocos registros ya en cache, el overhead no compensa.
6. **El prefetch se rompe al iterar fuera del recordset.** Acceder a campos sobre
   records sueltos (tras `filtered`/list comprehension que desagrupa) dispara N+1.
7. **SQL crudo evita reglas de acceso y `flush`.** Lee datos sucios si hay cambios
   pendientes en el cache ORM; haz `flush_model()`/`flush_recordset()` antes.
8. **`with_context(prefetch_fields=False)`** o dominios sobre campos no almacenados
   fuerzan recomputos en bucle: detecta computed-no-store usados en `search`.
9. **A escala, un `write` masivo dispara recompute storms.** Un `@api.depends`
   ancho/profundo recorre el arbol de triggers y marca to-recompute via `inverse()`
   por cada salto; el conjunto a recalcular puede exceder los records escritos.
   Ver `production-scale.md` A1. Acota el `depends`; no flushees por iteracion.
10. **`tracking=True` + carga masiva = write amplification.** Cada write trackeado
    genera `mail.message`/`mail.tracking.value`. En imports/migraciones usa
    `with_context(tracking_disable=True)` (o `mail_notrack=True`). Ver A6.
11. **El problema a escala suele estar en Postgres, no en Python.** Empieza por
    `pg_stat_statements` -> `EXPLAIN (ANALYZE, BUFFERS)`; si es Seq Scan, falta indice.

## NEVER

- NUNCA declarar una mejora sin re-medir con la misma tecnica que el baseline.
- NUNCA hacer `search`/`browse`/`read` dentro de un bucle sobre records.
- NUNCA reimplementar la conexion a Odoo: usa `odoo-pilot`.
- NUNCA documentar API inline: referencia `odoo-version-api-reference`.
- NUNCA añadir `index=True` o `store=True` "por si acaso" sin medir el trade-off.
- NUNCA escribir codigo, identificadores o comentarios fuera de ingles.

## ALWAYS

- SIEMPRE capturar baseline (queries/ms) antes de tocar nada.
- SIEMPRE clasificar el antipatron contra `references/orm-antipatterns.md`.
- SIEMPRE batchear lecturas/escrituras y usar `mapped`/`read_group` cuando aplique.
- SIEMPRE dejar un test `assertQueryCount` que fije la ganancia (via `odoo-test-generator`).
- SIEMPRE verificar la firma de API contra la version objetivo (v17-v19).

## Delegates to

| Skill | Suite | Cuando |
|-------|-------|--------|
| odoo-code-auditor | developer | Chequeos de performance estaticos para sacar candidatos |
| odoo-test-generator | developer | Tests `assertQueryCount` / `@warmup` de regresion |
| odoo-version-api-reference | common | Firmas/diferencias de API ORM por version (v17-v19) |
| odoo-pilot | common | Medir en vivo: ejecutar metodo, cronometrar, contar queries |
| odoo-deep-knowledge | odoo-knowledge | Gotchas de rendimiento conocidos (computes, prefetch, indices) |

## Consumed by

| Skill | Suite | Para que |
|-------|-------|----------|
| guion | developer | Paso de optimizacion dentro de un guion de desarrollo |

## REFERENCES

| Fichero | Contenido | Cargar cuando |
|---------|-----------|---------------|
| [references/orm-antipatterns.md](references/orm-antipatterns.md) | Catalogo de antipatrones ORM: sintoma y por que es lento | Paso 3 (diagnosticar) |
| [references/measurement-tools.md](references/measurement-tools.md) | Como medir: `--dev=all`, SQL logging, contador de queries, `assertQueryCount`, py-spy, perfilado de requests/informes | Paso 2 y 5 (medir) |
| [references/optimization-patterns.md](references/optimization-patterns.md) | Patrones de fix: prefetch/`mapped`, `store`+`@api.depends`, indices, batch read/write, `read_group`, vistas SQL, control de `flush`/recompute | Paso 4 (refactorizar) |
| [references/production-scale.md](references/production-scale.md) | Perf y depuracion a ESCALA DE PRODUCCION (BD grandes): fallos que solo emergen a volumen (recompute storms, N+1 a escala, indices, write amplification de mail.thread/automations/crons, informes) + como diagnosticar con y sin acceso a prod (pg_stat_statements, EXPLAIN, flags reales, py-spy) | Sintoma a escala / BD grande de cliente / debug en prod |
