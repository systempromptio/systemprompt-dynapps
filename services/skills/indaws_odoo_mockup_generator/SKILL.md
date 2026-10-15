---
name: indaws-odoo-mockup-generator
version: 0.6.2
effort: medium
model: opus
description: >-
  Genera mockups HTML autocontenidos de pantallas de backend Odoo (v17-v19) fieles a la UI real
  (form, list, kanban, search, dashboard) ANTES de construir, para alinear con cliente y dar spec al
  dev. Cada elemento se ancla a un constructo real (statusbar↔state, o2m↔One2many, chatter↔mail.thread)
  y se clasifica ESTÁNDAR/STUDIO/DESARROLLO, y emite el esqueleto XML+modelo para odoo-module-creator.
  Usar cuando pidan "mockup de Odoo", "maqueta de la pantalla", "boceto de un kanban" o "cómo se vería
  esta vista". NO captura un Odoo real (odoo-screenshot-generator) ni produce las vistas finales.
allowed-tools: [Read, Write, Bash, Glob, Grep, AskUserQuestion]
---

# Odoo Mockup Generator

Produce **mockups estáticos** (HTML autocontenido, no funcional) de pantallas de backend Odoo que
parecen Odoo de verdad **y son honestos**: cada cosa dibujada se mapea a un constructo real del ORM/vistas
y se etiqueta como **ESTÁNDAR** (nativo), **STUDIO** (configurable sin código) o **DESARROLLO** (custom),
señalando los **límites** de lo que Odoo hace nativo. El valor no es el HTML bonito: es que el mockup
diga qué es gratis, qué es Studio y qué hay que programar — antes de tocar código.

**El diseño es tuyo — sé creativo.** El input puede ser solo una **descripción en texto**, un **boceto en
Paint / foto de una pizarra**, un **HTML de referencia**, o nada más que una idea. Si te dan un mockup previo,
es **intención, no plantilla**: extrae qué se quiere resolver y diseña la **mejor pantalla nativa** para ello
(a veces la buena respuesta es una vista estándar distinta a la dibujada). El único límite duro es la fuente:
todo lo que pintes debe apoyarse en un widget/constructo Odoo real, o ir marcado como DESARROLLO.

## Agent Routing

| Fase | Modelo | Por qué |
|------|--------|---------|
| Recoger brief + elegir tipo de vista | opus (principal) | Juicio sobre qué vista y qué constructos |
| Anclar sintaxis/widgets de la versión | sonnet (subagente) | Delegación mecánica a la referencia de API |
| Generar HTML + capa de anotaciones | opus (principal) | Composición y clasificación estándar/studio/dev |

## Workflow

### STEP 0 — Brief [STOP]
Acepta cualquier input: texto, **boceto/imagen** (léela con `Read`: Paint, foto de pizarra, captura),
un HTML de referencia, o una **spec** del studio (`assets/mockup-studio.html`). Extrae la **intención**, no copies el diseño. `AskUserQuestion` lo que falte:
**pantalla** y **rol**, **modelo(s)** base y si es estándar o custom, **versión** Odoo objetivo, y
contexto de proyecto (pre-rellena de `PROJECT_REGISTRY.md` si existe). El **tipo de vista lo decides tú**
según la intención (no lo impongas desde el boceto). Un mockup sin modelo/versión es media maqueta. **Espera.**

### STEP 1 — Anclar a la fuente [SUBAGENTE sonnet — OBLIGATORIO]
Nunca emitas widgets ni sintaxis de vista de memoria. Tres anclajes:
- **(a) Widgets reales**: carga `references/widget-catalog.md` (lista + cómo se dibuja cada uno). Si dudas
  de la versión, refresca con `scripts/list_widgets.sh <odoo_root> [--all]` (extrae el registro real `category('fields')`).
- **(b) Sintaxis/deltas de versión**: `Agent(model: sonnet, subagent_type: general-purpose)` → invoca
  `odoo-version-api-reference` (tags raíz, `<tree>`→`<list>` v18, kanban `<t t-name="card">` v19, `<chatter/>` v18+).
- **(c) Campos**: verifica cada campo estándar que dibujes contra el MODELO real (delega en `odoo-module-scanner`
  o mira la fuente); marca cada campo **existente** vs **campo nuevo**.
Carga `references/view-capabilities-and-limits.md`. Guarda como `GROUND`.

### STEP 2 — Diseñar: la spec de 2 capas [opus]
Modela en la **spec de `references/spec-schema.md`**: capa **modelo** (campos con su **tipo ORM** + widget +
`origin` std/new + `compute`) definida UNA vez, y capa **vistas** que referencian campos por nombre
(coherencia form/list/kanban/search). **Valida widget↔tipo** con `references/widget-matrix.json` (widget ∈
`supportedTypes[type]`). **Clasifica por reglas** (no a mano): std=existe · nuevo+tipo estándar+widget nativo=**STUDIO** ·
computed/OWL/lógica=**DEV** · gantt/map/cohort=**EE**. Reutiliza modelos estándar antes de inventar (repair, sale…).
**Si el cliente pide algo NO nativo**, aplica `references/non-native-to-native.md`: sube la escalera (config→Studio→app→vista alternativa→dev) y sustituye por el equivalente nativo, anotándolo; solo lo que no sube por la escalera va como bloque **DEV/OWL**.

### STEP 3 — Generar el mockup [opus]
Construye desde `references/odoo-ui-anatomy.md` (HTML **autocontenido**, tokens reales, sin CDNs).
**Dibuja cada campo como su widget real** con los snippets de `references/widget-catalog.md` (priority =
estrellas, state_selection = punto de color, many2many_tags = pills, monetary alineado a la derecha,
statinfo en los smart buttons…), no un valor genérico. **Explota la arch nativa** (`view-arch-capabilities.md`): en list, color de fila (`decoration-*`), totales (`sum=`) y columnas opcionales; no lo marques dev — es declarativo. Pon en cabecera un **resumen de esfuerzo**
(≈ nativo / Studio / dev / EE). Añade la capa de anotaciones (`dev-annotation-layer.md`): círculos
numerados → panel, con modelo/campo/widget real + tag ESTÁNDAR/STUDIO/EE/DEV + existente/nuevo + MVP/Fase 2.
Disclaimer "Mockup de referencia — no funcional".

### STEP 4 — Entregar + emitir esqueleto
El `.html` + resumen de esfuerzo (estándar/Studio/dev/EE). Si piden construirlo, **emite el esqueleto
Odoo desde la spec** según `references/spec-to-odoo.md`: `models/*.py` (solo campos nuevos, tipos reales,
stubs `_compute_*`) + `views/*.xml` (sintaxis de la versión) → **entrégalo a `odoo-module-creator`**.
Prototipo del emisor + builder en `assets/`. Sugiere `odoo-screenshot-generator` para capturarlo una vez exista.

## GOTCHAS

1. **Es un MOCKUP, no Odoo.** Nunca lo presentes como funcional; disclaimer siempre.
2. **No dibujes lo que Odoo no hace nativo sin marcarlo DEV.** La honestidad estándar/studio/dev es el producto.
3. **statusbar = campo `state`/selection · smart button = `count` de un related · notebook = `<page>` · o2m = `One2many` editable-list · chatter = `mail.thread`.** No inventes componentes que no existen en backend.
4. **Sintaxis de vista versión-específica: NUNCA de memoria** → `odoo-version-api-reference` (STEP 1). `<tree>` no existe en v18; kanban v19 exige `<t t-name="card">`.
5. **Límites de la o2m/list**: no toda tabla anidada con subtotales "tipo secciones de pedido" es nativa — si la pides, va anotada como DESARROLLO (caso real Autosur).
6. **Sin CDNs** (offline/CSP): CSS y JS inline; nada de fuentes/scripts externos.

## NEVER
- Conectar a un Odoo real ni pretender datos vivos (eso es `odoo-screenshot-generator`).
- Emitir sintaxis de vista versión-específica de memoria.
- Pintar una capacidad custom como si fuera estándar sin anotación DESARROLLO.
- Dependencias externas (CDNs, fuentes, scripts remotos).

## ALWAYS
- Anclar en `odoo-version-api-reference` antes de dibujar (STEP 1).
- Clasificar cada bloque ESTÁNDAR / STUDIO / DESARROLLO y listar límites.
- Capa de anotaciones para dev + disclaimer "no funcional".
- Tokens y anatomía reales de Odoo; HTML autocontenido.
- **Provenance marker:** el `.html` acaba con `<!-- Made with odoo-mockup-generator · v<version> -->`, idempotente. Regla: `references/shared/skill-provenance-marker.md`.
- **Autochequeo pre-entrega:** cada elemento dibujado lleva su widget real y su tag (STD/STUDIO/EE/DEV); ningún elemento custom sin marcar; sin CDNs; disclaimer presente; resumen de esfuerzo en cabecera.
- **Interactivo, nunca "de foto":** entrégalo con la interacción básica funcionando (JS inline) — pestañas del notebook que cambian, o2m que despliega, toggle de chatter derecha/abajo, kanban navegable. Es parte de la fidelidad (esqueleto JS mínimo en `odoo-ui-anatomy.md`).

## Delegates to

| Skill | Suite | Cuándo |
|-------|-------|--------|
| odoo-version-api-reference | common | OBLIGATORIO — tags de vista, widgets y deltas de versión (STEP 1) |

**Opcional (si la suite developer está instalada):** `odoo-view-designer` para arquitectura de vista
profunda (searchpanel, kanban v19, herencia) y `odoo-orm-modeler` para viabilidad de modelo/campo.
Common no depende en firme de una hija: degradar a `view-capabilities-and-limits.md` si no están.

## Consumed by

| Skill | Suite | Para qué |
|-------|-------|----------|
| odoo-module-creator | developer | Toma la spec/esqueleto (models+views) como diseño de partida del módulo |

También (informativo): análisis/kickoff/validación (consultant) y pre-venta (sales) usan el mockup para alinear con cliente.

## References

| Fichero | Contenido | Cuándo |
|---------|-----------|--------|
| [references/widget-catalog.md](references/widget-catalog.md) | **Dato de fuente** — lista real de widgets + cómo se DIBUJA cada uno (snippets). SSOT de "widget real" | STEP 1 — anclar · STEP 3 — dibujar |
| [scripts/list_widgets.sh](scripts/list_widgets.sh) | **Motor** — extrae los widgets reales registrados para un `<odoo_root>`/versión | STEP 1 — refrescar catálogo |
| [references/odoo-ui-anatomy.md](references/odoo-ui-anatomy.md) | Esqueleto HTML/CSS con tokens reales por tipo de vista (form/list/kanban/search/dashboard) y componentes | STEP 3 — generar |
| [references/view-capabilities-and-limits.md](references/view-capabilities-and-limits.md) | Qué es nativo/Studio/dev · límites duros · punteros a deltas de versión (los widgets viven en widget-catalog) | STEP 1-2 — diseñar |
| [references/dev-annotation-layer.md](references/dev-annotation-layer.md) | Patrón de notas numeradas + cómo redactarlas (modelo/campo/widget, tag, MVP/Fase 2) | STEP 3 — anotar |
| [references/spec-schema.md](references/spec-schema.md) | **Contrato** — spec de 2 capas (modelo+vistas), reglas del motor y clasificador | STEP 2 — modelar |
| [references/widget-matrix.json](references/widget-matrix.json) | **Dato de fuente** — tipo↔widget (`supportedTypes`). Regenera con `scripts/extract_widget_matrix.py` | STEP 2 — validar widget |
| [references/spec-to-odoo.md](references/spec-to-odoo.md) | **Emisor** — spec → `models/*.py` + `views/*.xml` (sintaxis por versión) para odoo-module-creator | STEP 4 — emitir |
| [references/non-native-to-native.md](references/non-native-to-native.md) | **Estrategia** — escalera de sustitución + catálogo no-nativo→nativo (CE/EE) + cómo anotarlo + cuándo NO forzar | STEP 2 — si piden algo no nativo |
| [references/view-arch-capabilities.md](references/view-arch-capabilities.md) | **Arch/QWeb más allá de campos** — list (decoration/sum/optional/botón), kanban QWeb, form (ribbon/alert/view-widgets), searchpanel, report | STEP 2-3 — explotar la arch nativa |
| assets/mockup-studio.html | **Herramienta principal (field-first)** — define el modelo UNA vez (tipo+widget validado+origen) → form/list/kanban coherentes + spec 2 capas + emisor de código. Se abre en navegador | Entregar/usar con el consultor |
| assets/mockup-builder.html · assets/mockup-engine.html | Prototipos previos (builder drag-drop multi-vista · motor de render+emisor). Superados por el studio | Referencia |
