# Carril Upgrade Lab (migraciones grupo Dynapps)

> Carril **ligero** para las migraciones que Indaws realiza para el grupo Dynapps.
> Datos operativos (checklists, gates, pesos): `scripts/migration_spec.json` → `gates.upgrade_lab`, `checklists.ul_*`, `phase_weights.upgrade_lab`.

## Cuándo aplica

El proyecto es Upgrade Lab si: es para el **grupo Dynapps**, sus gates son **A/B/C** (no 0-7), tiene **4 tareas `[MIG]`** (no las 5 internas + INFRA), o el creador lo montó con `--profile upgrade-lab`. Ante la duda, pregunta al usuario "¿es un proyecto Upgrade Lab (Dynapps) o una migración interna completa?".

**Si es Upgrade Lab, NO guíes por las 7 fases internas.** Usa las 4 fases de aquí.

## Diferencias clave frente al proceso interno

| Aspecto | Interno | Upgrade Lab |
|---|---|---|
| Servidor | Lo despliega/valida Indaws | Lo pone el cliente — sin `[INFRA]` |
| Kickoff | Reunión + propuesta + firma (Gate 0, COO) | Análisis + informe de módulos. Sin propuesta ni firma |
| Estrategia | 1:1 o Refactoring | Siempre **1:1** (sin Studio) |
| Testeo funcional | Lo ejecuta Indaws | Lo ejecuta el cliente; Indaws solo smoke técnico |
| Ensayo / Go-Live | Rehearsal + Go-Live por Indaws | Sin rehearsal. Entrega de código+BD (o rama Odoo.sh) |
| Cierre | Soporte 2 sem + retrospectiva (Gate 7, COO) | Cliente reporta → corregir → re-migración final + fecha producción |
| Gates | 8 (COO/CTO/cliente) | **3 internos A/B/C** (solo TL/PM) |

## Las 4 fases y sus gates

| Fase | Tarea `[MIG]` | Qué se hace | Gate al cierre |
|---|---|---|---|
| UL-1 | Análisis e informe de módulos | Inventario + clasificación 1:1 + APIs deprecadas + estimación → **informe al cliente** | **Gate A** (TL) |
| UL-2 | Migración de código | Migrar cada módulo 1:1, CI verde, code review | — |
| UL-3 | Migración de base de datos | Upgrade BD, validación técnica/datos, **entrega al cliente** (o rama Odoo.sh) | **Gate B** (TL) |
| UL-4 | Corrección de feedback y re-migración final | Triar feedback, corregir, acordar fecha producción, **re-migración final limpia** | **Gate C** (TL/PM) |

Además: una tarea **Gestión del proyecto** (imputación). No hay "Reuniones" (sin kickoff ni dailies).

Los entregables exactos de cada gate están en `migration_spec.json` → `gates.upgrade_lab.{A,B,C}.checklist`. Los checklists de tarea en `checklists.ul_analysis / ul_modules_fixed / ul_database / ul_feedback`.

## Guía por rol (Upgrade Lab)

- **Dev:** ejecuta UL-1→UL-4. En UL-3 entrega código+BD (o despliega rama Odoo.sh) para que el cliente valide. En UL-4 corrige el feedback y hace la re-migración final limpia.
- **TL:** aprueba Gate A (informe enviado), Gate B (entrega lista), Gate C (cierre). Todo interno, sin COO/CTO.
- **PM:** coordina con el cliente Dynapps; co-aprueba Gate C (fecha de producción).
- **BC:** rol mínimo aquí — el testeo funcional lo hace el cliente, no Indaws.

## Lo que NO se hace en Upgrade Lab

Despliegue/validación de servidor (`[INFRA]`), testing funcional nuestro (9 flujos + técnico + acta), rehearsal/Fase 5, Go-Live ejecutado por nosotros, soporte intensivo 2 semanas, retrospectiva formal, aprobaciones de COO/CTO, firma del cliente.
