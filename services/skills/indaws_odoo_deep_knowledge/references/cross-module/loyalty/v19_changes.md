<!-- verified: v19.0 @ 2026-06 (diff loyalty+sale_loyalty v18→v19 vs odoo19 source; anclas verificadas) -->
# Loyalty & Promotions — Cambios v19

Diff real `loyalty` + `sale_loyalty` v18→v19 (verificado contra `$ODOO_SRC/odoo19/addons/`). **Sin
cambios de esquema disruptivos** (mismos 5 modelos, selections idénticas); el grueso del diff es
reformateo cosmético (campos a kwargs explícitos). Cambios accionables:

## loyalty (base)

| Cambio | Ancla v18 | v19 |
|--------|-----------|-----|
| `loyalty.reward.tax_ids` **eliminado** | `loyalty_reward.py::tax_ids` (`:89`) | ya no existe |
| `loyalty.card._get_mail_partner` **eliminado** | `loyalty_card.py::_get_mail_partner` (`:87`) | absorbido por el framework de mail (`_mail_get_partner_fields`) |
| `loyalty.program.toggle_active` **eliminado** | `loyalty_program.py::toggle_active` (`:437`) | no se sobreescribe (ver LOYG10) |
| `loyalty.history.card_id` → `index=True` | — | añadido |
| `loyalty.card.company_id` → `precompute=True` | — | añadido |

`program_type` (8 valores), `reward_type`, `discount_mode`, `reward_point_mode`,
`discount_applicability`, `applies_on`: **miembros idénticos**, sin cambios de valor.

## sale_loyalty

- **`sale.order.line._compute_tax_id` → `_compute_tax_ids`** (sigue el rename del core
  `sale.order.line` `tax_id`→`tax_ids` en v19).
- **`_is_not_sellable_line` eliminado**; nuevos `_can_be_edited_on_portal`, `_compute_discount`.
- Nuevos en `sale.order`: `gift_card_count` + `_compute_gift_card_count` + `action_view_gift_cards`.
- `loyalty.card`: v18 sobreescribía `_get_mail_partner`; v19 lo sustituye por el campo related
  `order_id_partner_id` + `_mail_get_partner_fields`.
- Los 4 métodos clave **siguen existiendo** con misma firma: `_apply_program_reward`,
  `_get_reward_values_discount`, `_update_programs_and_rewards`, `_program_check_compute_points`.

## Impacto

Módulos que extiendan `loyalty.reward` asumiendo `tax_ids`, o que llamen `toggle_active` /
`_get_mail_partner`, **rompen en v19**. El flujo de aplicación de programas (`_update_programs_and_rewards`
y compañía) se mantiene estable.
