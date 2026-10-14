---
name: indaws-odoo-classification-guide
version: 1.0.11
effort: low
model: sonnet
description: >
  Fuente de verdad unica para clasificar funcionalidades Odoo en Standard/Studio/Core/Custom.
  Consultar siempre que se evaluen alternativas a desarrollo custom o se determine estrategia de
  migracion. Invocado por dev-assessment, odoo-database-analyzer y odoo-module-upgrader.
  Triggers: "clasificar modulo", "es estandar o custom", "hace falta desarrollo",
  "studio o desarrollo", "core o custom", "golden rules", "clasificacion odoo".
---

# Odoo Classification Guide

Fuente de verdad unica para clasificar funcionalidades y modulos Odoo. Este skill es de **conocimiento puro** — no ejecuta workflows, solo proporciona las reglas que otros skills aplican.

## Clasificaciones

Toda funcionalidad o modulo Odoo se clasifica en exactamente una de estas 4 categorias:

| Emoji | Clasificacion | Significado |
|-------|--------------|-------------|
| 🟢 | **Standard** | Funcionalidad nativa de Odoo en la version destino |
| 🟡 | **Studio** | Configurable via Odoo Studio (no-code/low-code) |
| 🟣 | **Core** | Cubierto por un modulo Indaws Core probado |
| 🔴 | **Custom** | Requiere desarrollo a medida |

## Principio: estándar primero, después código gobernado

**Estándar primero; después, el código más GOBERNADO** (1 técnico + IA + pipeline de calidad) frente a Studio ingobernable para lógica no trivial. La prioridad de clasificacion depende del contexto. Doctrina en [references/ai-impact-studio-vs-custom.md](references/ai-impact-studio-vs-custom.md).

### Contexto: Evaluacion de nueva funcionalidad
```
Estandar → Core → Studio → Custom
```
Core tiene prioridad sobre Studio porque es codigo probado, mantenido y migrable.

### Contexto: Migracion con refactoring
```
Estandar → Core → Studio → Custom
```
Misma prioridad. Core Supremacy aplica: si existe modulo Core, es OBLIGATORIO usarlo.

### Contexto: Migracion 1:1 (directa)
```
Estandar → Custom (SOLO estas dos opciones)
```
No se recomienda ni Core ni Studio en migracion directa.

### Contexto: Migracion sin Studio
```
Estandar → Core → Custom (sin Studio)
```
Lo que seria Studio se absorbe como Custom.

## Golden Rules — Consultar SIEMPRE

Lee [references/golden-rules.md](references/golden-rules.md) para las reglas obligatorias que se aplican ANTES de cualquier otra evaluacion.

## Decision Trees

Lee [references/classification-rules.md](references/classification-rules.md) para los arboles de decision detallados por contexto.

## Studio Capabilities

Lee [references/studio-capabilities.md](references/studio-capabilities.md) para la matriz de capacidades de Studio por version de Odoo y los umbrales Studio vs Desarrollo.

## Umbrales de Complejidad

Lee [references/complexity-thresholds.md](references/complexity-thresholds.md) para los umbrales que determinan si algo es desarrollo menor, estandar o complejo.

## Golden Examples (ancla anti-drift)

Lee [references/golden-examples.md](references/golden-examples.md): 2 casos input→clasificación congelados. Antes de tocar las reglas, confirma que estos ejemplos siguen dando el mismo resultado.

## Reglas Anti-Alucinacion

1. **Si no existe en Core, no lo inventes.** Clasifica como Studio (si permitido) o Custom.
2. **Si no esta en Release Notes, no es Standard.** Verifica siempre contra changelogs o documentacion.
3. **Usa SOLO clasificaciones permitidas por la estrategia.** 1:1 solo permite Standard y Custom. No-Studio prohibe Studio.
4. **Cada fila de modulo DEBE llevar emoji.** 🟢🟡🟣🔴 son obligatorios en toda tabla de clasificacion.
5. **No recomiendes modulos OCA ni de terceros.** Solo: Estandar, Studio, Indaws Core o desarrollo a medida.

## GOTCHAS
1. **Studio sirve para lo trivial, no por defecto.** Los campos Studio sobreviven upgrades y van bien para retoques simples (un campo, una vista, una automatizacion de un paso); pero para **logica no trivial un modulo gobernado gana** (revisable, testeable, y con IA migra sin problema). No clasifiques como Studio por "evitar codigo".
2. **Indaws Core modules have dependencies** — installing one may pull in 5 others. Check dependency tree first.
3. **"Standard" doesn't mean "free"** — Enterprise features require Enterprise license. Classify CE vs EE.
4. **Studio customizations are database-specific** — they don't travel with code. Export as module for portability.
5. **Acoplarse fuerte a un Core al extenderlo es anti-patron** — extiende via un **modulo gobernado nuevo** (Studio solo para el retoque trivial de vista/campo).

For shared ecosystem patterns, see `common:references/shared/`.

## Delegates to
| Skill | Suite | When |
|-------|-------|------|
| odoo-effort-estimator | common | Equivalencia complejidad↔horas (clasificacion de complejidad para Custom) |
| content-collection | common | Indices de modulos Indaws Core para la regla Core Supremacy |

## Consumed by
| Skill | Suite | For what |
|-------|-------|----------|
| odoo-database-analyzer | common | Module classification in migrations |
| dev-assessment | common | Evaluating development requests |
