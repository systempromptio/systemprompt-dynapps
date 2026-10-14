---
name: indaws-migration-billing-tracker
version: 0.1.1
effort: high
model: opus
description: >-
  Hace el seguimiento de avance por gates y facturación por hitos (no T&M) de
  las migraciones Dynapps que ejecuta Indaws (45 EUR/h): vista de cobro para
  Julio y control presupuesto-vs-real por gate; crea hito draft en Odoo al
  alcanzar un gate. Triggers: "como va el cobro de las migraciones", "que puedo
  facturar de las migraciones", "estado de gates de migracion", "presupuesto vs
  real migracion", "vista de cobro Julio", "billing de migracion Dynapps",
  "que hitos de migracion estan listos".
allowed-tools: [Bash, Read, Write, Edit, Glob, Grep, AskUserQuestion, Skill, Agent]
---

# Migration Billing Tracker — Seguimiento + Facturación por gates

Hace el **seguimiento del avance por gates** de las migraciones que Indaws ejecuta
**para Dynapps** y produce la **vista de cobro** (qué hito está listo para facturar)
y el **control presupuesto-vs-real por gate** (horas estimadas en Fase 1 vs horas
reales). Facturación **por hitos/gates**, no por T&M. Va directo al estado y los
desvíos; no ejecuta lo técnico.

> Contexto, fases y reglas del suite en `migration-lab:DESIGN.md`.

Conexión a Odoo y límites: ver `references/shared/odoo-connection.md` y
`references/shared/batch-limits.md`.

## Gates de facturación (fuente de verdad)
Tres gates, en orden (detalle en `references/gates-and-billing.md`):
1. **Informe enviado** — entregado el informe/presupuesto a Janik (fin Fase 1).
2. **Entrega** — BD migrada + módulos migrados subidos a GitLab (dump entregado).
3. **Salida a producción** — go-live confirmado por Dynapps.

Cada gate alcanzado habilita su **hito de facturación**. El reparto de € por gate
sale del total del presupuesto (sale.order de Fase 1).

## Inputs (pregunta solo lo imprescindible)
1. **Alcance**: una migración (cliente/proyecto) o **toda la cartera** (default: cartera).
2. **Vista**: `cobro` (qué facturar ya), `gates` (estado de avance), `desvio`
   (presupuesto-vs-real), o `todo` (las tres). Default: `todo`.
3. (Opc.) **Periodo** para el corte de cobro (default: mes en curso — la vista de Julio).

## Workflow

1. **Resolver cartera** — Identifica las migraciones en curso y sus proyectos en
   Odoo. Si falta el mapa cliente↔proyecto↔sale.order, pídelo o delega en
   `migration-portfolio`.
2. **Estado de gates** [delega] — Lee los gates de cada migración vía
   `migration-assistant` (gates = fuente de verdad). Marca el último gate superado
   y el siguiente pendiente. Ver `references/gates-and-billing.md`.
3. **Horas reales** [delega] — Obtén horas reales consumidas por proyecto con
   `timesheet-report`. Mapea horas → bucket/gate cuando sea posible.
4. **Carga-vs-facturación** [delega] — Para el cruce de carga del equipo contra lo
   facturable usa `workload-billing-monitor` (no recalcules carga aquí).
5. **Presupuesto-vs-real por gate** [opus] — Aplica `references/budget-vs-actual.md`:
   estimado Fase 1 (por bucket/gate) vs real; calcula **desvío** (h y %), € a 45 €/h,
   y marca rojo si el real supera el estimado del gate.
6. **Vista de cobro (Julio)** — Lista los gates **alcanzados y aún no facturados**:
   migración, gate, € del hito, estado del hito en Odoo (borrador/pendiente).
7. **Hito de facturación** [delega, opcional] — Si un gate está alcanzado y el
   usuario lo pide, crea/deja listo el **hito de facturación en borrador** vía
   `odoo-pilot` (ver `references/gates-and-billing.md`). **No confirmar. No postear
   factura. No enviar correo.**

## GOTCHAS
1. **NUNCA enviar correo desde Odoo.** El skill informa del cobro y deja el hito en
   borrador; el envío a Janik es manual y fuera de Odoo, aunque digan "mándaselo".
2. **NUNCA confirmar ni validar/postear la factura.** Solo borrador del hito. La
   confirmación es decisión humana (Julio).
3. **Facturación por gates, no por horas.** Las horas reales sirven para **control de
   desvío**, no para facturar T&M. El importe de cada hito viene del presupuesto.
4. **Un gate no facturado ≠ trabajo no hecho.** Un gate puede estar superado y aún
   sin hito creado: eso es justo lo que la vista de cobro debe destacar.
5. **El estimado es por bucket (Fase 1), el real es por timesheet.** El mapeo
   bucket→gate no es 1:1; documenta supuestos (ver `budget-vs-actual.md`) y no
   inventes precisión que no hay.
6. **No recalcules la estimación.** El estimado lo fijó `migration-estimator`; aquí
   solo se compara contra el real. Si no hay estimado, márcalo como dato faltante.
7. **Gates = fuente de verdad de `migration-assistant`.** No deduzcas el estado del
   gate del timesheet ni del estado del sale.order; léelo de los gates.
8. **Desvío rojo = aviso, no bloqueo.** Reporta el sobrecoste; la decisión de
   renegociar con Dynapps es del responsable, no del skill.

## NEVER
- Enviar email desde Odoo, confirmar el hito o postear/validar la factura.
- Facturar por T&M ni derivar el importe del hito de las horas reales.
- Recalcular la estimación de Fase 1 o el estado de los gates por tu cuenta.

## ALWAYS
- Usar 45 €/h para valorar desvíos y los gates como fuente de verdad del avance.
- Separar claramente **estimado (Fase 1)** y **real (timesheet)** en el desvío.
- Destacar en la vista de cobro los gates **alcanzados y no facturados**.

## Delegates to
| Skill | Suite | For what |
|-------|-------|----------|
| `migration-assistant` | common | Estado de gates por migración (fuente de verdad) |
| `timesheet-report` | common | Horas reales consumidas por proyecto |
| `workload-billing-monitor` | management | Cruce carga del equipo vs facturación |
| `odoo-pilot` | common | Crear el hito de facturación en borrador vía API |

## Consumed by
| Skill | Suite | For what |
|-------|-------|----------|
| `migration-closeout` | migration-lab | Toma el desvío horas-vs-estimado y el estado de cobro para el cierre/factura final |

## REFERENCES
- `references/gates-and-billing.md` — Gates, mapeo gate→hito, vista de cobro, regla NO-email
- `references/budget-vs-actual.md` — Cálculo estimado(Fase 1) vs real(timesheet) por gate
- `migration-lab:DESIGN.md` — Diseño del suite y contexto de negocio
