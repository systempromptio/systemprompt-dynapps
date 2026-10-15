# Presupuesto (sale.order)

Tras aprobar el informe PDF, se crea el presupuesto en el Odoo **corporativo**
(`portal.dynapps.es`, db `master`) vía la skill `odoo-pilot` (scripts; nunca
código ad-hoc). Cliente fijo: **Dynapps Bélgica**.

> El mapeo de abajo está verificado contra los 22 `sale.order` históricos de
> Dynapps NV en el corporativo (jun-2026). Los IDs son los **esperados** en esa
> instancia; resuélvelos por nombre antes de crear y úsalos para validar, no los
> escribas a ciegas (anti-hardcoding).

## Cabecera — valores FIJOS (plantilla)
| Campo | Valor | Cómo resolverlo |
|---|---|---|
| `partner_id` | **Dynapps NV** (≈9183) | search `res.partner` name='Dynapps NV', is_company=True |
| `partner_invoice_id` | **Dynapps NV, Tine Peeters** (≈10927) | contacto de facturación hijo de Dynapps NV; si no existe, usar el propio `partner_id` |
| `company_id` | **Dynapps España** (1) | empresa actual |
| `pricelist_id` | **Tarifa pública (EUR)** (1) | por defecto del partner |
| `currency_id` | **EUR** | viene de la tarifa |
| `fiscal_position_id` | **Régimen Intracomunitario** (≈3) | search `account.fiscal.position` por nombre |
| `payment_term_id` | **vacío** | — |
| `source_id` | **Migraciones con Dynapps** (≈1345) | search `utm.source` name='Migraciones con Dynapps' (campo "Origen") |
| `state` | **draft** (borrador) | nunca confirmar |

**Documento de origen** (`origin`): el **título de la migración** = nombre del cliente/
proyecto (ej. `"Decontex"`, `"Iceberg-Icecubes"`, `"Horafrost"`). Es el mismo nombre
que encabeza el informe PDF.

## Cabecera — valores por DEFECTO (modificables)
| Campo | Valor | Nota |
|---|---|---|
| `user_id` | **usuario actual** (el que lanza la skill) | NO fijar a un comercial concreto: si se archiva da error. Usar el uid autenticado. |
| `team_id` | **Servicios** (≈32) | search `crm.team` name='Servicios' |

## Línea de pedido (1 línea)
| Campo | Valor |
|---|---|
| `product_id` | **Migración ERP** (≈32) — search `product.product` por nombre |
| `name` | `"Migración ERP\n<Cliente> migration project <versión destino>"` — p.ej. `"Migración ERP\nSteelinox migration project 19.0"` |
| `product_uom_qty` | **total de horas calibrado** (el mismo del PDF) |
| `price_unit` | **45** |
| `tax_ids` | IVA intracomunitario de la instancia (≈id 19) — lo aplica solo el producto/fiscal pos; verificar |

> Bolsas de horas sueltas (no migración) usan el producto **"Hora de trabajo"**
> (≈22). Para el flujo del estimator, siempre **"Migración ERP"**.

## Adjuntar el PDF (OBLIGATORIO)
El informe PDF aprobado se adjunta al `sale.order`:
- Crear `ir.attachment` con `execute_method.sh` (odoo-pilot va por JSON2/RPC
  directo, **sí admite base64** en `datas` — la restricción base64 es solo del MCP):
  `res_model='sale.order'`, `res_id=<id del pedido>`, `name='<fichero>.pdf'`,
  `datas=<base64 del PDF>`, `mimetype='application/pdf'`.
- Verificar que el adjunto queda enganchado al pedido (aparece en el chatter / nº de adjuntos).

## Estado y envío
- El pedido se crea **en presupuesto (borrador)** y se deja ahí. **NO confirmar**
  automáticamente (la confirmación llega cuando Dynapps acepta — Fase 2/3).
- **REGLA DURA: NUNCA enviar correo NI notificación desde Odoo.** El envío a Janik
  es **manual** y fuera de Odoo, aunque el usuario diga "mándaselo" (significa que lo
  enviará él).
- **En la práctica = solo `create`:**
  - **NO** llamar a `action_quotation_send` ni a `action_confirm` / `action_quotation_confirm`.
  - **NO** usar `message_post` para "avisar" a nadie.
  - Pasar `context: {mail_notrack: True, tracking_disable: True, lang: 'es_ES'}` al crear
    el `sale.order` y el `ir.attachment`, para no disparar seguimientos/notificaciones a followers.

## Persistir la referencia del presupuesto (OBLIGATORIO — para poder editarlo)
Tras crear el pedido, **guarda su referencia por migración** para poder localizarlo y
editarlo en futuras re-estimaciones (sin crear duplicados). Escribe un fichero
`docs/quotation.json` **junto al informe** de cada proyecto (misma carpeta que el PDF):

```json
{
  "instance": "portal.dynapps.es",
  "db": "master",
  "sale_order_id": 0,
  "sale_order_name": "PRxxxx",
  "partner_id": 9183,
  "line_id": 0,
  "product_id": 32,
  "hours": 0,
  "price_unit": 45,
  "amount_total": 0,
  "attachment_id": 0,
  "report_pdf": "Migration_Estimate_<Cliente>_vX_to_vY.pdf",
  "state": "draft",
  "created": "YYYY-MM-DD",
  "last_updated": "YYYY-MM-DD"
}
```

**Re-estimación = editar, no duplicar.** Al volver a estimar un proyecto:
1. Si existe `docs/quotation.json` con `sale_order_id`, **leerlo y comprobar el estado
   real** del pedido en Odoo (`search_read` por id).
2. Si sigue en **`draft`** → **actualizar** ese mismo pedido (qty/horas de la línea,
   re-adjuntar el PDF nuevo) con `update_record.sh`; refrescar `hours`, `amount_total`,
   `last_updated`. No crear uno nuevo.
3. Si el pedido está **confirmado** (`sale`) o ya no existe → **NO** lo edites en
   silencio: avisa al usuario y pregunta si crear uno nuevo (y enlazar al anterior).
4. Mantén `quotation.json` como la fuente para editar el presupuesto sin volver a
   buscarlo a mano.

> `migration-portfolio` puede leer este `quotation.json` para el seguimiento
> presupuesto-vs-real por migración.

## Conexión Odoo
Scripts de `odoo-pilot` (`auth.sh`, `create_record.sh`, `update_record.sh`,
`execute_method.sh`). Autenticar primero (`eval $(./scripts/auth.sh)`). Incluir
`context: {lang: es_ES}` en las llamadas.

Patrón de conexión y límites: ver `references/shared/odoo-connection.md` y
`references/shared/batch-limits.md`. Sintaxis de campos/tuplas command:
`references/shared/field-mapping-syntax.md`.
