---
name: indaws-odoo-effort-estimator
version: 1.3.2
effort: low
model: sonnet
description: >
  Referencia centralizada de estimacion de horas para proyectos Odoo. Contiene
  velocidades de equipo, horas por app, horas por tipo de tarea, clasificacion
  de complejidad, contingencias y factores AI. Usa este skill cuando necesites
  estimar horas de implantacion, presupuestar un proyecto, evaluar complejidad
  de un desarrollo, o calcular contingencia de migracion. Triggers directos:
  "cuantas horas", "estimar proyecto", "presupuesto horas", "contingencia".
  Triggers indirectos: lo invoca cualquier skill que necesite estimar horas.
---

# Odoo Effort Estimator

Referencia centralizada de datos de estimacion para proyectos Odoo. Este skill NO ejecuta nada — es una libreria de datos numericos que otros skills consultan para estimar horas de forma consistente.

## Cuando se usa

- **Directamente**: El usuario pregunta cuantas horas lleva algo
- **Indirectamente**: cualquier skill que necesite horas, velocidades, complejidad,
  contingencia o factores IA delega aqui.

## Estructura de referencia

| Fichero | Contenido |
|---------|-----------|
| `references/team-velocities.md` | Ritmos de equipo, split 70/30, gestion de proyecto, soporte produccion |
| `references/hours-per-app.md` | Tabla unificada: app x complejidad → semanas + senales |
| `references/hours-per-task-type.md` | CONFIG/STUDIO/DEV/MIGR/TEST/TRAIN: peso → horas |
| `references/hours-per-component.md` | Horas base por componente tecnico (modelos, vistas, OWL, integraciones, overhead) |
| `references/hours-migration.md` | Migracion: BD por volumen, iteraciones, salto de version y formula completa |
| `references/complexity-classification.md` | STUDIO/MINOR/STANDARD/COMPLEX: umbrales unificados |
| `references/risk-contingency.md` | Contingencia por tamano de proyecto y tipo de trabajo |
| `references/ai-factors.md` | Factores IA de esfuerzo. Fuente de verdad unica del ecosistema |
| `references/golden-examples.md` | Ancla de regresión: casos input→horas congelados (anti-drift) |

## Que NO hace

- No decide si usar Studio o desarrollo (eso es `dev-assessment`)
- No genera presupuestos (eso es `quotation-builder`)
- No planifica proyectos (eso es `project-plan`)
- No analiza codigo (eso es `odoo-database-analyzer`)
- No ejecuta nada en Odoo

## Como usar desde otros skills

```
Para horas, velocidades, complejidad, contingencia o factores IA:
delega en la skill `odoo-effort-estimator`.
```

No apuntes a sus ficheros internos: la delegacion entre skills es por nombre.

## GOTCHAS
1. **Hour tables are base figures, WITHOUT AI** — you overestimate unless you apply the factors from `ai-factors.md`. The factor goes per component, never flat on the total.
2. **AI factor is multiplicative, not additive** — a 0.7 AI factor on 10h = 7h, not 10h-3h.
3. **Migration contingency stacks** — version jump contingency + code complexity contingency. Don't apply just one.
4. **Studio hours are not zero** — Studio customizations take 30-50% of equivalent dev time, not 0%.
5. **Multi-company doubles security work** — always add 20-30% for ir.rules, check_company, testing per company.
6. **Testing is never included in dev estimates** — always add 20-30% on top of development hours. That 20-30% is *effort*, not coverage: the coverage target is 90%.

## Sanity gates (antes de entregar)

Invariantes de `references/hours-per-app.md`, `hours-per-task-type.md`, `risk-contingency.md`. Comprobar sobre cada estimación emitida; fuera de banda → STOP y revisa.

- [ ] **Horas por app dentro de la banda documentada** (semanas × complejidad de `hours-per-app.md`; config inicial siempre 1 sem/10h) → fuera de rango sin señal de complejidad que lo justifique → ajusta.
- [ ] **Horas por tarea dentro del rango de su peso** (peso1 0.5-1h … peso5 24-80h+; por tipo CONFIG/STUDIO/DEV/MIGR/TEST/TRAIN) → fuera → ajusta o justifica.
- [ ] **Contingencia aplicada según riesgo** y DESPUÉS del cálculo base (peq +10% / med +18% / grande +25%; migración apila version-jump + complejidad de código), redondeo ceil → si falta o se aplicó antes → corrige.
- [ ] **Testing nunca incluido en horas de dev** (+20-30% aparte); **Studio no es 0h** (30-50% del dev equivalente) → revisa.
- [ ] **Factor IA aplicado por componente** y ANTES de la contingencia; overhead tecnico a ×1.00; ninguna tarea de cliente por debajo de 1h → si falta o se aplicó plano al total → corrige.
- [ ] **Nota a quien consuma estos datos**: debe aplicar estos mismos gates sobre las horas que tome de aquí; este skill es SSOT pero no valida por sus consumidores.

## Delegates to
None — this is a pure reference skill.

## Consumed by
| Skill | Suite | For what |
|-------|-------|----------|
| odoo-database-analyzer | common | Hours estimation for migrations |
| migration-assistant | common | Quick estimation per phase |
| dev-assessment | common | Complexity classification |
| project-analysis | consultant | Project hour budgets |
| project-plan | consultant | Task hour planning |
| migration-project-creator | consultant | Hours for migration project tasks/dates |
| quotation-builder | sales | Quote hour calculations |
| odoo-migration-pitch | sales | Migration effort estimation |

For shared ecosystem patterns, see `common:references/shared/`.

## Workflow
1. Read project scope (modules, apps, features)
2. Classify each item by complexity (simple/medium/complex)
3. Apply base hours from the tables in references/
4. Apply AI factors per component (`ai-factors.md`)
5. Apply risk contingency (`risk-contingency.md`)
6. Output structured estimate with breakdown
