# Generacion de flujos (gaps custom)

> **CRITICO — contrato real vs. documentado.** El prosa `odoo-flow-tester/references/flow_schema.md`
> esta DESINCRONIZADO con lo que el `validate_flow.py`/executor realmente aceptan. El **contrato
> ejecutable real** (verificado con el validador + `examples/*.json`) es el que manda. Generar
> SIEMPRE en este formato y validar con `validate_flow.py` (ver al final). Ver
> `templates/flow_scenario.yaml` y `examples/CHAIN_BDKSALES_PICKING_SECTIONS.yaml` (ambos PASAN).

## Diferencias del CONTRATO REAL frente a flow_schema.md

| Tema | flow_schema.md (prosa) | CONTRATO REAL (validator/executor) |
|------|------------------------|------------------------------------|
| Version | `target_version` | **`odoo_version`** |
| Preconditions | dict `{required_modules, required_data:[...]}` | **lista plana** `[{model,domain,fields,save_as}]` |
| Datos de create/update | `data:` | **`values:`** |
| Acciones atajo | `confirm/cancel/deliver/invoice/pay` | **NO existen** -> `action: execute` + `method:` |
| Acciones validas | (amplias) | **create, delete, execute, multi_update, return_picking, scrap, search, update** |
| Metodo custom | `action: custom_method` | usar **`action: execute`** + `method:` |

Modulos requeridos: NO van en el flujo (no hay `required_modules` en el contrato real). Se
declaran en `scope.md`/`coverage_matrix.md`; el flow-tester los verifica en su Paso 2
(`verify_data.py`).

## Estructura minima obligatoria de un flujo generado

```yaml
flow_id: CHAIN_<AREA>_<DESC>        # UPPER_SNAKE, regex ^CHAIN_[A-Z][A-Z0-9_]+$  (REQUERIDO)
flow_name: "..."                    # REQUERIDO
description: "..."
odoo_version: "19.0"               # version REAL de la instancia
environment: staging                # staging | production
data: { qty: 2 }                    # valores reutilizables -> {data.*}
preconditions:                      # LISTA PLANA: buscar maestros existentes (NO crearlos)
  - {model: ..., domain: [...], fields: ["id"], save_as: ..., min_records: 1}
steps:                              # REQUERIDO
  - {step: 1, name: "...", action: create, model: ..., values: {...}, save_as: ...}
  - {step: 2, name: "...", action: execute, model: ..., method: action_confirm, ids: ["{result.*}"]}
assertions:                          # verificaciones finales (estado, importes, efectos custom)
  - {model: ..., id: "{result.*}", field: ..., expected: ...}
revert:                              # orden cronologico INVERSO
  - {step: N, action: execute|delete, model: ..., method: ..., ids: ["{result.*}"]}
cleanup:
  strategy: auto                     # auto | manual | sql_fallback
```

## Reglas de autoria (alineadas con el contrato real)

1. **Solo datos transaccionales** en steps; los maestros se localizan en `preconditions`
   (busqueda). NUNCA crear product/partner/journal en steps.
2. **Variables**: `{pre.*}` (precondiciones), `{result.*}` (steps con `save_as`), `{data.*}`,
   `{var.*}`, `{today}`, **`{prefix}`** (resuelve a `[TEST-YYYYMMDD-HHMM]`). OJO: el executor
   `execute_flow.py` **NO** auto-inyecta `{data.test_prefix}` (eso era del esquema en prosa);
   usar **`{prefix}`** en el campo de prefijo, o omitirlo y dejar que el executor lo inyecte solo
   via `PREFIX_FIELD_MAP` (sale.order->client_order_ref, purchase.order->partner_ref,
   account.move/payment->ref, stock.picking->origin).
3. **Cada step que crea registro -> entrada de `revert`** (el validador avisa si falta).
4. **Confirmar/cancelar/etc.** -> `action: execute` + `method:` correcto por modelo:
   `sale.order`=`action_confirm`/`action_cancel`; `purchase.order`=`button_confirm`/`button_cancel`;
   `account.move`=`action_post` / `button_draft`+`button_cancel`; `account.payment`=`action_post`/`action_cancel`.
5. **Custom**: metodos `action_*`/`button_*` propios -> `action: execute` + `method:`.
   Efectos de automatizaciones -> `assertions` (o `detect_cascade` si crean registros).
6. **Detectar registros en cascada** (albaran, factura) -> `detect_cascade` en el step que los
   provoca (mas robusto que buscar por `{result.*_name}`, que el validador no resuelve).
7. **Assertions de personalizaciones**: si un `x_studio_*` es computed/required, anadir una
   assertion que verifique su valor tras el step relevante.

## CONTRATO REAL DEL EXECUTOR (`execute_flow.py`) — verificado en v19, manda sobre todo lo demas

El `execute_flow.py` es mas estricto que `validate_flow.py` y que el prosa. Hallazgos verificados
ejecutando contra una instancia 19.0 real (encodear SIEMPRE asi):

1. **Acciones validas en el EXECUTOR:** `create, delete, execute, return_picking, search, update,
   verify`. **NO** soporta `multi_update`, `scrap`, ni los atajos `confirm/cancel/deliver/invoice/pay`
   (aunque `validate_flow.py` los acepte). Para varias escrituras -> varios pasos `update`.
2. **`create` y `update` usan `values`** (no `data`). `update` admite `ids` (lista).
3. **v19 BLOQUEA metodos privados por RPC** ("Private methods ... cannot be called remotely").
   Por tanto **NO** funcionan `sale.order._create_invoices` ni `account.move._reverse_moves`.
   Alternativas publicas (via wizard):
   - Facturar venta: `create` `sale.advance.payment.inv` con `context {active_model:'sale.order',
     active_ids:[so], active_id:so}` y `values {advance_payment_method:'delivered'}` -> luego
     `execute create_invoices` (publico) en ese wizard.
   - Nota de credito: `create` `account.move.reversal` (context active_ids de la factura) ->
     `execute reverse_moves`/`refund_moves`.
4. **Entregar (stock):** requiere stock disponible. Patron que FUNCIONA: `execute action_assign`
   -> (si queda `assigned`) `execute button_validate` con `context {skip_backorder:true}`. El
   metodo `action_set_quantities_to_reservation` **NO existe** en v19. Si el picking no llega a
   `assigned` (sin stock), la entrega no se puede automatizar -> marcar gap o usar producto con stock.
5. **Variables de lista no interpolan en strings:** `ids: ["{result.picking_move_line_ids}"]` se
   substituye como el string `"[65745]"` y rompe el write. Evitar pasar listas por plantilla;
   preferir metodos que no necesiten los ids de sublineas (p.ej. `button_validate` sobre el picking).
6. **`{result.<save_as>_name}`** solo existe si el modelo tiene campo `name` legible; algunos
   wizards (`sale.advance.payment.inv`) no lo tienen (warning no fatal).
7. **El executor consume JSON** (no YAML). Convertir los `.yaml` a `.json` antes de ejecutar.
8. **Permisos:** el usuario de conexion necesita los grupos FUNCIONALES (ventas/compras/stock/
   contabilidad), no solo `base.group_system`. Si se cambian grupos con el server arrancado,
   **reiniciar el server** (la cache de permisos no se invalida sola entre procesos).
9. **Auto-revert:** ante fallo de un paso, `execute_flow.py` revierte lo creado en ese run. El
   `revert_flow.py` tiene un bug (`int < str`) con multi-registro; para limpiar usar prefijo o ORM.

## Version-sensibilidad (replicar GOTCHAS del flow-tester)

| Tema | <=16.0 | 17.0+ / 18.0 / 19.0 |
|------|--------|---------------------|
| producto almacenable | `type='product'` | `is_storable=True` |
| cantidad en move.line | `qty_done` | `quantity` |
| gasto contabilizar | `action_sheet_move_create` | `action_sheet_move_post` |
| ausencia a borrador | `action_draft` | `action_reset_confirm` |
| suscripcion recurrencia | `recurrence_id` | `plan_id` |
| ausencia fechas (v19) | `date_from/date_to` | `request_date_from/request_date_to` |
| gasto estado (v19) | — | `state` computed -> usar `approval_state`/acciones |
| `action_post()` (v19) | — | puede devolver wizard -> ctx `disable_abnormal_invoice_detection: True` |

Hash fiscal: `button_draft()` puede estar bloqueado; `unlink()` solo lo bloquea el audit trail.
Picking `done` no se cancela: revertir con `return_picking` (wizard `stock.return.picking`).

## Validacion antes del handoff

Validar CADA archivo con el validador del flow-tester:

```bash
# Resolucion agnostica del agente — ver references/shared/skill-path-resolution.md
FT="$(ls -d ${DYNAPPS_SKILLS_ROOT:+"$DYNAPPS_SKILLS_ROOT"/*/skills/odoo-flow-tester} \
    "$HOME/.claude/plugins/cache"/*/*/*/skills/odoo-flow-tester \
    "$HOME/.config/opencode/skills/odoo-flow-tester" \
    "$HOME/.dyn-skills"/*/suites/*/skills/odoo-flow-tester 2>/dev/null | head -1)"
for f in "$OUT"/flows/*.yaml; do python3 "$FT/scripts/validate_flow.py" "$f"; done
```

Corregir cualquier `SCHEMA_ERROR` antes de continuar. Tratar `SCHEMA_WARNING` (revert faltante).

## Nomenclatura de flow_id para gaps

`CHAIN_<MODULO_ABREV>_<FUNCION>` — ej: para `bdk_sales` metodo de seccion en albaran:
`CHAIN_BDKSALES_PICKING_SECTIONS`. Para un campo Studio en sale.order:
`CHAIN_STUDIO_SALEORDER_<CAMPO>`.
