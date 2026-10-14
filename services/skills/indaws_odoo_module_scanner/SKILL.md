---
name: indaws-odoo-module-scanner
version: 1.2.1
effort: low
model: sonnet
description: Analiza la estructura de un modulo Odoo y genera un inventario de componentes (modelos, campos, vistas, seguridad, controladores, JS). Usa este skill cuando otro skill necesite entender que contiene un modulo antes de actuar sobre el. Triggers directos - "analiza este modulo", "que tiene este modulo", "escanea el modulo". Triggers indirectos - invocado por odoo-test-generator, odoo-code-auditor, odoo-database-analyzer, odoo-module-documenter cuando necesitan un inventario de componentes.
---

# Odoo Module Scanner

Skill de referencia que analiza un modulo Odoo y produce un inventario estructurado de sus componentes. NO genera codigo ni ejecuta acciones — produce informacion que otros skills consumen.

## Cuando se usa

- **Directamente**: El usuario quiere entender que contiene un modulo
- **Indirectamente**: Otro skill necesita un inventario antes de actuar:
  - `odoo-test-generator` → necesita saber que componentes testear
  - `odoo-code-auditor` → necesita saber que analizar
  - `odoo-database-analyzer` → necesita clasificar modulos
  - `odoo-module-documenter` → necesita saber que documentar
  - `odoo-module-upgrader` → necesita saber que patrones buscar
  - `odoo-copy-staging-master` → necesita inventario de modulos custom para clasificar y detectar riesgos

## Agent Routing
Motor determinista (script, ~0 tokens) + enrutado; solo escala a LLM lo que el script no resuelve. Corre en sonnet.

See `references/shared/field-mapping-syntax.md` for field types and command tuples.

## Workflow

### Paso 1: Recibir path del modulo
Path del modulo (debe contener `__manifest__.py`). Pedirlo si falta.

### Paso 2: Motor determinista (SIEMPRE primero)
`python3 scripts/scan_module.py <path>` → inventario JSON (contrato de `references/output-format.md`)
+ bloque `confidence` con `route`. Barato: NO rastrees el arbol a mano antes de esto.
`--md` para salida legible al usuario; JSON para skills consumidores.
**Fallback:** si el script falla o el entorno no lo ejecuta, escanea a mano con `references/detection-patterns.md`.

### Paso 3: Enrutar segun `confidence.route` (SESGO A ESCALAR — ante duda, escalar)
- **`script`** (high) → usar la salida tal cual. Fin. Cubre el ~90% (modulos estandar).
- **`script+db`** (bandera `studio_signals`) → los `x_studio_*` viven en `ir.model.fields`, NO en
  codigo: completar esa parte via odoo-pilot/MCP (`references/shared/odoo-connection.md`) y
  fusionar en el inventario.
- **`llm`** (banderas de parser: manifest/xml ilegible, definiciones dinamicas) → escanear a mano con
  `references/detection-patterns.md`, **SEMBRADO con el JSON parcial del script** (no partir de cero).

### Paso 4: Presentar inventario
El JSON del script YA cumple `references/output-format.md`. Devolver JSON al skill consumidor;
`--md` para el usuario. La clasificacion (SIMPLE/STANDARD/COMPLEX/ADVANCED) la calcula el motor.

## Que NO hace este skill

- No genera codigo
- No modifica ficheros
- No ejecuta tests
- No audita calidad (eso es `odoo-code-auditor`)
- No estima esfuerzo (eso es `odoo-effort-estimator` / `odoo-database-analyzer`)

## Motor y referencias
- `scripts/scan_module.py` — motor determinista: implementa las Fases 2-9, emite el contrato en JSON + `confidence`/`route`.
- `references/detection-patterns.md` — grep patterns y reglas (fallback LLM en ruta `llm`).
- `references/output-format.md` — contrato de salida (el JSON del motor lo cumple).

## GOTCHAS
1. **El motor es el paso 1, no opcional** — no rastrees el arbol a mano; corre `scan_module.py` y enruta por `confidence`.
2. **Studio (x_studio_*) vive en `ir.model.fields`, no en codigo** — un script estatico no lo ve: el motor marca `studio_signals` → ruta `script+db` (completar via odoo-pilot/MCP). NO reportar campos Studio desde solo codigo.
3. **No inflar `confidence`** — falso "high" = inventario incompleto en silencio. Ante duda, el motor baja a `llm`/`script+db`; respeta esa ruta.
4. **Ruta `llm` = sembrada, no desde cero** — parte del JSON parcial del script y solo rellena lo que falta.
5. **Computed sin store=True no estan en BD** — el motor los detecta en codigo (`compute=`); la presencia en BD solo se confirma en `script+db`.

## Delegates to
None — this is a leaf skill that produces inventory data.

## Consumed by
| Skill | Suite | For what |
|-------|-------|----------|
| odoo-database-analyzer | common | Component inventory for migration analysis |
| odoo-copy-staging-master | common | Module inventory for config migration |
| odoo-module-creator | developer | Pre-creation duplicate check |
| odoo-module-upgrader | developer | Component inventory before upgrade |
| odoo-code-auditor | developer | Component inventory for audit |
| odoo-test-generator | developer | Component inventory for test generation |
| odoo-rebrander | developer | Component inventory for rebranding |
| odoo-module-documenter | consultant | Component inventory for documentation |
| indaws-core-documenter | consultant | Component inventory for Core docs |

## RULES
### NEVER
- Report a field count without distinguishing inherited vs own fields
- Assume all models are in models/ — check also wizards/, reports/, controllers/
### ALWAYS
- Include security files (ir.model.access.csv, ir_rules.xml) in inventory
- Report file counts AND line counts per component type
