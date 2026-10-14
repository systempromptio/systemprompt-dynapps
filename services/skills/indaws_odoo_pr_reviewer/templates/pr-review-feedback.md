# Review del PR — Correcciones · {título de la tarea}

**PR**: {url} | **Tarea**: {project.task id/nombre} | **Rama**: `{rama}` | **Revisor**: {arquitecto}
**Veredicto**: ❌ Correcciones requeridas (no merge)

## Resumen

{1-2 frases: qué está bien y qué bloquea el merge.}

## Bloqueantes (impiden merge)

1. **{qué incumple}** — incumple {criterio/contrato del MD}. Se espera: {qué}. Dónde: `{file:line}`.
2. …

## Revisión manual obligatoria (no lo garantiza la automatización)

> Disparadores de `manual-review-triggers.md` que aplican a este PR. Bloquean el merge hasta firma humana.

1. **{disparador — p.ej. cálculo de impuestos}** — un test verde no prueba la corrección fiscal.
   Debe revisarlo: **{consultor contable}**. Estado: {pendiente/firmado}.

## Mayores (arreglar antes de aprobar)

1. **{finding}** — {por qué} → {qué se espera}.

## Menores (opcionales, no bloquean)

- {sugerencia de estilo/nombre/optimización}.

## Tests

- Criterios cubiertos: {X/Y}. Faltan: {lista}.
- Unit: {estado} · e2e: {estado} · Verde real (CI/ejecución): {sí/no + enlace}.

## Lo que está bien

- {reconocer lo correcto — review educativo}.

## Siguiente paso

Corrige los **bloqueantes** (y mayores), vuelve a lanzar los tests y **notifícanos de nuevo en el
PR**. Si crees que algún punto es del contrato y no tuyo, dilo: lo revisamos con el arquitecto.

---
*(Si el fallo raíz es del contrato/diseño → este caso se escala a `odoo-solution-architect`, no es
responsabilidad del dev.)*
