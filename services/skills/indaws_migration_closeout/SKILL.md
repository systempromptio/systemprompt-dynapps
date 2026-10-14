---
name: indaws-migration-closeout
version: 0.1.1
effort: high
model: opus
description: >-
  Cierra formalmente una migración Odoo ejecutada por Indaws para Dynapps
  tras go-live: handover package (GitLab + dump + docs), post-mortem con KPIs
  (horas reales vs estimado Fase 1, gates fallidos) y factura de liquidación en
  DRAFT. Triggers: "cerrar migracion", "closeout migracion", "post-mortem
  migracion", "handover migracion", "factura final migracion", "liquidacion
  migracion", "pasar a mantenimiento", "kpis de la migracion".
allowed-tools: [Bash, Read, Write, Edit, Glob, Grep, AskUserQuestion, Skill, Agent]
---

# Migration Closeout — Finalización de migraciones Dynapps

Cierre **formal** de una migración que Indaws hizo **para Dynapps**: handover,
post-mortem con KPIs (desvío de horas reales vs estimado de Fase 1), factura
final/liquidación **en borrador**, y transición a soporte/mantenimiento. No
re-valida calidad técnica ni redacta la aceptación del cliente: lo **delega**.

> Contexto, reglas y fases del suite en `migration-lab:DESIGN.md`.

Conexión a Odoo: ver `references/shared/odoo-connection.md`.

## Inputs (pregunta solo lo imprescindible)
1. **Proyecto/cliente** de la migración (para localizar el `project.project` y el
   sale.order/horas en Odoo).
2. **Estimado de Fase 1** (horas del informe `migration-estimator`) si no se
   deduce del sale.order original.
3. **Versión origen → destino** y **repo GitLab** del código migrado.

## Workflow

1. **Pre-requisito: calidad técnica** [delega] — Antes de cerrar, exige el pase
   de `quality-gate` sobre el código migrado. Si no ha pasado, **para**: no se
   cierra una migración sin gate verde.
2. **Handover** — Construye el paquete de entrega siguiendo
   `references/handover-checklist.md`: código en **GitLab**, **dump migrado**
   entregado, docs mínimas. Recuerda: Indaws NO levanta entorno de test (lo monta
   Dynapps). El go-live / copia de config es de `odoo-copy-staging-master`.
3. **Aceptación del cliente** [delega] — El documento de aceptación lo redacta
   `project-validation-drafting`. Este skill solo lo **referencia** en el cierre.
4. **Post-mortem + KPIs** [opus] — Aplica `references/postmortem-kpis.md`: lee
   horas **reales** (timesheets/`project.project.hour.line` vía `odoo-pilot`) y
   compáralas con el **estimado de Fase 1**. Calcula desvío %, gates fallados,
   retrabajos, y módulos que se salieron del cap ~20h.
5. **Factura final / liquidación** [delega odoo-pilot] — Ver
   `references/final-billing-and-handoff.md`: genera la factura por las horas
   reales a **45 €/h**, la deja **en BORRADOR**. **No confirmar. No enviar correo.**
6. **Transición a mantenimiento** — Cierra el proyecto y define el traspaso a
   soporte (contacto, repo, alcance de correcciones post-go-live). Ver
   `references/final-billing-and-handoff.md` §Transición.

## GOTCHAS
1. **NUNCA enviar correo desde Odoo.** La factura final se deja en borrador; el
   envío a Dynapps es manual y fuera de Odoo, aunque el usuario diga "mándasela".
2. **No cierres sin `quality-gate` verde.** El closeout asume calidad ya validada;
   no la re-implementes aquí, exígela como pre-requisito y para si falta.
3. **Horas reales ≠ estimado.** El KPI clave es el **desvío**; usa horas reales de
   timesheets, no las del presupuesto. `project.project.hour.line` no tiene
   `date_from`/`date_to`, usa `date`.
4. **Liquidación = horas reales, no estimadas.** La factura final factura lo
   ejecutado a 45 €/h; el sale.order de Fase 1 era una estimación.
5. **Indaws NO hace testing funcional ni levanta test.** No incluyas eso en el
   handover; el dump migrado + código en GitLab es la entrega.
6. **La aceptación la redacta otro skill.** No generes el documento de aceptación
   aquí; delega a `project-validation-drafting`.
7. **Excluir CTOs del análisis de horas**: David Gómez Quilón (194), Jesús Vargas
   Sánchez (193). Excluir holiday_id/global_leave_id de los partes.

## NEVER
- Enviar email desde Odoo ni confirmar/validar la factura final automáticamente.
- Cerrar la migración sin el pase de `quality-gate`.
- Re-redactar la aceptación del cliente (es de `project-validation-drafting`).
- Facturar por horas estimadas en vez de horas reales.

## ALWAYS
- Comparar horas reales vs estimado de Fase 1 y reportar el desvío %.
- Dejar la factura final en borrador, a 45 €/h, marca Dynapps.
- Definir el traspaso a soporte/mantenimiento al cerrar el proyecto.

## Delegates to
| Skill | Suite | For what |
|-------|-------|----------|
| `quality-gate` | common | Validación de calidad técnica del código migrado (pre-requisito) |
| `project-validation-drafting` | common | Documento de aceptación del cliente |
| `odoo-copy-staging-master` | common | Go-live / copia de configuración a producción |
| `odoo-pilot` | common | Horas reales y creación de la factura final vía API |

## Consumed by
| Skill | Suite | For what |
|-------|-------|----------|
| `migration-billing-tracker` | migration-lab | Cierra la migración tras el último gate de cobro |
| `migration-portfolio` | migration-lab | Marca migraciones de la cartera como finalizadas |

## REFERENCES
- `references/handover-checklist.md` — Paquete de entrega (GitLab + dump + docs)
- `references/postmortem-kpis.md` — KPIs: desvío de horas, gates, retrabajos
- `references/final-billing-and-handoff.md` — Factura final (borrador) + transición
- `migration-lab:DESIGN.md` — Diseño del suite y contexto de negocio
