# Gates y facturación por hitos

Fuente: `migration-lab:DESIGN.md` (§3.5, Fase 3). Indaws factura a Dynapps **por
hitos/gates**, no por T&M. El importe total lo fija el presupuesto (sale.order de
Fase 1, 45 €/h); aquí solo se **reparte por gate** y se controla el cobro.

## Los 3 gates
Orden estricto; un gate no puede facturarse antes de alcanzarse.

| # | Gate | Significado | Quién lo confirma |
|---|---|---|---|
| 1 | **Informe enviado** | Informe + presupuesto entregados a Janik (fin Fase 1) | Indaws (manual, fuera de Odoo) |
| 2 | **Entrega** | BD migrada + módulos migrados subidos a GitLab; dump entregado | Indaws entrega / Dynapps recibe |
| 3 | **Salida a producción** | Go-live en producción del cliente final | Dynapps |

> Recordatorio de alcance: Indaws NO hace testing funcional ni levanta entorno de
> test. El testing entre gate 2 y gate 3 lo hace Dynapps.

## Estado de gates = fuente de verdad
El estado del gate se **lee de `migration-assistant`** (gates en Odoo), no se deduce
del timesheet ni del estado del sale.order. Para cada migración:
- último gate **superado**,
- siguiente gate **pendiente**,
- si el gate superado tiene ya **hito de facturación** creado o no.

## Reparto del importe por gate
El total del presupuesto se reparte entre los 3 gates. Salvo indicación distinta del
usuario/Dynapps, usar un reparto por defecto y **dejarlo explícito** en el informe
(es un supuesto, no un dato pactado):

| Gate | % por defecto del total |
|---|---|
| 1 Informe enviado | 0% (suele ir incluido / no se factura suelto) |
| 2 Entrega | 60% |
| 3 Salida a producción | 40% |

Si Dynapps pactó otro reparto, usar ese. Nunca derivar el importe del hito de las
horas reales (eso sería T&M).

## Vista de cobro (corte de Julio)
Lista, por migración, los gates **alcanzados y aún no facturados**:

| Migración | Gate alcanzado | € del hito | Hito en Odoo | Acción sugerida |
|---|---|---|---|---|
| Geosquare | 2 Entrega | … | no creado | crear borrador |

"Acción sugerida" nunca incluye confirmar/enviar: como mucho **crear el borrador**.

## Crear el hito de facturación (borrador) — vía odoo-pilot
Solo si el gate está alcanzado y el usuario lo pide. Usar los scripts de `odoo-pilot`
(`auth.sh`, `create_record.sh`, `update_record.sh`, `execute_method.sh`); nunca código
ad-hoc. Autenticar primero; incluir `context: {lang: es_ES}`.

- Crear/marcar el **hito** (milestone) o la **línea facturable** del gate en el
  sale.order/proyecto, en estado **borrador**.
- Importe = € del gate (del reparto), descripción = migración + gate.
- **NO confirmar el sale.order. NO postear ni validar la factura. NO enviar correo.**

## REGLA DURA — NUNCA enviar correo desde Odoo
El skill informa del estado de cobro y deja el hito en borrador. El envío a Janik es
**manual y fuera de Odoo**, aunque el usuario diga "mándale el cobro": significa que
lo enviará él, no el skill. Tampoco se usa la acción "Enviar por email" de
factura/presupuesto.

## Conexión Odoo
Patrón y límites: `references/shared/odoo-connection.md`,
`references/shared/batch-limits.md`. Sintaxis de campos/tuplas command:
`references/shared/field-mapping-syntax.md`.
