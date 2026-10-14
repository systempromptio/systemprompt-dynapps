---
name: indaws-odoo-deep-knowledge
version: 2.8.47
effort: low
model: sonnet
description: >-
  Comportamientos ocultos, gotchas e interacciones cross-módulo de Odoo 18 CE+EE
  (+v19 diffs): ~88 dominios, 5000+ gotchas, índice de síntomas, mapas de
  integración y 14 playbooks end-to-end. Invocar al depurar comportamiento
  inesperado, diseñar módulos, planificar implementaciones o cuando un flujo se
  rompe a mitad. Triggers: "por qué no se reconcilia", "stock no reserva", "cron
  no ejecuta", "valoración incorrecta", "gotcha", "comportamiento oculto", "por
  qué Odoo hace X", "el flujo se rompe a mitad".
---

# Odoo Deep Knowledge v2.3

Comportamientos ocultos, interacciones entre módulos y gotchas de Odoo 18.0 + v19 diffs.

## Cómo usar — 3 pasos

> **Atajo de tokens:** `scripts/kb.py "<síntoma/keywords>"` devuelve solo los bloques-gotcha relevantes (~15× menos tokens que leer ficheros enteros). `--titles` = TOC barato, `--id ACP4` = expandir uno, `--model sale.order` = por modelo. Úsalo ANTES de abrir ficheros; abre el fichero completo solo si necesitas el contexto alrededor.
>
> **⚠️ Búsqueda por NIVELES (recall).** `kb.py` es keyword-grep (no semántico); escala según su línea **`CONFIANZA:`** (no a ojo): **alta** → fíate del top; **media/baja/nula** → identifica el dominio (`--titles` / `_index/symptoms.md`) y **lee su `gotchas.md` completo**; solo si eso no resuelve, IA (semántico / leer el dominio) como **ÚLTIMO recurso, nunca el primer salto**. `CONFIANZA` baja o "sin resultados" = **ESCALAR**, nunca rendirse.

### Paso 1: Buscar por síntoma o modelo (SIEMPRE primero)

Los tres índices de `_index/` son TABLAS: quieres una fila, no el fichero. **Se preguntan, no se leen** — leerlos cuesta entre 100 y 400 veces la respuesta. Si no sabes por dónde, `kb.py --cheat` lista las 78 secciones (~1.270 tok). Abrir un `.md` entero solo vale para editarlo, o si la consulta sale vacía.

| Si tienes... | Pregunta | Coste vs leer el índice |
|---|---|---|
| Un síntoma concreto ("no se reconcilia", "stock no reserva") | `kb.py --symptom "<síntoma>"` | ~800 tok vs 79.732 |
| Un modelo concreto (`stock.move`, `hr.expense`) | `kb.py --map stock.move` | ~210 tok vs 8.463 |
| Una pregunta general sobre un módulo | `kb.py --cheat stock` | ~90 tok vs 34.894 |
| Un flujo que se rompe a mitad | `kb.py --symptom …` → `playbooks/<flujo>.md` | |

### Paso 2: Leer el fichero del dominio

Cada dominio tiene ficheros con patrón consistente:

| Fichero | Contenido | Cuándo leer |
|---------|-----------|-------------|
| `core.md` | Modelo principal, ciclo de vida, campos clave | Siempre primero |
| `gotchas.md` | Errores comunes, comportamientos ocultos | Debugging, antes de escribir código |
| `v19_changes.md` | Cambios en Odoo 19 | Si el target es v19 |
| `integrations.md` | Flujos cross-module | Diseño de soluciones |
| `config_guide.md` | Recetas de configuración | Implementación |

### Paso 3: Solo si necesitas más profundidad

Algunos dominios tienen ficheros especializados (`rules_deep.md`, `deep.md`, `operations.md`, etc.). Ver `_index/module-map.md` para listado completo por dominio.

## Estructura de dominios

```
references/
├── _index/        ← symptoms.md, module-map.md, cheatsheet.md
├── functional/    ← 29 dominios CE (stock, account, sale, purchase, hr, project, crm, pos, mail,
│                    mailing, website, delivery, repair, maintenance, partnership, fleet, lunch, survey,
│                    edi, spreadsheet, calendar, im_livechat, event, expense, slides, recruitment, forum, blog, membership)
├── enterprise/    ← 45 dominios EE (approvals, fsm, mrp_workorder, helpdesk, planning, timesheet,
│                    hr_payroll, hr_appraisal, mrp_plm, subscription, studio, sign, rental, knowledge, marketing_automation, social,
│                    whatsapp, stock_barcode, intercompany, quality, intrastat, sale_amazon,
│                    spreadsheet, iot, voip, pos_blackbox, appointment, documents, enterprise_tools,
│                    + 6 account_* y website_sale_subscription
│                    + banca/compliance: bank_statement_import, account_iso20022, account_saft,
│                      account_extract, account_loans
│                    + admin tools: data_tools (cleaning/merge/recycle), frontdesk)
├── cross-module/  ← analytic, discuss_bus, mrp, product, security, cron, reporting
├── playbooks/     ← 14 flujos end-to-end cross-módulo (O2C, P2P, MTO, FSM, eCommerce,
│                    subscription, intercompany, project profitability, RMA, POS reconciliation)
├── ecosystem/     ← indaws (154 módulos v18; 158 en v19), inbuilt (construcción), l10n_es, oca-accounting, queue_job (OCA async jobs), mis_builder (OCA reporting financiero)
└── engine/        ← DELEGADO → odoo-version-api-reference
```

## Playbooks — flujos completos cross-módulo

Cuando el problema NO es un módulo aislado sino un proceso de negocio que cruza 4-7 módulos
y "se rompe a mitad", lee el playbook correspondiente en `playbooks/`. Cada uno documenta la
cadena etapa-por-etapa (trigger → modelo.método → qué produce → config → gotchas que aplican →
dónde se rompe) y cross-referencia los gotcha IDs existentes. Índice en `playbooks/README.md`
y entradas de síntoma en `_index/symptoms.md` §26.

## Para framework/engine (ORM, HTTP, OWL, QWeb)

Consultar **`odoo-version-api-reference`** skill directamente (ORM, HTTP, JS/OWL, Performance, Secure coding). Ver `references/engine/README.md` para mapa completo.

## Agent Routing

Este skill es **content-only** (knowledge base estática). NO ejecuta agentes ni dispara workflows. Es invocado por otras skills mediante delegación horizontal y consultado directamente por Claude cuando aplica.

| Phase | Model | Runs as |
|-------|-------|---------|
| Lectura indexada | sonnet | Subagent (vía consumer skill) |

Los consumidores de este skill (workflows de migración, code auditor, module upgrader, etc.) son quienes decidirán el modelo según el tamaño/criticidad de la consulta.

## GOTCHAS

1. **Las referencias deeper-than-1 se leen parcialmente.** Mantén accesos directos desde `_index/symptoms.md` y `module-map.md` — no encadenes referencias por más de un nivel.
2. **Las versiones del knowledge base están atadas a la versión Odoo cubierta.** Tag `v18.X` cubre Odoo 18 + v19 diffs. Si trabajas con v17 o anterior, este skill puede no ser completo.
3. **Los gotchas verticales (inbuilt, indaws, l10n_es) asumen instalación Indaws Core.** Si la instancia objetivo no tiene `indaws_*` módulos instalados, varias gotchas no aplican.
4. **OCA v19 portability is moving target.** El estado documentado en `ecosystem/oca-accounting/v19_status.md` puede quedarse desfasado cada release OCA. Verificar antes de usar como source-of-truth.
5. **`engine/` está DELEGADO al skill `odoo-version-api-reference`.** No buscar ahí internals de ORM/HTTP/OWL — consulta el otro skill directamente.

## NEVER
1. Modificar gotcha IDs (`STG1`, `APPG1`, etc.) — son referenciados cruzados desde `_index/symptoms.md`.
2. Borrar entradas sin actualizar `_index/symptoms.md` y `module-map.md`.
3. Documentar comportamientos NO verificados en código fuente real (mentir = veneno para Claude).

## ALWAYS
1. Anclar a `fichero.py → método/campo` (estable entre patches); la línea es pista aproximada, NO te obsesiones con su drift. Validar símbolos con `scripts/cite_audit.py --methods` (¿el `_método` existe en fuente?) y citas con `cite_audit.py` (¿fichero/línea resuelven?).
2. Asignar nuevo gotcha ID al añadir entry: **prefijo en MAYÚSCULAS + número secuencial por dominio** (p.ej. `MCG41`, `STKG73`, `A3M16`). Nunca repetir un ID dentro del mismo fichero. Validar con `scripts/coverage.py --lint` (detecta colisiones intra-fichero); el contador es agnóstico al formato del prefijo.
3. Mantener patrón `core.md + gotchas.md + integrations.md + config_guide.md + v19_changes.md` al añadir dominio.
4. Actualizar `_index/symptoms.md` y `_index/cheatsheet.md` tras cada deep-scan.
5. Antes de un release del KB: correr **`scripts/kb_release_check.py`** — UN comando que junta los gates deterministas (cite_audit + coverage + quality + recall_eval, con umbrales → veredicto) y emite la muestra de `sample_gotchas` para el paso semántico (verificar contra fuente con agentes → CONFIRMED/PARTIAL/REFUTED, registrar %, arreglar REFUTED). **NO correr los 5 sueltos.**

## Delegates to

| Skill | Suite | For what |
|-------|-------|----------|
| odoo-version-api-reference | common | Internals de engine (ORM, fields, HTTP, OWL, QWeb). `engine/` apunta aquí. |

## Consumed by

| Skill | Suite | For what |
|-------|-------|----------|
| odoo-module-upgrader | developer | v19 breaking changes antes de migrar código |
| odoo-code-auditor | developer | Identificar gotchas en código auditado |
| odoo-database-upgrader | developer | Comportamiento cross-version, validación post-migración |
| migration-assistant | common | Source-of-truth de cambios v18→v19 |
| dev-task-writer | common | Restricciones técnicas y gotchas para tareas dev |
| odoo-database-analyzer | common | Cross-reference comportamientos al analizar legacy code |
| tl-assistant | common | Decisiones técnicas standard vs custom basadas en gotchas conocidos |
| indaws-core-developer | local | Knowledge sobre Indaws Core módulos (vía `ecosystem/indaws/`) |
| code-authoring (developer) | developer | orm-modeler, mail-integration, performance-profiler, security-designer, module-migration-scripts, module-creator, traceback-doctor, impact-analyzer — gotchas de comportamiento oculto al codificar/depurar |
| consultoría (consultant) | consultant | project-analysis (standard-vs-custom/GAP/viabilidad), consultant-assistant (hub), project-plan (gotchas en estimación), questionnaire-generator, odoo-configurator, ticket-solver, migration-project-creator |

## REFERENCES

- `common:references/shared/` — N/A: skill content-only, sin conexión a Odoo ni batch ops. Consumers usan shared/.
- `references/_index/HOW-TO-EXTEND.md` — **método** para extender/mejorar el KB (cobertura por preguntas-cliente, anti-alucinación)
- `references/_index/` — los tres índices, que se consultan con `kb.py` y no se leen: `symptoms.md` (síntoma → fichero, `--symptom`), `module-map.md` (modelo → fichero, `--map`), `cheatsheet.md` (quick facts por dominio, `--cheat`)
- `references/functional/` — 29 módulos Community
- `references/enterprise/` — 45 módulos Enterprise
- `references/cross-module/` — 10 transversales
- `references/playbooks/` — 14 flujos end-to-end (README.md = índice)
- `references/ecosystem/` — verticales propietarios + OCA
- `references/engine/README.md` — DELEGADO → `odoo-version-api-reference`
