<!-- verified: v18.0 base + v19.0 diffs @ 2026-06 (payment CE engine: provider/transaction/token/method; anclas verificadas vs fuente v18 y v19) -->
# Payment Engine — Core Reference (Odoo 18.0 CE)

Motor de pago transversal (`payment`, CE). Lo consumen `website_sale` (checkout), `sale`
(confirmar pedido), `account_payment` (asiento + conciliación) y `sale_subscription` (cobro S2S).
**POS NO usa este motor** (usa `pos.payment.method` con terminal; ver [[pos]]).

## Cuatro modelos

| Modelo | Rol |
|--------|-----|
| `payment.provider` | El acquirer configurado (Stripe, PayPal…). Estado, features, restricciones. |
| `payment.transaction` | Un intento de cobro/reembolso con su máquina de estados. |
| `payment.token` | Método guardado (tarjeta tokenizada) para cobros futuros. |
| `payment.method` | Marca/método de pago (tarjeta, bizum…), primario vs brand. |

## payment.provider — campos clave

| Campo | Notas (ancla) |
|-------|---------------|
| `state` | `disabled`/`enabled`/`test` (default `disabled`, `copy=False`) — `payment/models/payment_provider.py:34`, `class PaymentProvider`. En `test` se procesa un pago falso. |
| `code` | código técnico del acquirer (`_get_code`, `:726`); recalcula features con `@api.depends('code')`. |
| `allow_tokenization` (`:55`) · `capture_manually` (`:60`) | habilitan guardar token y autorizar→capturar. |
| `available_currency_ids` (`:108`) · `maximum_amount` (`:121`) | restricción de moneda e importe (vacío = todas). |
| `support_tokenization` · `support_manual_capture` · `support_express_checkout` · `support_refund` | `_compute_feature_support_fields` (`:219`); base todo `None` salvo `support_refund='none'`, cada provider sobreescribe. |

`_get_compatible_providers` (`payment/models/payment_provider.py:455`, `class PaymentProvider`, `@api.model`) filtra por compañía, `state in
('enabled','test')`, país, importe máximo, moneda, tokenización y express checkout.

## payment.transaction — máquina de estados

`state` (`payment/models/payment_transaction.py:58`, `class PaymentTransaction`): `draft → pending → authorized → done` (+ `cancel`, `error`).
Transiciones (cada una solo desde estados permitidos, vía `_update_state`, `:765`):

| Método | Desde |
|--------|-------|
| `_set_pending` (`:678`) | `draft` |
| `_set_authorized` (`:695`) | `draft`, `pending` |
| `_set_done` (`:712`) | `draft`, `pending`, `authorized`, **`error`** |
| `_set_canceled` (`:730`) | `draft`, `pending`, `authorized` |
| `_set_error` (`:748`) | `draft`, `pending`, `authorized` |

`operation` (`:70`): `online_redirect`/`online_direct`/`online_token`/`validation`/`offline`/`refund`
— **no fiable si el state es draft/pending** (comentario literal en `:70`).

## Flujos de cobro

- **Redirect / direct**: `_get_processing_values` (`:402`) renderiza el form de redirección (si
  `operation in ('online_redirect','validation')`); el provider responde por webhook/return →
  `_handle_notification_data` (`:637`) → `_get_tx_from_notification_data` + `_process_notification_data`
  (stubs que cada provider sobreescribe). **⚠️ v19 reescribe todo este flujo — ver `v19_changes.md`.**
- **Token (S2S)**: `_send_payment_request` (`:509`) cobra contra un `payment.token` guardado
  (`online_token`/`offline`); base de los cobros recurrentes de suscripción.
- **Backend**: `action_capture`/`action_void`/`action_refund` (`:246`/`:274`/`:289`) — captura,
  anula o reembolsa; los parciales crean **transacciones hijas** (`_create_child_transaction`, `:600`).

## payment.token / payment.method

- `payment.token`: `active` (`payment/models/payment_token.py:39`, `class PaymentToken`), `provider_ref` (`:31`). No se puede **desarchivar**
  si su method está inactivo o el provider `disabled` (`payment/models/payment_token.py:78`, `PaymentToken.write`).
- `payment.method`: `active`, `primary_payment_method_id`/`brand_ids`, `is_primary`.
  `_get_compatible_payment_methods` (`payment/models/payment_method.py:209`, `PaymentMethod._get_compatible_payment_methods`) solo devuelve **primarias**.

Trampas en [`gotchas.md`](gotchas.md) (PAYG*); flujos cross-module en [`integrations.md`](integrations.md);
capacidades de consultor en [`capabilities.md`](capabilities.md).
