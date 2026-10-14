<!-- verified: v18.0 @ 2026-06 (loyalty CE engine; anclas verificadas vs fuente) -->
# Loyalty & Promotions — Gotchas (Odoo 18.0 CE)

Comportamientos ocultos del motor. Núcleo en [`core.md`](core.md); flujos sale/web/POS en
[`integrations.md`](integrations.md) (LOYG15+).

**LOYG1. Cambiar `program_type` BORRA y recrea reglas/recompensas/comunicaciones.**
`_compute_from_program_type` (`loyalty/models/loyalty_program.py:386`, `class LoyaltyProgram`) hace `write` con `[(5,0,0),(0,0,{...})]`
sobre `rule_ids`/`reward_ids`/`communication_plan_ids`. Reconfigurar el tipo de un programa
existente **descarta** lo que tenías. (Usa el contexto `loyalty_skip_reward_check` para no romper el
constraint "al menos una recompensa" durante el recreado.)

**LOYG2. `is_nominative` decide gift_card/ewallet/loyalty vs promoción.** `_compute_is_nominative`
(`:196`): True si `applies_on='both'`, o `program_type in ('ewallet','loyalty')` con `applies_on='future'`.
Las nominativas se atan a un partner.

**LOYG3. `is_payment_program` distingue "pagar" de "descontar".** `_compute_is_payment_program`
(`:202`): True solo para `gift_card` y `ewallet`. Estos consumen saldo (pagan el pedido); el resto
descuenta.

**LOYG4. `reward_point_split` prohibido en loyalty/ewallet.** Constraint `_constraint_trigger_multi`
(`loyalty/models/loyalty_rule.py:82`, `class LoyaltyRule`) lanza `ValidationError` si `reward_point_split=True` con `applies_on='both'`
o `program_type='ewallet'`. El split solo aplica a programas `future` modo money/unit.

**LOYG5. El código de una regla no puede coincidir con el de ninguna tarjeta (y viceversa).**
`loyalty_rule.py::_constrains_code` (`:89`) + `loyalty_card.py::_contrains_code` (`:54`): unicidad
cruzada entre reglas `with_code` activas y `loyalty.card` activas. Programas y cupones no comparten código.

**LOYG6. Las tarjetas de fidelización NO pueden caducar.** `_restrict_expiration_on_loyalty`
(`loyalty/models/loyalty_card.py:65`, `class LoyaltyCard`) es un **`@api.onchange`** (solo salta en la UI; un `write` directo del campo
NO lo activa) que lanza `ValidationError` si pones `expiration_date` con `program_type='loyalty'`.

**LOYG7. Email automático al crear la tarjeta (si tiene partner).** `create` (`loyalty/models/loyalty_card.py:190`, `class LoyaltyCard`)
→ `_send_creation_communication` envía la plantilla del plan con `trigger='create'`, solo si hay
partner. Inhibible con contexto `loyalty_no_mail`. Si la plantilla no tiene `email_from`, se inyecta
el del autor para garantizar el envío.

**LOYG8. Email "al alcanzar X puntos": solo el hito MÁS ALTO cruzado.** `write` (`loyalty/models/loyalty_card.py:196`, `LoyaltyCard.write`)
→ `_send_points_reach_communication`: si una subida de puntos cruza varios hitos a la vez, envía
solo el de mayor target. No envía si los puntos bajan o no cambian.

**LOYG9. Cada recompensa crea un producto de descuento OCULTO.** `_create_missing_discount_line_products`
(`loyalty/models/loyalty_reward.py:267`, `LoyaltyReward._create_missing_discount_line_products`) crea un `product.product` `service`, `sale_ok=False`, `lst_price=0` por
reward (para imputar la línea de descuento). Archivar el reward lo propaga al producto.

**LOYG10. (Des)archivar el programa propaga a hijos.** `toggle_active` (`loyalty/models/loyalty_program.py:437`, `LoyaltyProgram.toggle_active`)
fuerza el mismo `active` en `rule_ids`/`reward_ids`/`communication_plan_ids` y el
`discount_line_product_id`. (⚠️ `toggle_active` **eliminado en v19** — ver `v19_changes.md`.)

**LOYG11. No se puede borrar un programa activo.** `_unlink_except_active` (`loyalty/models/loyalty_program.py:432`, `class LoyaltyProgram`)
lanza `UserError`; hay que archivarlo primero.

**LOYG12. `trigger_product_ids` se descarta salvo en gift_card/ewallet.** `create`
(`loyalty/models/loyalty_program.py:615`, `class LoyaltyProgram`) elimina ese related del vals si el tipo no es gift_card/ewallet, para no
pisar los `product_ids` de la regla.

**LOYG13. El cálculo de productos descontables se puede desactivar por parámetro.**
`_compute_all_discount_product_ids` (`loyalty/models/loyalty_reward.py:176`, `LoyaltyReward._compute_all_discount_product_ids`) respeta el config param
`loyalty.compute_all_discount_product_ids` (default `enabled`); en catálogos grandes se desactiva y
se delega en el dominio (rendimiento).

**LOYG14. Los datos demo de gift_card/ewallet pueden no existir.** `_program_type_default_values`
(`loyalty/models/loyalty_program.py:266`, `LoyaltyProgram._program_type_default_values`) referencia `loyalty.gift_card_product_50`/`ewallet_product_50` con
`raise_if_not_found=False`; sin demo instalado, las reglas se crean sin `product_ids`.

## Gotchas adicionales (pricing, divisa, aplicación)

**LOYG25. Programa y pricelist DEBEN compartir moneda.** `_check_pricelist_currency`
(`loyalty_program.py`) revienta si añades una tarifa de otra divisa; el dominio del campo ya filtra
`[('currency_id','=',currency_id)]` (no ves las de otra moneda).

**LOYG26. La divisa del programa la fija la COMPAÑÍA, no la eliges.** `_compute_currency_id`
(`loyalty_program.py`): `currency_id = company_id.currency_id`. Cambiar la compañía reescribe la
divisa de gift_card/ewallet en silencio (y la del `loyalty.card` por related).

**LOYG27. El descuento nunca supera el total actual del pedido.** `_get_reward_values_discount`
(`sale_order.py`) fuerza `max_discount = min(amount_total, max_discount)`. Un cupón de 50€ sobre un
pedido de 30€ descuenta 30€; el resto del saldo NO se guarda en ese pedido.

**LOYG28. `per_point` recalcula el coste real de puntos por divisa y redondeo.** `_get_reward_values_discount`:
`point_cost = round(converted_discount / reward.discount)`; con el descuento truncado por el total,
gastas MENOS puntos de los nominales.

**LOYG29. Los puntos `per_point` se truncan a múltiplos enteros de la recompensa.**
`points = points // required_points * required_points` ("no se ofrecen parcialmente") — salvo
`is_payment_program` (gift/ewallet), que sí usa saldo fraccionado. `sale_order.py::_get_reward_values_discount`.

**LOYG30. "Cheapest" mezcla base con-IVA y sin-IVA en dos acumuladores.** `_discountable_cheapest`
(`sale_order.py`): `discountable += price_total/qty` (con impuestos) pero `discountable_per_tax +=
price_unit*(1-discount)` (sin); con impuestos no incluidos el descuento real por línea no cuadra.

**LOYG31. El producto de recompensa NO puede ser `combo`.** Dominio `[('type','!=','combo')]` +
`@api.constrains` (`loyalty_reward.py`); además `_cheapest_line` ignora `combo_item_id`.

**LOYG32. Gift card recomputa el price_unit metiendo los impuestos price_include del producto.**
Solo `program_type=='gift_card'` (no ewallet): con `force_price_include=True`, el importe canjeado
cambia según la posición fiscal del cliente. `sale_order.py::_get_reward_values_discount`.

**LOYG33. La caducidad del cupón se mide contra la fecha del PAGO, no "hoy".**
`_get_confirmed_tx_create_date` (`sale_order.py`): con tx `done`/`authorized` usa `min(create_date)`;
un cupón caducado entre el pago y la confirmación sigue válido.

**LOYG34. El orden de aplicación de programas es por `sequence`, NO por mejor descuento.**
`loyalty.program._order='sequence'`; entre descuentos globales gana el MENOR
(`_best_global_discount_already_applied`, para gastar menos voucher), pero no optimiza el total.

**LOYG35. Cambiar `program_type` en write inyecta el tipo en el contexto para el `default_get` del
reward.** `loyalty_program.write` mete `program_type` en contexto (además de `loyalty_skip_reward_check`)
para que `loyalty.reward.default_get` no coja `reward_type='discount'` por defecto al recrear.

**LOYG36. gift/ewallet descuentan sobre el total CON impuestos fijos y envío.** `_discountable_order`:
para `is_payment_program` NO se restan las líneas neutras ni se filtran impuestos `fixed` (el resto de
programas sí). Mismo método, dos comportamientos.

**LOYG37. Un cupón `future` nominativo cambia quién puede canjear dónde.** `_get_claimable_rewards`
salta `applies_on='future'` cuyo `order_id==self`; `_apply_program_reward` da error "only on future
orders" solo si NO es nominativo.
