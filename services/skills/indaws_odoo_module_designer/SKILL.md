---
name: indaws-odoo-module-designer
description: Convierte una idea o plan de un módulo/producto Odoo CUSTOM NUEVO (o una suite de varios módulos) en un paquete listo para desarrollar — decisiones de diseño cerradas y trazables, mapa de reuso del parque, arquitectura, fasing por fases/commits, scaffold multi-módulo y set de tareas. Es nivel PRODUCTO/multi-módulo (breadth), aguas arriba de odoo-solution-architect (que diseña el contrato de UNA tarea). Úsalo cuando el usuario diga "quiero diseñar un módulo nuevo", "convierte esta idea/plan en desarrollo", "cerremos el diseño de este producto", "descompón esto en módulos y tareas", "cómo estructuro esta suite de módulos". NOT clasificar si hace falta custom (→ dev-assessment) · NOT redactar una tarea suelta (→ dev-task-writer) · NOT generar el código real (→ odoo-module-creator).
version: 1.0.1
effort: high
model: opus
allowed-tools: [Bash, Read, Write, Edit, Glob, Grep, AskUserQuestion, Agent, Skill]
---

# Odoo Module Designer v1.0

Cierra el diseño de un módulo/producto Odoo custom **nuevo** desde una idea o plan, y lo deja descompuesto,
scaffoldeado y listo para tareas. **Diseña, no cementa** (planos y decisiones, no código de solución).

> **Frontera (crítica, va en las descriptions para no confundir al router):**
> - **breadth** — `odoo-module-designer` (esta): idea/plan → diseño de UN producto/varios módulos.
> - **depth** — `odoo-solution-architect`: 1 tarea acotada → su contrato/planos.
> - upstream `dev-assessment` (¿custom?) · downstream `dev-task-writer` (redacta cada tarea).

## Regla de ORDEN (lo que evita reprocesos — obligatorio)

El orden importa más que la exhaustividad. Aprendizaje de campo: preguntar el detalle antes que la forma,
o scaffoldear antes de mirar el reuso, obliga a reescribir arquitectura y revisar manifests dos veces.

1. **FORMA primero** — premisa de producto + arquitectura ANTES que cualquier Q&A de detalle. Si el
   producto es "configurable" o "de simple a complejo", decide YA el mecanismo (módulos por check
   `module_*` vs gating por grupos) — condiciona todo lo demás.
2. **REUSO segundo** — scan del parque ANTES de fijar dependencias y ANTES del scaffold.
3. **DETALLE después** — Q&A acotado: solo decisiones load-bearing. Los detalles de implementación se
   deciden CONSTRUYENDO, no en preguntas (no eleves cada detalle a pregunta → parálisis).

## Workflow

### Step 0 — Gate + intake [inline]
- Delega a **`dev-assessment`** si no está confirmado que es custom. Si Standard/Studio/Core lo cubre → PARA, no se diseña.
- Recoge: versión Odoo objetivo, premisa de negocio, si es 1 módulo o suite, fuentes/plan previo.

### Step 1 — FORMA: premisa + arquitectura [OPUS · AskUserQuestion]
- `AskUserQuestion` sobre la **forma del producto**: ¿configurable simple→complejo? ¿mecanismo (módulos por
  check vs grupos)? ¿granularidad (fusiona lo que va siempre junto — ver `references/design-loop.md`)? ¿multi-company?
- Verifica el mecanismo elegido contra fuente (p.ej. patrón `module_<x>` en `res.config.settings`) — **nunca de memoria**.
- Salida: mapa de módulos (base + capacidades + bridges) y su cadena de deps.

### Step 2 — REUSO: scan del parque [SONNET SUBAGENT]
- Lanza `Agent(model: sonnet)` (o delega a **`odoo-module-scanner`** en modo parque) para buscar en el
  código custom del cliente/suite qué YA existe (mixins, modelos, bridges) y evitar reinventar.
- Salida: tabla de reuso {pieza, aporta, encaje, confianza} → **ajusta las deps del Step 1** antes de scaffoldear.

### Step 3 — DETALLE: BUCLE iterativo de Q&A hasta cerrar gaps [OPUS · AskUserQuestion] ⭐ NÚCLEO DE VALOR
Esta es la mecánica central de la skill. **La SKILL genera las preguntas** (el usuario no tiene que pensar
qué preguntar — solo elige o escribe). Bucle:
1. **La skill propone la ronda de preguntas** de la capa actual (alcance → diseño de datos → comportamiento),
   cada una con opciones sensatas + recomendación, y las lanza **una a una** con `AskUserQuestion` (elegir o escribir).
2. Cada respuesta = una decisión con **ID `D##`**, verificada a fuente si es versión-específica, aplicada
   al doc **fuente única de verdad** (anota las derogadas, no las dejes vivas en tablas viejas).
3. **Entre rondas, análisis de gaps**: la skill se pregunta "¿qué queda abierto que bloquee una fase? ¿qué
   nuevas preguntas destapa lo ya decidido?" y **propone la siguiente ronda**. Un cambio de arquitectura a
   mitad reabre decisiones → recíclalas explícitamente.
4. **Convergencia**: repite rondas hasta que **ningún gap bloquee ninguna fase**. Criterio de parada honesto:
   si la siguiente ronda serían detalles que un dev resuelve solo (matriz ACL, nombres, N de un aviso) → PARA.
   "Acotado" NO significa "pocas preguntas": significa las correctas, en orden, hasta cerrar — pueden ser muchas.
- **Health-check** (🟢/🟠/🔴) por dimensión (decisiones · consistencia · riesgo técnico · preparación).
  Distingue lo que **cierra una pregunta** de lo que **solo se cierra construyendo** — no lo pintes verde.
- Detalle del bucle, IDs, health-check y fuente única en `references/design-loop.md`.

### Step 4 — Fasing + scaffold [OPUS + script]
- Fasing estilo obra (cimientos→…→entrega) con **rama única + commit por bloque, push por fase**.
- Genera el **scaffold multi-módulo** (manifests con deps del Step 1/2, seguridad, modelos con campos
  declarados, `TODO` en la lógica) con un script; **valida sintaxis** (py_compile + XML + manifests). Ver `references/scaffold-and-phasing.md`.

### Step 5 — Tareas [delega]
- Delega a **`dev-task-writer`** una tarea por módulo (o por fase), pasando las decisiones `D##`, el fasing y las deps.
- Para el contrato profundo de un módulo de riesgo, delega a **`odoo-solution-architect`**.

## NEVER
1. Q&A de detalle antes de cerrar la FORMA (arquitectura) — provoca reescritura.
2. Scaffold antes del scan de REUSO — provoca revisar manifests dos veces.
3. Dar código de solución (esto diseña; el código lo hace `odoo-module-creator`).
4. Sintaxis versión-específica de memoria — verifica a fuente Odoo (ver `references/shared/odoo-source.md`).
5. Elevar detalles de implementación a preguntas — se deciden construyendo.
6. Dejar decisiones derogadas vivas en tablas antiguas — una sola fuente de verdad.

## ALWAYS
1. FORMA → REUSO → DETALLE, en ese orden.
2. **El bucle iterativo de Q&A es el núcleo de valor**: la skill genera las preguntas, una a una, ronda
   tras ronda con análisis de gaps entre rondas, hasta que ningún gap bloquee una fase. No lo saltes ni lo
   abrevies por prisa; sí páralo cuando solo queden detalles que un dev resuelve construyendo.
3. Cada decisión trazable (`D##`) y las versión-específicas verificadas a fuente.
4. Health-check honesto (🟢/🟠/🔴); nombra lo que solo se cierra construyendo.
5. Estimación en cifra única (esfuerzo real con factor IA) vía `odoo-effort-estimator`, recalibrable por fase.
6. Scaffold validado sintácticamente antes de entregar.
7. Delegar la redacción de tareas a `dev-task-writer`, no redactarlas aquí.

## GOTCHAS
1. **`module_<x>` en `res.config.settings`** instala/desinstala el módulo `<x>` (patrón nativo; `stock` usa `module_stock_picking_batch`). Es EL mecanismo de "activar con check".
2. **Desinstalar un check borra los datos** de ese módulo — avisa; no lo pintes como reversible sin coste.
3. **La estimación churnea** con cada ronda: eso es descubrimiento honesto, no error — deja trazabilidad del porqué.
4. **Naming/concepto colisiona con `odoo-solution-architect`** — mantén breadth vs depth explícito o el router carga la skill equivocada.
5. **Costear con landed cost, valued picking, etc. son deps con cadena** — el scan de reuso (Step 2) las descubre ANTES de scaffoldear.
6. **Cross-module seams**: un estado/campo del base que un módulo de capacidad refina → decide dónde vive el campo base y quién lo sincroniza (evita la costura frágil).

## Delegates to
| Skill | Suite | Para qué |
|---|---|---|
| `dev-assessment` | common | Gate ¿custom? antes de diseñar |
| `odoo-classification-guide` | common | Reglas Standard/Studio/Core/Custom |
| `odoo-module-scanner` | common | Scan de reuso (módulo/parque) |
| `odoo-effort-estimator` | common | Estimación por módulo/fase + factor IA |
| `odoo-solution-architect` | developer | Contrato profundo de un módulo de riesgo |
| `dev-task-writer` | common | Redactar cada tarea del set |
| `odoo-module-creator` | developer | (downstream) generar el código real |

## Consumed by
| Skill | Suite | Para qué |
|---|---|---|
| `tl-assistant` | common | Cuando el TL arranca el diseño de un módulo nuevo antes de asignar |
| `project-plan` | consultant | Cuando un GAP requiere diseñar un módulo custom multi-pieza |

## REFERENCES
- `references/design-loop.md` — orden FORMA→REUSO→DETALLE, Q&A por capas, IDs `D##`, health-check, fuente única.
- `references/scaffold-and-phasing.md` — fasing obra + git (rama/commits), scaffold multi-módulo y su validación.
- `references/shared/odoo-source.md` — resolver la fuente Odoo para verificar (no asumir `/opt/odoo`).
- `references/shared/odoo-connection.md` — patrón de conexión (no reimplementar).
