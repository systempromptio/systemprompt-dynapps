<!-- verified: v18.0 base + v19.0 diffs @ 2026-06 (loyalty CE: program/rule/reward/card/history; anclas verificadas vs fuente v18 y v19) -->
# Loyalty & Promotions — Core Reference (Odoo 18.0 CE)

Motor único de **promociones, cupones, fidelización, tarjetas regalo y monedero** (`loyalty`, CE).
Lo aplican `sale_loyalty` (pedido), `website_sale_loyalty` (eCommerce), `pos_loyalty` (TPV) y
`sale_loyalty_delivery` (envío gratis). Un mismo motor para los 8 tipos de programa.

## Cuatro modelos

| Modelo | Rol |
|--------|-----|
| `loyalty.program` | El programa: tipo, validez, disparo, reglas y recompensas. |
| `loyalty.rule` | Cómo se gana / qué dispara (mínimos, productos, puntos, código). |
| `loyalty.reward` | Qué se obtiene (descuento o producto, aplicabilidad, puntos requeridos). |
| `loyalty.card` | El cupón/tarjeta con su saldo de `points` y código único. `loyalty.history` = movimientos. |

## loyalty.program — campos clave

| Campo | Notas (ancla `loyalty_program.py`) |
|-------|------------------------------------|
| `program_type` (`:58`) | **8 valores**: `promotion`, `coupons`, `loyalty` (puntos), `gift_card`, `ewallet`, `promo_code` ("Discount Code"), `buy_x_get_y`, `next_order_coupons`. Default `promotion`. |
| `applies_on` (`:83`) | `current` / `future` / `both` (computado de `program_type`, editable). |
| `trigger` (`:89`) | `auto` / `with_code`. |
| `is_nominative` (`:196`) · `is_payment_program` (`:202`) | computados: nominative = atado a partner; payment = `gift_card`/`ewallet` (consumen saldo). |
| `pricelist_ids` (`:34`) · `date_from`/`date_to` (`:69`) · `limit_usage`/`max_usage` (`:77`) | restricciones de tarifa, fechas y nº de usos. |

## loyalty.rule — cómo se gana

`reward_point_mode` (`loyalty/models/loyalty_rule.py:59`, `class LoyaltyRule`): `order`/`money`/`unit` + `reward_point_amount`.
Condiciones: `minimum_qty`, `minimum_amount` (+`minimum_amount_tax_mode` incl/excl),
`product_ids`/`product_category_id`/`product_domain` (`_get_valid_product_domain`, `:127`).
`mode` `auto`/`with_code` + `code` (mutuamente computados).

## loyalty.reward — qué se obtiene

`reward_type` (`loyalty/models/loyalty_reward.py:63`, `class LoyaltyReward`): `discount` o `product`.
- Descuento: `discount` + `discount_mode` (`percent`/`per_order`/`per_point`) + `discount_applicability`
  (`order`/`cheapest`/`specific`).
- Producto: `reward_product_id` (dominio `type != 'combo'`), `required_points`, `clear_wallet`.
- **Envío gratis** (`shipping`) NO está en base: lo añade `sale_loyalty_delivery` (ver [[delivery]]).

## loyalty.card — el cupón/tarjeta

`points` (`tracking=True`), `code` (default `_generate_code` "044"+uuid, UNIQUE), `partner_id`
(nominativa si no vacío), `expiration_date`. `loyalty.history` registra `issued`/`used` con
referencia polimórfica al pedido. Email automático al crear / al alcanzar puntos (`loyalty_mail`).

Trampas del motor en [`gotchas.md`](gotchas.md) (LOYG1-14); flujos sale/web/POS en
[`integrations.md`](integrations.md) (LOYG15+); capacidades en [`capabilities.md`](capabilities.md);
cambios v19 en [`v19_changes.md`](v19_changes.md). Pago de gift_card/ewallet → [[payment]].
