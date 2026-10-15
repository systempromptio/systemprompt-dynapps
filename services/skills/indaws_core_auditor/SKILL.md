---
name: indaws-core-auditor
description: >
  Audita un modulo Odoo contra la filosofia Indaws Core (multi-cliente, anti-hardcoding, nomenclatura
  indaws_*, documentacion) y reporta fallos accionables. Usar antes de PR de modulos indaws_*.
  Triggers: "auditar modulo core", "cumple la filosofia core", "core compliance",
  "revisar modulo antes del PR", "es valido como Core", "tiene hardcoding de cliente".
version: 1.4.2
effort: medium
model: sonnet
---

# Indaws Core Auditor

Audita un modulo Odoo contra la **filosofia Indaws Core** y reporta fallos
accionables, para que un PR de modulo Core salga con el minimo de incidencias.
No reimplementa el code-auditor ni el escaner: los **orquesta** y aporta el
conocimiento de filosofia Core que ninguno tiene.

> Es el cliente quien se adapta al sistema, no el sistema al cliente. No se
> reinventa lo que el estandar o Core ya hacen; y el codigo que se escribe debe
> ser gobernado (revisado, testeable) y servir a muchos clientes: cerca del
> estandar, empaquetado y documentado. Criterio Studio-vs-custom en `odoo-classification-guide`.

Las 4 particularidades que se auditan (el "por que" en
[references/core-philosophy.md](references/core-philosophy.md), **cargar al iniciar**):

1. **Multi-cliente** — reutilizable, sin hardcoding de cliente; lo variable se parametriza.
2. **No reinventar lo existente** — no reimplementar estandar/Studio/Core ya disponible (Golden Rules + Core Supremacy; criterio en `odoo-classification-guide`).
3. **Empaquetado** — `indaws_*`, en un paquete por area, manifest coherente.
4. **Documentacion** — README/manifest util dentro del modulo.

## Agent Routing

Runs in sonnet. Orquestador: delega analisis a subagentes sonnet (module-scanner, code-auditor); aporta la filosofia Core de `references/` — **sin conexion a Odoo**. **Antes de reportar, consultar [references/gotchas.md](references/gotchas.md)** (falsos positivos: `self.env.company`/`ir.config_parameter` no son hardcoding; version del manifest sin asumir migracion).

See also `references/shared/coding-standards.md` for nomenclatura y convenciones de codigo.

## References

| File | Contents | Load when |
|------|----------|-----------|
| [references/gotchas.md](references/gotchas.md) | Falsos positivos a evitar | **Siempre**, antes de reportar |
| [references/core-philosophy.md](references/core-philosophy.md) | Filosofia Core completa y el "por que" (fuente: Conocimiento art. 1933) | **Siempre**, al iniciar |
| [references/core-antipatterns.md](references/core-antipatterns.md) | Catalogo de anti-patrones con patrones de busqueda y ejemplos bien/mal | Al auditar multi-cliente y anti-codigo |
| [references/compliance-checklist.md](references/compliance-checklist.md) | Checklist completo + scoring (Core Compliance Score) + severidades | Al evaluar y puntuar |
| [references/delegation.md](references/delegation.md) | Como invocar module-scanner / code-auditor / classification-guide y fusionar resultados | Al delegar |
| [references/report-generation.md](references/report-generation.md) | Estructura del informe, niveles, longitudes | Al generar el informe |
| [assets/report_template.md](assets/report_template.md) | Plantilla del informe de auditoria Core | Al generar el informe |

## Quick Start Workflow

### 1. Reunir informacion
- **Version de Odoo**: NO la preguntes ni la asumas — **deriva la serie del
  `version` del `__manifest__.py`** del modulo. Esa es la version vigente.
- **Target**: por defecto **NO hay migracion**; auditas el modulo en su version
  vigente. Solo trata otra version como destino si el usuario lo pide
  explicitamente ("migrar a 19", "para el catalogo Core 19"). Asumir un target
  de migracion inventa deuda inexistente (ver `references/gotchas.md`).
- **Contexto**: pre-PR / candidato a Core / auditoria de Core existente.
- **Ruta(s)**, **Nivel** (Alto/Medio/Bajo) y **Longitud** (Extenso/Estandar/
  Resumen): pregunta solo lo que falte; reutiliza lo que ya este en
  `desarrollos/PROJECT_REGISTRY.md` o lo que pase el skill invocante
  (dev-assessment, quality-gate, git-pr) sin re-preguntar.

### 2. Cargar la filosofia
Lee `references/core-philosophy.md` y `references/gotchas.md`: son la vara de
medir; sin ellas no audites.

### 3. Inventario del modulo  [SONNET SUBAGENT]
Agent(model: sonnet, description: "scan {nombre_modulo} inventory") → invoca
**odoo-module-scanner** sobre {ruta}: modelos, campos, vistas, security,
controladores, JS, manifest, datos XML. Ver `references/delegation.md`.

### 3.5 Contexto de dependencias  [SOLO LECTURA — NO auditar]
Si el modulo hereda (`_inherit`) o llama `super()`/overridea metodos de otros
modulos Core de sus `depends`, escanea **solo la superficie publica** de esos
padres (firmas de los metodos/campos que el modulo extiende) para validar el
bloque B con base real, no con docstrings. **Esos modulos NO se auditan, NO se
corrigen, NO entran en el score**: son contexto. Ver `references/delegation.md`.

### 4. Validar la clasificacion Core
Consulta **odoo-classification-guide** (Golden Rules + Core Supremacy) para
confirmar que debe ser Core y detectar partes que serian Standard, Studio u
otro modulo Core. Materializa el principio anti-codigo.

### 5. Auditar la filosofia Core (analisis propio)
Ejecuta `python3 scripts/scan_core_antipatterns.py <ruta> [--json]` (escaner
determinista; no reinventes greps), interpreta cada hit con
`references/core-antipatterns.md` filtrando con `references/gotchas.md`, y
evalua naming/empaquetado/manifest/documentacion con
`references/compliance-checklist.md`.

### 6. Delegar calidad/seguridad  [SONNET SUBAGENT]
Agent(model: sonnet, description: "audit {nombre_modulo} code v{version}") →
invoca **odoo-code-auditor** sobre {ruta}. **Pasa la version vigente {version}
del manifest e indica si NO hay migracion**, para que no marque como deuda APIs
correctas en esa version. Focus: Standard Features, Conventions, Security,
Performance, Code Quality; **Migrability SOLO si se pidio migracion** (con
target). No reimplementes. Ver `references/delegation.md`.

### 7. Consolidar e informar
Calcula el **Core Compliance Score**, fusiona tus hallazgos con los delegados y
genera el informe con `references/report-generation.md` +
`assets/report_template.md`. Cada hallazgo **accionable**: que esta mal, por
que rompe la filosofia Core, como arreglarlo.

## Delegates to

| Skill | Suite | When |
|-------|-------|------|
| odoo-module-scanner | common | Inventario de componentes del modulo |
| odoo-classification-guide | common | Golden Rules / Core Supremacy / clasificacion |
| odoo-code-auditor | developer | Calidad, seguridad, performance, migrabilidad, standard features |

## Consumed by

| Skill | Suite | For what |
|-------|-------|----------|
| dev-assessment | common | Validar que una idea propuesta como Core cumple la filosofia antes de desarrollar |
| quality-gate | common | Gate de filosofia Core pre-merge de modulos `indaws_*` |
| git-pr / git-manager | developer | Chequeo pre-PR cuando el modulo es `indaws_*` o Core |
| odoo-code-auditor | developer | Capa de filosofia Core sobre la auditoria de codigo |
| odoo-module-upgrader | developer | Aplica los arreglos detectados (tras la auditoria) |

## Prohibitions
- NO reimplementar module-scanner, code-auditor ni classification-guide: delega.
- NO reportar hallazgos sin pasar por `references/gotchas.md`.
- NO exigir conexion a Odoo (filosofia empaquetada). La documentacion se audita
  solo en el modulo (README/manifest); la ficha en Conocimiento NO se audita.
- NO inventar reglas de filosofia: la unica fuente es `references/`.
- NO auditar, corregir ni puntuar los modulos de los que depende (contexto solo-lectura, paso 3.5).

## Gotchas
Lista completa en `references/gotchas.md` (consultar **siempre** antes de
reportar). Mas caros: `self.env.company`/`ir.config_parameter`/`res.company` no
son hardcoding (son parametrizacion correcta); demo data en `demo/` es
aceptable; ausencia de `x_`/Studio es correcta en Core; version derivada del
manifest sin asumir migracion; dependencias = contexto, no se auditan.
Ortografia es_ES obligatoria en el informe (tildes y enes).
