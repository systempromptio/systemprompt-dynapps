---
name: indaws-odoo-context-briefing
version: 1.0.4
effort: medium
model: opus
description: >-
  Briefing del Odoo de un cliente antes de tocar nada: que hay instalado, custom vs Studio vs estandar, colisiones e inventario vivo en .ai-memory/; en modo knowledge mantiene la KB verificada del custom e ingiere conocimiento humano (decisiones, reglas de negocio, actas). Usala cuando vayas a desarrollar o actualizar un modulo sobre su instancia, pidan contexto del proyecto o que hay instalado, o quieran guardar/consultar una decision o regla de negocio del custom. Triggers: "voy a desarrollar/tocar X", "que hay instalado", "dame contexto del proyecto", "ingiere esta acta", "por que decidimos X", "/odoo-context-briefing". Odoo 14-19.

---

# odoo-context-briefing

Skill orquestadora. Genera contexto previo a desarrollo Odoo y mantiene inventario incremental en `.ai-memory/inventory/` del repositorio del proyecto activo.

## Modos

| Modo | Pasos | Cuando |
|------|-------|--------|
| `quick` | 0, 1, 2, 4-light, 7 | "que hay instalado", chequeo rapido (incluye contadores Studio) |
| `incremental` | 0, 1, 2, 3-selectivo, 4b, 5, 7 | Refresh tras cambios — re-scan solo modulos con hash cambiado |
| `target <modulo>` | 0-7 con deep-dive solo del objetivo + dependencias inversas | "voy a tocar X" |
| `full` | 0-7 completo, todos los no-estandar | Primera vez / re-baseline |
| `studio-only` | 0, 4, 4b, 7 | Solo customizaciones Studio + extraccion de codigo |
| `knowledge` | K0-K3 (bundles + agente-por-tanda + verificacion vs fuente) | KB verificada del custom. **Explicito, nunca automatico** — no lo disparan full/target/incremental |

Default: si no existe `.ai-memory/inventory/SNAPSHOT.md` -> `full`. Si existe y el usuario no especifica modo, preguntar entre `quick`, `incremental` y `target <modulo>`. `knowledge` SIEMPRE es explicito (coste alto): solo si el usuario lo pide.

Ver `references/modes.md` para matriz completa.

## Pipeline

Paso 0 self_test → Pasos 1/4/1b en paralelo (SONNET: módulos, Studio, disk-vs-BD) → Paso 2 clasificar (OPUS) →
Paso 3 deep-dive N SONNET → Paso 4b extract_studio_code → Paso 5 collisions → Paso 6 dev-assessment (opcional) →
Paso 7 update_memory (OBLIGATORIO) → memory-check → render_briefing al usuario.

Diagrama completo + pipeline del modo `knowledge` (K0-K3) en `references/pipeline.md`; contratos de cada subagent en `references/pipeline-detail.md`.

## Estructura de `.ai-memory/` (OBLIGATORIO)

**`.ai-memory/` tiene dos subarboles complementarios + un punto de entrada:**

- `INDEX.md` — punto de entrada UNICO (generado). Cualquier consumidor empieza aqui.
- `inventory/` — mapa mecanico (este skill, todos los modulos). Regenerable barato.
- `knowledge/` — KB verificada contra fuente (modo `knowledge`, solo custom). Estable, cara.

**Precedencia ante conflicto**: gana `knowledge/` (verificado vs fuente; `inventory/` a veces hereda
README/manifest desfasados).

### Estructura de `inventory/`

**Ubicacion**: por defecto se crea en el **cwd actual**. NO se sube al git toplevel automaticamente.

Override del usuario:
- Flag `--inventory-dir <path>` o argumento explicito.
- Variable de entorno `ODOO_CONTEXT_INVENTORY_DIR`.
- Si el usuario dice "ponlo en X", obedecer.

Árbol: `$PWD/.ai-memory/` con `INDEX.md` (entrada única) + `inventory/` (SNAPSHOT.md, meta.json, modules/studio/collisions.yaml, deep/, studio_code/) + `knowledge/` (_index/symptoms.md, modules/<m>/core.md+gotchas.md, _bundles/).

Esquema completo de `inventory/` en `references/inventory-schema.md`; de `knowledge/` en `references/knowledge-mode.md`.

## Output al usuario

Markdown en 6 secciones (resumen ejecutivo, módulos relevantes, colisiones, recomendaciones, anomalías, sub-sistemas). Plantilla completa en `references/briefing-template.md`.

## Delegates to

> La conexión Odoo (pasos 1, 4, 4b) usa `odoo-pilot` en modo lectura según el contrato compartido `references/shared/odoo-connection.md`.

| Skill | Uso | Acceso |
|-------|-----|--------|
| `odoo-pilot` | Conexion + queries (pasos 1, 4, 4b) | **READ-ONLY** |
| `odoo-classification-guide` | Reglas standard/oca/indaws_core/custom/studio (paso 2) | local |
| `odoo-module-scanner` | Deep-dive por modulo (paso 3) | filesystem read |
| `odoo-deep-knowledge` | Enriquecer lista de modelos criticos (futuro) | local |
| `dev-assessment` | Evaluacion pre-desarrollo (paso 6, opcional) | local |
| `memory-check` | Validacion post-update (paso 7) | local |
| `tl-assistant` | Sugerir tareas a partir de colisiones high (futuro) | common |

## Consumed by

| Skill | Suite | Para qué |
|-------|-------|----------|
| `ticket-solver` | consultant | Contexto del custom antes de resolver un ticket |
| `odoo-solution-architect` | developer | Inventario + colisiones + KB antes de diseñar una tarea |

### Restriccion CRITICA sobre odoo-pilot

`odoo-pilot` se usa **EXCLUSIVAMENTE EN MODO LECTURA** (auth/search/read permitidos; create/write/unlink/copy y métodos que muten estado PROHIBIDOS). El inventariado es auditoria: NUNCA debe alterar la BD del cliente. Matriz permitido/prohibido en `references/read-only-contract.md`.

## Scripts (`scripts/`)

Catálogo completo (15 scripts, responsabilidad y flags) en `references/scripts.md`. Cada script usable con `--help`.

## GOTCHAS

22 gotchas operativos (odoo-pilot posicional, hash/Studio/delta, cross-mount Docker, resolver cache, ...) en `references/gotchas.md`.

## NEVER

- **Mutar la BD de Odoo bajo ningun concepto**.
- Saltarse el paso 7 (memory sync). La memoria SIEMPRE se refresca.
- Saltarse el paso 0 (self_test). Si falla, abortar con error claro.
- Escribir el briefing al usuario sin haber actualizado `.ai-memory/inventory/`.
- Hacer deep-dive de modulos `standard` u `oca` (desperdicio de tokens).
- Conectar a Odoo con codigo Python/curl ad-hoc — siempre via `odoo-pilot` en modo READ.
- Editar `.ai-memory/inventory/` a mano — todos los writes pasan por `update_memory.py`.
- Asumir que `studio_scan.sh` cubre TODAS las automations — verificar `total_automations_manual` en `studio.yaml/metadata`.

## ALWAYS

- Ejecutar `self_test.sh` antes de cualquier pipeline (paso 0).
- **Preflight de frescura git (paso 0, si el custom vive en repos):** `git -C <repo> fetch --quiet && git -C <repo> rev-list --left-right --count HEAD...@{u}`. Si va por detras del upstream → AVISAR (rama + N commits + `git log HEAD..@{u} --stat`) y recomendar `git pull` ANTES de inventariar. Nunca generar el briefing sobre codigo desfasado (caso real: inventario sobre checkout de 4 semanas → se perdio un modulo nuevo y una fase se marco mal).
- Validar conexion Odoo (`auth.sh`).
- Re-calcular `collisions.yaml` desde cero en cada run (no incremental, es derivado).
- Incluir delta + warnings en `SNAPSHOT.md`.
- Generar `meta.json` sidecar para consumo downstream sin re-parsear Markdown.
- Marcar modulos sin hash con `hash: unknown` y `risk: high`.
- Distinguir Studio automations `origin=studio` vs `origin=manual`.
- Incluir studio_code en el escritura via `--studio-code-dir` cuando exista.
- Usar `context: {lang: es_ES}` en TODAS las llamadas a Odoo.

## REFERENCES

- `references/inventory-schema.md` — Esquemas YAML completos (modules/studio/collisions/meta.json) y SNAPSHOT.md
- `references/classification-rules.md` — Reglas standard/oca/indaws_core/custom/studio
- `references/risk-scoring.md` — Algoritmo de risk: low/medium/high
- `references/memory-protocol.md` — Lifecycle, atomic writes, recovery
- `references/modes.md` — Matriz de comportamiento por modo (incluye `incremental`)
- `references/knowledge-mode.md` — Modo `knowledge`: pipeline, contrato de formato de gotchas, calibración
- `references/briefing-template.md` — Plantilla del Markdown final
- `references/studio-detection.md` — Identificar customizaciones Studio en BD por version
- `references/pipeline.md` — Diagrama ASCII completo del pipeline + modo `knowledge` (K0-K3)
- `references/pipeline-detail.md` — Contratos de cada subagent del pipeline
- `references/scripts.md` — Catálogo completo de los 15 scripts (responsabilidad + flags)
- `references/gotchas.md` — 22 gotchas operativos
- `references/read-only-contract.md` — Matriz permitido/prohibido de odoo-pilot (modo lectura)
