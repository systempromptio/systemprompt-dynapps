<!-- verified: v18.0 @ 2026-06 (payment CE engine; anclas verificadas vs fuente) -->
# Payment Engine — Gotchas (Odoo 18.0 CE)

Comportamientos ocultos del motor de pago. Mecánica en [`core.md`](core.md); flujos cross-module
(sale/account/website_sale) en [`integrations.md`](integrations.md) (PAYG15+).

**PAYG1. El `operation` no es fiable en draft/pending.** Comentario literal en
`payment/models/payment_transaction.py:70`, `class PaymentTransaction`: *"This should not be trusted if the state is draft or pending."* No
ramifiques lógica por `operation` hasta que la tx esté en `authorized`/`done`.

**PAYG2. Los estados no revierten libremente y un estado ilegal se ignora SIN error.**
`_update_state` (`payment/models/payment_transaction.py:765`, `PaymentTransaction._update_state`): si la tx ya está en el target → solo log INFO
(`:799`); si la transición es ilegal → log WARNING y **se ignora** (`:805`), no lanza excepción.
`_set_done` sí admite venir de `error` (`:721`), pero `_set_pending` solo desde `draft`.

**PAYG3. Cambiar el `state` de un provider archiva sus tokens.** `write` (`payment/models/payment_provider.py:307`, `PaymentProvider.write`)
llama `_archive_linked_tokens` (`:375`) en cualquier transición que no sea "habilitar" o "sin
cambio" — incluido `test`→`enabled`. Los clientes pierden sus tarjetas guardadas al reconfigurar.

**PAYG4. No puedes desarchivar un token con method inactivo o provider disabled.**
`payment_token.write` (`payment/models/payment_token.py:78`, `PaymentToken.write`) lanza `UserError`; archivar dispara `_handle_archiving`
en sudo (`:98`).

**PAYG5. `maximum_amount` se compara en la moneda de la COMPAÑÍA, no en la del pago.**
`_get_compatible_providers` convierte el importe con `currency._convert(... company.currency_id ...)`
antes de comparar (`payment/models/payment_provider.py:510`, `PaymentProvider._get_compatible_providers`). Solo aplica si no es validación y hay moneda.

**PAYG6. El cron de post-procesado se auto-activa/desactiva según haya providers no-disabled.**
`_toggle_post_processing_cron` (`payment/models/payment_provider.py:358`, `class PaymentProvider`) pone `cron_post_process_payment_tx.active`
a True/False al crear (`:303`) y al cambiar state (`:326`). **Sin providers activos el cron no
despierta** → no asumas que está corriendo.

**PAYG7. El cron de post-proceso solo reintenta 4 días.** `_cron_post_process`
(`payment/models/payment_transaction.py:849`, `PaymentTransaction._cron_post_process`) filtra `last_state_change >= now-4 días` (PayPal puede tardar tanto).
Pasado ese plazo, una tx `is_post_processed=False` no se vuelve a procesar nunca.

**PAYG8. `amount` se invalida tras create para arreglar el float.** `create`
(`payment/models/payment_transaction.py:189`, `PaymentTransaction.create`) hace `invalidate_recordset(['amount'])` porque `1111.11` puede quedar
`1111.1100000000001` en caché; se relee del DB con `float_repr`.

**PAYG9. El estado `authorized` exige que el provider soporte captura manual.** Constraint
`_check_state_authorized_supported` (`payment/models/payment_transaction.py:140`, `class PaymentTransaction`) lanza `ValidationError` si
`state=='authorized'` y `not provider_id.support_manual_capture`.

**PAYG10. Las acciones backend usan sudo deliberadamente.** `action_capture`/`void`/`refund`
(`payment/models/payment_transaction.py:246`, `PaymentTransaction.action_capture`/`274`/`289`) hacen `check_rights_on_recordset` y luego `.sudo()` para
`_send_*_request`; void exige `authorized`, refund exige `done`, o lanzan `ValidationError`.

**PAYG11. La transacción fuente se confirma sola cuando los hijos cubren el importe.**
`_update_source_transaction_state` (`payment/models/payment_transaction.py:825`, `PaymentTransaction._update_source_transaction_state`) suma hijos `done`/`cancel` de la
misma `operation`; si igualan `source.amount`, llama `_update_state(('authorized',),'done',...)`
directamente (no `_set_done`) para no entrar en bucle.

**PAYG12. Ningún `_send_*_request` funciona con el provider `disabled`.** Todos pasan por
`_ensure_provider_is_not_disabled` (`payment/models/payment_transaction.py:588`, `PaymentTransaction._ensure_provider_is_not_disabled`), que lanza `UserError`. Los refunds
niegan el importe (`amount = -amount`, `:618`) y usan prefijos de referencia `R-`/`P-`.

**PAYG13. No puedes borrar un provider con xmlid ni cambiarle la compañía si tiene transacciones.**
`_unlink_except_master_data` (`payment/models/payment_provider.py:402`, `class PaymentProvider`) bloquea unlink salvo xmlid `__export__`;
`_onchange_company_block_if_existing_transactions` (`:283`) lanza `UserError` si hay txs.

**PAYG14. Habilitar un payment method exige un provider habilitado que lo soporte.**
`payment_method.write` (`payment/models/payment_method.py:186`, `PaymentMethod.write`) lanza `UserError` ("needs a partner in crime") si
todos los providers de la primary method están `disabled`. Y solo se ofrecen métodos `is_primary`.

## Gotchas adicionales (provider + account_payment)

**PAYG25. Webhook sin secreto configurado se acepta SIN verificar firma.** Si `get_webhook_secret`
devuelve vacío, `_verify_notification_signature` (`payment_stripe/controllers/main.py`) solo loguea un
warning y procesa el evento. Sin secreto no hay validación HMAC.

**PAYG26. El webhook rechaza eventos de más de 10 min (anti-replay).** `WEBHOOK_AGE_TOLERANCE = 600`
(`payment_stripe/controllers/main.py`): reenvíos manuales tardíos del proveedor se descartan aunque
la firma sea válida.

**PAYG27. El reembolso de tx en `error` SÍ puede venir de `done` (excepción única a los estados).**
`_set_error` para `operation='refund'` se llama con `extra_allowed_states=('done',)`
(`payment_stripe/models/payment_transaction.py`). Es el único caso donde `done→error` está permitido.

**PAYG28. Un refund confirmado dispara el cron de post-proceso al instante.** Al recibir `done` con
`operation='refund'`, ejecuta `env.ref('payment.cron_post_process_payment_tx')._trigger()` porque
nadie navegará la tx de refund desde el portal; sin esto el `account.payment` del refund no se crearía
hasta el cron periódico.

**PAYG29. `requires_action` (SCA/3DS pendiente) mapea a `draft`, no a `pending`.**
`STATUS_MAPPING` (`payment_stripe/const.py`): una tx esperando autenticación del cliente queda `draft`.

**PAYG30. El reembolso parcial disponible se calcula sobre PAGOS confirmados, no sobre tx.**
`account.payment._compute_amount_available_for_refund` resta solo refunds con `account.payment`
confirmado; un refund colgado por webhook fallido evita doble reembolso.

**PAYG31. Los tokens de providers con captura manual quedan EXCLUIDOS del selector de pago.**
`account.payment._compute_suitable_payment_token_ids` filtra `capture_manually==False`: no se puede
cobrar offline por token con un provider de captura manual.

**PAYG32. Un pago por token que no llega a `done` se CANCELA, no queda pendiente.**
`account.payment.action_post`: tras `_send_payment_request`+`_post_process`, los pagos cuya tx no
quedó en `done` reciben `action_cancel()`.

**PAYG33. El partner del `account.payment` se fuerza al `commercial_partner_id`.** `_create_payment`
(`account_payment/models/payment_transaction.py`): si la tx la hizo un contacto hijo, el pago contable
cuelga de la empresa matriz.

**PAYG34. Stripe recibe el código del método PRIMARIO en el PaymentIntent, pero el BRAND en el
SetupIntent.** `_stripe_prepare_payment_intent_payload` usa `primary_payment_method_id.code` ("card");
`_stripe_prepare_setup_intent_payload` usa `payment_method_code` (el brand) — asimetría.

**PAYG35. El descuento por pronto pago (EPD) solo se aplica si el importe casa EXACTO.** `_create_payment`
solo añade el write-off de EPD si `installment_state=='epd'` y `self.amount == amount_due`; un pago por
importe distinto con EPD vigente no aplica el descuento.

**PAYG36. `_update_landing_route` concatena query params en crudo.** `payment/controllers/portal.py`
hace `f'{landing_route}?tx_id=…&access_token=…'`; si el `landing_route` ya tenía `?`, se rompe (doble
query string).

**PAYG37. El refund se localiza por `provider_reference` en `charge.refund.updated`.**
`_get_tx_from_notification_data` (`payment_stripe`): ese evento no trae merchant ref; si dos tx
comparten `provider_reference` o está vacío, el matching falla con `ValidationError`.
