<!-- verified: v19.0 @ 2026-06 (diff payment v18→v19 vs odoo19 source; anclas verificadas) -->
# Payment Engine — Cambios v19

Diff real `payment` v18→v19 (verificado contra `$ODOO_SRC/odoo19/addons/payment/`). **No hay cambios
de esquema disruptivos** (mismos 4 modelos, +3 campos), pero **sí un refactor profundo de la API de
procesamiento**: cualquier módulo de acquirer que herede de `payment` en v18 **rompe en v19**.

## Flujo de notificaciones reescrito (breaking)

El contrato `*_notification_data` de v18 desaparece y se sustituye por `*payment_data`:

| v18 (eliminado) | v19 (sustituto) |
|---|---|
| `_handle_notification_data` | `_process(provider_code, payment_data)` (`payment/models/payment_transaction.py:738`, `PaymentTransaction._set_canceled`, orquestador) |
| `_get_tx_from_notification_data` | `_search_by_reference` (`:759`) + `_extract_reference` (`:782`) |
| `_process_notification_data` | `_apply_updates` (`:858`) + `_extract_amount_data` (`:845`) + `_validate_amount` (`:794`) + `_tokenize` (`:876`) + `_extract_token_values` (`:902`) |

## Firmas de captura/void/refund cambiadas

Los `_send_*_request` **pierden el argumento de importe** y aparecen wrappers públicos nuevos:

- `_send_capture_request(self)` (`:611`) · wrapper `_capture(amount_to_capture=None)` (`:591`)
- `_send_void_request(self)` (`:643`) · wrapper `_void(amount_to_void=None)` (`:623`)
- `_send_refund_request(self)` (`:675`) · wrapper `_refund(amount_to_refund=None)` (`:655`)

`_set_pending/_set_authorized/_set_done` pasan `state_message` a **keyword-only**
(`def _set_done(self, *, state_message=None, ...)` `:949`). `_compute_reference_prefix` pierde el
primer parámetro `provider_code` (`:451`).

## Capa HTTP unificada (nuevo)

- `payment.transaction._send_api_request(method, endpoint, *, params, data, json, **kwargs)` (`:1124`)
  y `_charge_with_token` (`:561`), `action_post_process` (`:355`), `_build_action_feedback_notification` (`:330`).
- `payment.provider` gana toda una capa de cliente HTTP: `_send_api_request` (`:756`),
  `_build_request_url`/`_headers`/`_auth` (`:816`/`:828`/`:842`), `_log_request`/`_log_response`,
  `_parse_response_content`/`_error`, `_parse_proxy_response`. Nuevo fichero raíz `logging.py`.
- `_get_specific_secret_keys` (v18) **eliminado**.

## Campos nuevos

- `payment.transaction.primary_payment_method_id` (`:47`, compute `_compute_primary_payment_method_id` `:142`) y `is_live` ("Production Environment", `:90`).
- `payment.method.support_manual_capture` (`:72`) + constraint `_check_manual_capture_supported_by_providers` (`:178`).
- `payment.token.partner_id` pasa a `index=True`.

## Onboarding refactorizado

- Eliminado `models/onboarding_step.py` (con `action_validate_step_payment_provider`).
- Nuevos en `payment.provider`: `action_start_onboarding` (`:491`), `action_reset_credentials` (`:503`), `_get_reset_values`.
- `_get_removal_domain` → renombrado `_get_provider_domain` (`:1000`); `_setup_provider`/`_remove_provider` aceptan `**kwargs`.
- `const.py` ampliado: `SENSITIVE_KEYS` y `COUNTRY_NUMERIC_CODES`.

## Sin cambios

`state` y `operation` (selections) **idénticos** a v18 — el vocabulario de estados no cambia, solo
la mecánica de cómo se llega a ellos.
