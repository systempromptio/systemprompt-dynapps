<!-- verified: v18.0 @ 2026-06 (loyalty ↔ sale_loyalty/website_sale_loyalty/pos_loyalty/sale_loyalty_delivery; anclas verificadas vs fuente) -->
# Loyalty & Promotions — Integrations (Odoo 18.0 CE)

Cómo el motor (`loyalty`) se aplica en pedidos, eCommerce y TPV. Núcleo en [`core.md`](core.md);
gotchas del motor en [`gotchas.md`](gotchas.md) (LOYG1-14).

## Aplicación al pedido (sale_loyalty)

1. **Orquestador**: `sale_order.py::_update_programs_and_rewards` — 5 pasos (programas aplicables →
   actualizar aplicados → reconstruir líneas de recompensa reusando el pool vía `_reset_loyalty()` →
   auto-aplicar → cleanup). Todo cambio del pedido (pricelist, qty, cupón) pasa por aquí.
2. **Confirmación**: `sale_order.py::action_confirm` valida que ningún cupón quede con puntos
   negativos, crea `loyalty.history`, materializa `coupon.points += change` y envía
   `_send_reward_coupon_mail()`. **Los puntos solo se mueven de verdad al confirmar.**
3. **Líneas de recompensa**: `sale.order.line` marca `is_reward_line`, `reward_id`/`coupon_id`
   (`ondelete='restrict'`), `points_cost`, `reward_identifier_code` (agrupa varias líneas de la misma
   recompensa, p.ej. descuento partido por impuesto).
4. **Descuento**: `_get_reward_values_discount` genera **una línea por grupo de impuesto** (`price_unit`
   negativo, `tax_id` mapeado por `fiscal_position_id`); el `points_cost` se asigna a UNA sola línea.
   gift_card/ewallet (`is_payment_program`) → una única línea de pago.
5. **Código en el SO**: `_try_apply_code` resuelve contra `loyalty.rule` (with_code) o `loyalty.card.code`;
   bloquea con `FOR UPDATE NOWAIT`, rechaza loyalty/ewallet por código. Wizard manual:
   `sale_loyalty_coupon_wizard.py::action_apply`.
6. **Puente de puntos pre-confirmación**: `sale.order.coupon.points` (UNIQUE order_id+coupon_id,
   `ondelete='cascade'`) registra puntos a emitir antes de confirmar.

## eCommerce (website_sale_loyalty)

- Código promo en checkout: `controllers/main.py::pricelist` (`/shop/pricelist`, param `promo`) →
  `order._try_apply_code(promo)`; `cart`/`shop_payment` llaman `_update_programs_and_rewards` +
  `_auto_apply_rewards`. `/coupon/<code>` guarda `pending_coupon_code` en sesión.
- Dominios web: `_get_program_domain`/`_get_trigger_domain` sustituyen `sale_ok`→`ecommerce_ok` y
  añaden filtro `website_id`.

## TPV (pos_loyalty)

- Carga vía `pos.load.mixin`: `loyalty.program._load_pos_data` (filtra por `pos.config`),
  `loyalty.reward._load_pos_data`. Programa sin POS asignado → aplica a todos.
- Confirmación: `pos.order::confirm_coupon_programs` crea las `loyalty.card` ganadas, actualiza gift
  cards vendidas, mapea ids negativos del front a reales y suma puntos.

## Envío gratis (sale_loyalty_delivery)

`reward_type='shipping'` (`selection_add`); línea generada en `_get_reward_values_free_shipping`
(`price_unit = -min(discount_max_amount, delivery_line.price_unit)` sobre la 1ª línea `is_delivery`).

## Gotchas cross-module

**LOYG15. Las líneas de recompensa NO se copian.** `sale_order.py::copy` hace `unlink()` de las
líneas `is_reward_line`; las promos se recalculan en el pedido nuevo, no se arrastran.

**LOYG16. `ondelete='restrict'` en `reward_id`/`coupon_id`.** No puedes borrar un reward/card
referenciado por una línea; `pos_loyalty` archiva en vez de borrar si hay `pos.order.line` que lo usa.

**LOYG17. Los puntos solo se mueven en estado `sale`.** En `sale_order_line.py` `create`/`write`,
`coupon.points -= points_cost` solo si `line.state=='sale'`. En borrador todo es virtual
(`coupon_point_ids`). Cambiar `points_cost` en un pedido confirmado mueve puntos al instante.

**LOYG18. Cancelar revierte puntos e historial pero NO regenera cupones.** `_action_cancel` borra la
`loyalty.history` del pedido, resta puntos, elimina líneas de recompensa y borra las cards no
nominativas sin `use_count`. Los cupones "next order" perdidos **no se regeneran** solos.

**LOYG19. Un cupón "future" no se canjea en el pedido que lo genera.** `_get_claimable_rewards` salta
los cupones `applies_on='future'` cuyo `order_id == self`.

**LOYG20. `_get_no_effect_on_threshold_lines` es el punto de extensión de líneas neutras.** Vacío en
sale_loyalty, sobreescrito en sale_loyalty_delivery para excluir el envío. Si añades un tipo de línea
que no debe contar para el mínimo ni recibir descuento y olvidas extenderlo, el envío cuenta para
alcanzar mínimos o recibe descuento.

**LOYG21. gift_card/ewallet se aplican sobre el total CON impuestos y envío.** `_discountable_order`:
los programas de pago incluyen impuestos fijos y líneas de envío (los descuentos normales los
excluyen) y se ordenan al final (pagos tras descuentos).

**LOYG22. Programas nominativos solo para usuarios logueados en web.** `_allow_nominative_programs`
devuelve False si el usuario del website es público; loyalty/ewallet no se auto-cargan para anónimos.

**LOYG23. Des-reclamar una recompensa web la mete en `disabled_auto_rewards`.** Poner qty 0 en una
línea de recompensa (contexto `website_sale_loyalty_delete`) impide que `_auto_apply_rewards` la
vuelva a aplicar en ese pedido.

**LOYG24. Solo una recompensa de envío gratis simultánea.** `sale_loyalty_delivery::_get_claimable_rewards`
filtra el resto si ya hay una `shipping` aplicada; el coste se recalcula con `rate_shipment` si había
`free_over`. Ver [[delivery]].
