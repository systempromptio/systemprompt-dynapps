<!-- verified: v18.0 @ 2026-06 (payment CE + providers payment_<x>; fees/POS negativos; anclas verificadas vs fuente) -->
# Payment (`payment`) — Capacidades (¿Odoo puede…?)

> Eje **consultor**, anclado a fuente. **CE** (providers integrados). Método: [`../../_index/HOW-TO-EXTEND.md`](../../_index/HOW-TO-EXTEND.md). Mecánica en [`core.md`](core.md); trampas en [`gotchas.md`](gotchas.md) (PAYG*).

**PAYCAP1. ¿Qué pasarelas trae de serie?** — **NATIVO** · baja · módulos `payment_<x>` en CE18: adyen, aps, asiapay, authorize, buckaroo, custom, demo, flutterwave, mercado_pago, mollie, nuvei, paypal, razorpay (+oauth), stripe, worldline, xendit; EE añade `payment_sepa_direct_debit`.

**PAYCAP2. ¿Guardar tarjeta / tokenización?** — **NATIVO** · baja · `payment.provider.allow_tokenization` + `support_tokenization` + modelo `payment.token`. Para los providers que lo declaran.

**PAYCAP3. ¿Cobro recurrente automático con token (S2S)?** — **NATIVO** · media · `payment_transaction.py::_send_payment_request` (off-session; `payment_stripe` usa `off_session=True`). El motor de recurrencia vive en `sale_subscription` (EE). Ver [[subscription]].

**PAYCAP4. ¿Autorizar y capturar después (captura manual)?** — **NATIVO** · media · `payment.provider.capture_manually` + `payment_transaction.py::action_capture`/`_send_capture_request`. Solo providers con `support_manual_capture`.

**PAYCAP5. ¿Reembolsar desde Odoo?** — **NATIVO** · media · `payment.provider.support_refund` + `payment_transaction.py::_send_refund_request` (`action_refund`). En providers que lo declaran.

**PAYCAP6. ¿Reembolso parcial?** — **NATIVO** · media · `payment/models/payment_provider.py:169`, `class PaymentProvider` `support_refund` valor `'partial'` ("Full & Partial"). El parcial crea una tx hija refund.

**PAYCAP7. ¿Modo test/sandbox por proveedor?** — **NATIVO** · baja · `payment.provider.state='test'`. Procesa un pago falso por la interfaz de prueba.

**PAYCAP8. ¿Límite de importe / restricción por moneda o país?** — **NATIVO** · baja · `payment.provider.maximum_amount` / `available_currency_ids` / `available_country_ids` (filtrado en `_get_compatible_providers`). Ojo: el importe se compara en moneda de la compañía (PAYG5).

**PAYCAP9. ¿Pago manual/offline con instrucciones (transferencia)?** — **NATIVO** · baja · módulo `payment_custom` (provider con instrucciones de pago manual). Cliente recibe instrucciones, pago se confirma a mano.

**PAYCAP10. ¿Enviar enlace de pago desde backend?** — **NATIVO** · baja · `wizards/payment_link_wizard.py::_prepare_access_token` (`generate_access_token`). Genera un link de pago para el cliente.

**PAYCAP11. ¿Express checkout (Apple Pay / Google Pay)?** — **NATIVO** · media · `payment.provider.support_express_checkout`/`allow_express_checkout` (p.ej. `payment_stripe`: `support_express_checkout=True` + `express_checkout_form.js`).

**PAYCAP12. ¿3DS / SCA?** — **NATIVO (delegado al provider)** · media · gestionado por el flujo de redirección/confirmación de cada provider (`payment_stripe`/`payment_adyen` controllers + `off_session`/`setup_future_usage`); **no** hay campo genérico `support_3ds` en el core.

**PAYCAP13. ¿Comisiones del proveedor (fees) reflejadas?** — **NO** · — · `grep fee/commission` en `payment/models/*.py` = 0. El motor no modela ni contabiliza comisiones del acquirer.

**PAYCAP14. ¿POS y eCommerce comparten este motor?** — **NO** · — · eCommerce/web usa `payment.provider`+`payment.transaction`; POS usa `pos.payment.method` con terminal (`use_payment_terminal`). Solo el puente `pos_online_payment` reusa este motor. Ver [[pos]].

**PAYCAP15. ¿Anticipo / down payment en el pedido (cobrar % antes)?** — **NATIVO(CE)** · baja · wizard `sale.advance.payment.inv.advance_payment_method` (`sale/wizard/sale_make_invoice_advance.py:13`, `SaleAdvancePaymentInv.advance_payment_method`, % o importe fijo) + `deduct_down_payments` al facturar el resto. En web/checkout se cobra online contra el `payment.transaction`.
**PAYCAP16. ¿Domiciliación SEPA / recibo domiciliado (direct debit)?** — **EE u OCA** · media · `account_sepa_direct_debit` (EE, OEEL-1) o `account_banking_sepa_direct_debit` (OCA, `bank-payment`); gestionan mandato + fichero SEPA. NO existe en CE.
**PAYCAP17. ¿Conciliación automática del pago con la factura?** — **NATIVO(CE)** · baja · `payment.transaction._create_payment` (`account_payment/models/payment_transaction.py:135`, `PaymentTransaction._create_payment`): su docstring garantiza que "if the transaction is linked to some invoices, their reconciliation is done automatically". Crea el `account.payment` y concilia.
**PAYCAP18. ¿Multi-adquirente por compañía (multi-empresa)?** — **NATIVO(CE)** · baja · `payment.provider.company_id` (`payment/models/payment_provider.py:45`, `PaymentProvider.company_id`, indexado para record rules): cada pasarela pertenece a una compañía y se filtra por multi-company.
**PAYCAP19. ¿Reintento de pago fallido (dunning) con token/suscripción?** — **EE** · media · `sale.order._create_recurring_invoice` (`sale_subscription/models/sale_order.py:1555`, `SaleOrder._create_recurring_invoice`) factura y cobra recurrentemente con token, con lógica de reintento ante fallo (`:1415`). El motor `payment` base NO reintenta solo; requiere `sale_subscription`.
**PAYCAP20. ¿El cliente paga varias facturas juntas desde el portal?** — **NATIVO(CE)** · media · `account_payment/controllers/portal.py::portal_my_overdue_invoices` (`:54`) suma el `amount_residual` de todas las facturas vencidas (`:84`) y lanza una sola transacción por el total.
