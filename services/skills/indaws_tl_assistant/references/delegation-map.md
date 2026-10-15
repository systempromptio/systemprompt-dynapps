# Delegation map — tl-assistant

Detalle completo de las skills que tl-assistant delega ("Delegates to") y las que lo consumen ("Consumed by"). La forma compacta vive en `SKILL.md`; aquí está el porqué de cada una.

## Delegates to

Routing: el TL decide con **opus** (principal); las consultas mecánicas a Odoo (`team_workload.py`, `estimation_history.py`) y la conexión vía `odoo-pilot` siguen el contrato compartido `references/shared/odoo-connection.md`.

NO duplicar — sugerir al usuario que las invoque:

| Skill | Para |
|-------|------|
| `/dev-task-writer` | **Redactar tareas para devs** (estándar / migración / bug fix) — usar en WF3 y WF6 |
| `/dev-assessment` | Evaluar si necesita desarrollo (pirámide anti-código) |
| `/odoo-module-scanner` | Inventariar componentes de un módulo |
| `/odoo-code-auditor` | Auditoría profunda de calidad |
| `/quality-gate` | Validar código listo para merge/deploy |
| `/odoo-database-analyzer` | Esfuerzo de migración con riesgos |
| `/migration-assistant` | Gestión de migración por fases y gates |
| `/odoo-module-upgrader` | Migración técnica de módulos |
| `/odoo-version-api-reference` | Cambios de API entre versiones |
| `/team-task-assigner` | Planificación semanal + forecast 4 semanas |
| `/dev-assistant` | Planificación y horas del TL |
| `/ticket-report` | Estado tickets del equipo |
| `/odoo-pilot` | Consultas directas a Odoo |

## Consumed by

| Skill | Suite | Para qué |
|-------|-------|----------|
| `dev-task-writer` | common | Calibra estimaciones con `estimation_history` del TL; el TL lo invoca en WF3 (mutuo) |
| `odoo-com-sync` | common | Decisiones de TL en el sync semanal de oportunidades CRM |
| `indaws-env-monitor` | common | Decisiones de TL sobre el parque (churn, migración, renovaciones) |
| `odoo-solution-architect` | developer | El TL arranca el diseño de una tarea antes de asignar al dev |
| `odoo-module-designer` | developer | El TL arranca el diseño de un módulo nuevo antes de asignar |
| `odoo-deep-knowledge` | odoo-knowledge | Decisiones standard vs custom basadas en gotchas conocidos |
| `odoo-context-briefing` | common | Sugerir tareas de TL a partir de colisiones high detectadas en el inventario |
