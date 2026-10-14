<!-- verified: v18.0 @ 2026-06 (payment ↔ sale/account_payment/website_sale/sale_subscription; anclas verificadas vs fuente) -->
# Payment Engine — Integrations (Odoo 18.0 CE)

Cómo el motor (`payment`) se conecta con sale, account y website_sale. Núcleo en [`core.md`](core.md);
gotchas del motor en [`gotchas.md`](gotchas.md) (PAYG1-14).

## Cadena de confirmación

1. **El post-procesado es la columna vertebral, NO la notificación.** `_handle_notification_data`
   (`payment/models/payment_transaction.py:637`, `PaymentTransaction._handle_notification_data`) solo hace match + actualiza estado. Toda la lógica
   de negocio (confirmar SO, crear payment, postear factura) vive en `_post_process` (base `:877`
   solo pone `is_post_processed=True`); cada módulo lo extiende vía `super()`.
2. **Confirmación del pedido.** `sale/models/payment_transaction.py::_check_amount_and_confirm_order`
   (`:107`) confirma la SO con `action_confirm()` **solo si hay UNA SO ligada** y
   `_is_confirmation_amount_reached`. El umbral respeta prepago: `sale/models/sale_order.py:1875`, `SaleOrder._get_default_payment_link_values`, compara
   `_get_prepayment_required_amount()` vs `amount_paid` (basta el % de prepago).
3. **Asiento + conciliación.** `account_payment/models/payment_transaction.py::_create_payment`
   (`:135`) crea el `account.payment` (`payment_transaction_id`), lo postea y concilia
   `payment.move_id.line_ids + invoices.line_ids` por `account_id == destination_account_id`
   (`:211`). El token se vincula al asiento (`payment_token_id`, `:164`).

## Webhook / S2S / backend

- **Webhook (patrón).** Cada provider expone su controller que acaba llamando
  `tx_sudo._handle_notification_data(code, data)`. Canónico: `payment_stripe/controllers/main.py::stripe_webhook`
  (`:72`, route `/payment/stripe/webhook`, `auth='public'`, `csrf=False`), verifica firma con
  `_verify_notification_signature` antes de procesar.
- **Checkout website_sale.** `website_sale/controllers/payment.py::shop_payment_transaction` (`:25`)
  bloquea la SO (`FOR NO KEY UPDATE NOWAIT`), valida importe vs `amount_total` y crea la tx con
  `sale_order_ids = [Command.set([order_id])]`.
- **S2S con token.** `payment/controllers/portal.py::_create_transaction` (`:285`): si `flow=='token'`
  ejecuta `tx._send_payment_request()` (`:509`) de inmediato; website_sale puede diferirlo con
  `delay_payment_request=True`. Base de los cobros recurrentes de `sale_subscription`.
- **Captura/void/refund** (`action_capture`/`void`/`refund`): los parciales crean `_create_child_transaction`.

## Gotchas cross-module

**PAYG15. Sin webhook, el cobro queda "colgado" hasta el cron.** Si el cliente no vuelve a
`/payment/status/poll` (`post_processing.py::poll_status:40` → `_post_process`), la SO no se confirma
en el momento; reprocesa `_cron_post_process` (10 min, **`active=False` por defecto**, ventana 4 días).

**PAYG16. `_post_process` es idempotente por flag, NO por lógica, y el flag se RESETEA.**
`is_post_processed` se pone True en `_post_process` (`:887`) pero `_update_state` (`:821`) lo resetea
a False al cambiar de estado. Una tx `pending→done` se **re-post-procesa**.

**PAYG17. Pagos agrupados rompen la confirmación.** `_check_amount_and_confirm_order` (sale `:121`)
solo confirma si `len(tx.sale_order_ids) == 1`. Ligar varias SO a una tx → ninguna se confirma sola.

**PAYG18. Conciliación silenciosa por `destination_account_id`.** `_create_payment` (account `:211`)
concilia por `account_id == destination_account_id`. Si la factura usa otra cuenta a cobrar, la
conciliación NO ocurre y el payment queda suelto. El destino se fuerza desde `payment_term_lines[0]`.

**PAYG19. `validation` no crea payment ni confirma pedido.** Las tx `operation=='validation'` (alta
de token) se saltan `_create_payment` (account `:118`) y la confirmación de SO (sale `:59`). No
esperes asiento de una validación de tarjeta.

**PAYG20. Importe negativo → pago outbound.** `_create_payment` (account `:156`): `amount: abs(...)`,
`payment_type: 'inbound' if amount>0 else 'outbound'`. Un importe negativo genera pago de salida.

**PAYG21. Facturación automática post-cobro es opcional y asíncrona.** sale `_post_process` (`:90`)
solo factura si `ir.config_parameter 'sale.automatic_invoice'`; el email puede diferirse a
`send_invoice_cron` (diario, `active=False`, y apunta al modelo `payment.transaction`).

**PAYG22. El autor del chatter cambia según contexto.** `_log_message_on_linked_documents`
(account `:232`, sale `:137`): backend (`payment_backend_action=True` o SUPERUSER) postea como el
usuario; los flujos web postean como el cliente (`self.partner_id`).

**PAYG23. Captura/void/refund parciales crean tx HIJAS, no modifican la original.** El payment y la
conciliación se gestionan en la hija (`child_transaction_ids`); `_create_payment` resuelve las
facturas vía `source_transaction_id`. No busques el refund en la tx original.

**PAYG24. Anular calcula el importe ya capturado por los hijos.** `action_void` (`:274`) hace
`_send_void_request(amount_to_void = tx.amount - captured_amount)` sumando hijos `done` de la misma
`operation`. Anular tras una captura parcial solo libera el resto.
