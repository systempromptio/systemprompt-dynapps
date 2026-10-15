# Modos de ejecucion

Matriz de comportamiento por modo. Cada modo decide qué pasos del pipeline ejecutar y cuanta profundidad aplicar.

## Tabla resumen

| Paso | quick | incremental | target `<mod>` | full | studio-only |
|------|-------|-------------|----------------|------|-------------|
| 0. self_test (entorno) | ✅ | ✅ | ✅ | ✅ | ✅ |
| 1. Inventario via odoo-pilot | ✅ | ✅ | ✅ | ✅ | ❌ |
| 1b. disk_vs_installed | ❌ | ✅ | ✅ | ✅ | ❌ |
| 2. Clasificar | ✅ | ✅ | ✅ | ✅ | ❌ |
| 3. Deep-dive modulos no-estandar | ❌ | Solo modulos con hash cambiado | Solo target + deps inversas + hash-changed | TODOS los no-estandar | ❌ |
| 4. Studio scan | ✅ (`--quick`: solo counters) | ✅ | ✅ | ✅ | ✅ |
| 4b. Extraccion de codigo Studio | ❌ | ✅ | ✅ | ✅ | ✅ |
| 5. Compute collisions | ❌ | ✅ | ✅ | ✅ | ❌ (necesita modules.yaml — usa el previo) |
| 6. Dev-assessment | ❌ | ❌ | Solo si usuario describio funcionalidad | ❌ | ❌ |
| 7. Memory sync (OBLIGATORIO) | ✅ | ✅ | ✅ | ✅ | ✅ |
| Briefing al usuario | minimal+studio | delta-focused | target-focused | full | studio-focused |

## Cuando usar cada modo

### `quick`

- Pregunta "que hay instalado" sin intencion de modificar.
- Refresh rapido del inventario (delta vs ultimo scan).
- **Incluye contadores Studio** (light scan: counts + lista de campos, sin code).
- Coste: 3 subagents (inventory + studio-light + memory-check). ~bajo.

Output al usuario: tabla resumen por tipo + Studio counts + delta. Sin deep-dives. Sin colisiones recalculadas.

### `incremental`

- Refresh ligero entre `quick` y `full`. Util cuando quieres re-detectar cambios reales.
- Re-escanea solo modulos cuyo hash ha cambiado vs el scan anterior.
- Reaprovecha `studio.yaml` y `modules.yaml` previos para los inalterados.
- Coste: 2 + N (N = modulos con hash cambiado). Tipico: 0-5 modulos.

Output: delta-focused — qué cambió desde el último scan.

### `target <modulo>`

- "Voy a tocar X", "voy a actualizar X", "antes de modificar X".
- Deep-dive del modulo objetivo + sus dependencias inversas (BFS sobre `depends`).
- Re-escanea cualquier modulo no-estandar cuyo hash haya cambiado desde el ultimo scan.
- Calcula colisiones completas.
- Si el usuario describio funcionalidad nueva en su mensaje, ejecuta dev-assessment.

Coste: 2 + N (donde N = modulos a re-scanear). Tipicamente ~5-8 subagents.

Output: foco en el modulo objetivo, sus deps inversas, colisiones que le afectan, recomendaciones.

### `full`

- Primera vez en un proyecto.
- Re-baseline (cuando se sospecha que el inventario tiene drift grande).
- Re-escanea TODOS los modulos no-estandar.
- Coste: 2 + M (M = total no-estandar). Tipicamente 15-30 subagents.

Output: briefing completo con top-N riesgos, todas las colisiones, snapshot del proyecto.

### `knowledge`

- Genera/actualiza la **KB verificada del custom** en `.ai-memory/knowledge/` (analogo por-cliente
  de `odoo-deep-knowledge`): `core.md` + `gotchas.md` por modulo, anclados y verificados vs fuente.
- **Explicito, nunca automatico**: no lo disparan `full`/`target`/`incremental` (coste alto). Solo si
  el usuario lo pide ("genera/actualiza el knowledge del custom").
- Requiere que `inventory/modules.yaml` exista (de el saca la lista de custom + source_path).
- Pipeline K0-K3: bundles (sin LLM) -> 1 agente por tanda (verifica anclas) -> indice determinista.
- Coste: ~1 agente por tanda de modulos (enrutar por tamaño). Es la operacion mas cara del skill.

- **Conocimiento humano** (decisiones, reglas de negocio, actas que NO están en el código): dropea la
  fuente en `knowledge/raw/`, compílala a `knowledge/domain/<topic>.md` citando el raw, anota en
  `knowledge/log.md` (append-only) y regenera índice. Triggers: "ingiere esta acta/decisión al
  knowledge", "guarda esta regla de negocio", "por qué decidimos X". Es el complemento humano a la KB
  de código (modelo wiki tipo Karpathy: raw→wiki+index+log, con `--lint` de consistencia).
- **Lint** de consistencia tras ingerir: `gen_knowledge_index.py --ai-memory <path> --lint`.

Output: `knowledge/modules/<m>/core.md`+`gotchas.md`, `knowledge/domain/<topic>.md`,
`knowledge/raw/`, `knowledge/log.md`, `knowledge/_index/symptoms.md`, `INDEX.md`.
Consumidor: `ticket-solver` (lee `.ai-memory/knowledge/` via `read_inventory.py --symptom/--gotchas/--core`).

Detalle en `knowledge-mode.md`.

### `studio-only`

- "Que customizaciones Studio hay", auditoria Studio.
- Solo paso 4 + 7. Reutiliza `modules.yaml` previo si existe (no re-inventaria modulos).
- Coste: 1 subagent (studio_scan). ~muy bajo.

Output: bloque Studio focalizado (campos, server actions, automations, modelos custom).

## Inferencia de modo desde el mensaje del usuario

Si el usuario no especifica modo, inferir:

| Mensaje contiene | Modo inferido |
|------------------|---------------|
| "voy a desarrollar/tocar/actualizar/modificar `<X>`" + nombre de modulo identificable | `target <X>` |
| "voy a desarrollar `<funcionalidad>`" sin nombre de modulo concreto | `target <best-guess>` con dev-assessment ON |
| "que hay instalado", "inventario", "que modulos" | `quick` |
| "refresh", "actualiza el inventario", "que ha cambiado" | `incremental` |
| "que hay de Studio", "customizaciones Studio", "code de Studio" | `studio-only` |
| "primera vez", "baseline", "todo el proyecto" | `full` |
| Ambiguo + existe `.ai-memory/inventory/SNAPSHOT.md` | preguntar al usuario entre `quick`, `incremental` y `target` |
| Ambiguo + NO existe inventario | `full` (forzado) |

## Flags adicionales

| Flag | Efecto |
|------|--------|
| `--no-deep` | Fuerza saltar paso 3 incluso en target/full (debug, output minimal) |
| `--no-studio` | Salta paso 4 (proyectos sin Studio o version <16) |
| `--force-rescan` | Ignora hashes y re-escanea todo lo no-estandar |
| `--dev-assess "<descripcion>"` | Fuerza paso 6 con la descripcion dada como input |
| `--config <path>` | Path alternativo al `.config.yaml` (modelos criticos extra, etc.) |

## Default flow

```
1. ¿Usuario especifico modo?  -> usar ese
2. ¿Inferible del mensaje?    -> usar inferido
3. ¿Existe SNAPSHOT.md previo? 
   - SI  -> preguntar quick vs target
   - NO  -> full (forzado, primera vez)
```

## Promesa de freshness

Sea cual sea el modo, **el paso 7 (memory sync) siempre se ejecuta**. Eso garantiza:
- `.ai-memory/inventory/` tiene fecha del scan actual
- delta calculado y persistido
- hash global actualizado
- memory-check (si esta) valida coherencia

Si el paso 7 falla, el briefing al usuario incluye un warning rojo y NO se actualiza SNAPSHOT (rollback).
