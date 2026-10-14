# Module Inference: Odoo Signals → Module Names

Use this table when database.env module data is unavailable. Cross-reference task names, ticket subjects, and sale order line descriptions.

## Confirmed = Hecho/Realizado

Tasks marked "Hecho" (Done) or "Realizado" (Completed) in project stage = module confirmed implemented.

## Signal → Module Mapping

| Signal in Odoo | Module | Notes |
|----------------|--------|-------|
| Task "Fabricación" Hecho | MRP (Manufacturing) | mrp |
| Task "Punto de venta" Hecho | Point of Sale | point_of_sale |
| Task "CRM" Hecho | CRM | crm |
| Task "Ventas" Hecho | Sales | sale_management |
| Task "Compras" Hecho | Purchase | purchase |
| Task "Inventario" Hecho | Inventory | stock |
| Task "Contabilidad" Hecho | Accounting | account_accountant |
| Task "Facturación" Hecho | Invoicing | account |
| Task "Proyectos" Hecho | Project | project |
| Task "RRHH" or "Empleados" Hecho | HR | hr |
| Task "Gastos" or ticket "Gastos" resolved | Expenses | hr_expense |
| Ticket "FACE" or "FACe" | Electronic invoicing (Spain) | l10n_es_facturae / l10n_es_face |
| Ticket "Facturae" | Facturae format | l10n_es_facturae |
| Sale line "Integración con MRW" | MRW shipping | delivery_mrw (OCA) |
| Task "sendcloud" or "SendCloud" in production | SendCloud shipping | sendcloud |
| Task "WhatsApp" En producción | WhatsApp Business | various |
| Ticket "código de barras" or "Barcode" | Barcode scanner | stock_barcodes |
| Ticket "comisiones" or "Sales Commission" | Sales commissions | sale_commission (OCA) |
| Sale line "Docky" | Indaws document AI | indaws_docky |
| Sale line "Chatty" | Indaws sales chatbot | indaws_chatty |
| Sale line "INDAWS ACADEMY" | Training platform | indaws_academy |
| Task "VOIP" or "Voip" | VoIP integration | various |
| Ticket "Stripe" | Stripe payment gateway | payment_stripe |
| Ticket "pasarela de pago" | Payment gateway (generic) | payment_* |
| Ticket "Outlook" + "Calendar" | Calendar sync | microsoft_calendar |
| Ticket "DUA" | Customs/import duties | l10n_es_dua |
| Task "Importación LDM" | Bill of Materials import | mrp |
| Ticket "límite de crédito" | Credit limit | account_credit_control |
| Ticket "numeración de documentos" | Custom sequences | l10n_es_account_sequence |
| Task "Barcode" Hecho | Barcode | stock_barcodes |
| Ticket "correos masivos" | Email marketing | mass_mailing |
| Task "Configuración correos masivos" | Mass mailing config | mass_mailing |

## Indaws Core indicators

Sale order lines with "INDAWS ACADEMY", "Docky", "Chatty", "Cuota servidor Indaws Cloud" indicate Indaws-proprietary modules. These will show as `indaws_*` in database.env if the customer is on an Indaws-managed instance.

## Note on OCA modules

Modules like `sale_commission`, `delivery_mrw`, `partner_credit_limit` are OCA (Odoo Community Association) modules — open source but not standard Odoo. Mention them as "OCA extensions" in the case study rather than "Odoo standard modules".
