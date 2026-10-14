<!-- verified: v18.0 @ 2026-06 (loyalty CE + sale/website/pos/delivery bridges; anclas verificadas vs fuente) -->
# Loyalty & Promotions (`loyalty`) — Capacidades (¿Odoo puede…?)

> Eje **consultor**, anclado a fuente. **CE**. Método: [`../../_index/HOW-TO-EXTEND.md`](../../_index/HOW-TO-EXTEND.md). Mecánica en [`core.md`](core.md); trampas en [`gotchas.md`](gotchas.md) (LOYG*).

**LOYCAP1. ¿Qué tipos de programa hay?** — **NATIVO** · baja · `loyalty.program.program_type` (8): `promotion`, `coupons`, `loyalty` (puntos), `gift_card`, `ewallet`, `promo_code` (Discount Code), `buy_x_get_y`, `next_order_coupons`.

**LOYCAP2. ¿Descuentos % o fijo, sobre pedido / productos / el más barato?** — **NATIVO** · baja · `loyalty.reward.discount_mode` (`percent`/`per_order`/`per_point`) + `discount_applicability` (`order`/`cheapest`/`specific`).

**LOYCAP3. ¿Compra X llévate Y?** — **NATIVO** · baja · `loyalty.program.program_type='buy_x_get_y'`. Regla de producto disparador + recompensa de producto.

**LOYCAP4. ¿Cupones con código (uso único/multiuso) y generación masiva?** — **NATIVO** · baja · `wizard/loyalty_generate_wizard.py::generate_coupons` + `loyalty.program.limit_usage`/`max_usage`. Generación por cantidad/cliente.

**LOYCAP5. ¿Tarjetas regalo (compra y canje)?** — **NATIVO** · baja · `program_type='gift_card'` (`is_payment_program`); canje en `sale_loyalty/models/sale_order.py` (líneas de pago). La compra usa un producto gift card.

**LOYCAP6. ¿Monedero / ewallet (saldo del cliente)?** — **NATIVO** · baja · `program_type='ewallet'` (`is_payment_program`), saldo en `loyalty.card.points`.

**LOYCAP7. ¿Puntos por dinero/unidad/pedido y canje por recompensas?** — **NATIVO** · media · `loyalty.rule.reward_point_mode` (`money`/`unit`/`order`) + `reward_point_amount`; canje vía `loyalty.reward.required_points`.

**LOYCAP8. ¿Caducidad de puntos/cupones?** — **NATIVO** · baja · `loyalty.card.expiration_date` (+ `program.date_from`/`date_to`). Ojo: las tarjetas `loyalty` no admiten caducidad (LOYG6).

**LOYCAP9. ¿Restricción por producto, importe mínimo, fechas, pricelist, web?** — **NATIVO** · media · `loyalty.rule` (`product_ids`/`minimum_amount`/`minimum_qty`) + `loyalty.program` (`date_from`/`date_to`/`pricelist_ids`). El filtro por **web concreta** (`website_id`) lo añade `website_sale_loyalty`.

**LOYCAP10. ¿Solo con código vs automática?** — **NATIVO** · baja · `loyalty.program.trigger` (`auto`/`with_code`) + `loyalty.rule.mode`.

**LOYCAP11. ¿Recompensa de envío gratis?** — **CONFIG** (módulo `sale_loyalty_delivery`) · baja · `reward_type` `selection_add=[('shipping',…)]`. NO está en `loyalty` base. Ver [[delivery]].

**LOYCAP12. ¿Aplicar en eCommerce y POS?** — **NATIVO** (vía puentes) · baja · web: `website_sale_loyalty/controllers/main.py` (`_try_apply_code`); POS: `pos_loyalty/models/pos_order.py::validate_coupon_programs`.

**LOYCAP13. ¿Cupones de siguiente pedido?** — **NATIVO** · baja · `program_type='next_order_coupons'` (`applies_on='future'`). Genera un cupón para un pedido futuro.

**LOYCAP14. ¿Combinar varios programas en un mismo pedido?** — **NATIVO** · media · `sale_loyalty/models/sale_order.py::_get_applicable_program_points` / `_get_applied_programs`. El pedido evalúa y aplica todos los programas aplicables a la vez; no hay límite a uno solo.

**LOYCAP15. ¿Programa de referidos ("trae un amigo")?** — **NO (nativo)** · alta · `loyalty.program.program_type` (`loyalty/models/loyalty_program.py:58`, `class LoyaltyProgram`) no incluye tipo `referral`; el único `*referral*` en fuente es `hr_referral` (RRHH/reclutamiento EE), NO clientes. Camino = desarrollo, o apaño con `next_order_coupons` emitidos a mano.
**LOYCAP16. ¿Niveles/tiers de fidelidad (Silver/Gold VIP)?** — **NO (nativo)** · alta · no hay campo `tier`/`level` en `loyalty` (0 refs). La fidelidad acumula puntos pero no gradúa VIP automáticamente. Camino = desarrollo, o simular con varios programas + reglas por segmento.
**LOYCAP17. ¿El cliente ve su saldo de puntos (portal)?** — **NATIVO(CE)** · baja · controller `loyalty/controllers/portal.py` + `loyalty.card.points_display`/`_compute_points_display` (`loyalty/models/loyalty_card.py:37`, `LoyaltyCard.points_display`, y `loyalty/models/loyalty_card.py:61-63`, `LoyaltyCard._compute_points_display`); las tarjetas del cliente se listan en su portal.
**LOYCAP18. ¿Código de barras/QR en cupones y tarjetas (escaneo en POS)?** — **NATIVO(CE)** · baja · `loyalty.card.code` autogenerado por `_generate_code` (`loyalty/models/loyalty_card.py:18`, `LoyaltyCard._generate_code` y `loyalty/models/loyalty_card.py:39`, `LoyaltyCard.code`); el POS escanea ese código para aplicar el cupón/tarjeta. No hay imagen QR embebida por defecto, es el código escaneable.
**LOYCAP19. ¿Cupón/tarjeta nominativo (ligado a un cliente concreto)?** — **NATIVO(CE)** · baja · `loyalty.card.partner_id` (`loyalty/models/loyalty_card.py:34`, `class LoyaltyCard`): la tarjeta puede ir a nombre de un partner o quedar anónima (campo vacío) para cupones al portador.
**LOYCAP20. ¿Reglas por importe mínimo o cantidad mínima de producto?** — **NATIVO(CE)** · baja · `loyalty.rule.minimum_qty` + `minimum_amount` (`loyalty/models/loyalty_rule.py:61`, `LoyaltyRule.minimum_qty`, y `loyalty/models/loyalty_rule.py:62`, `LoyaltyRule.minimum_amount`) con `minimum_amount_tax_mode` (con/sin impuestos); condiciona cuándo se activa la regla.
