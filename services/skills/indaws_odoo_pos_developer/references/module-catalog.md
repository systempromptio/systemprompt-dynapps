# Catálogo de módulos POS oficiales (Odoo 18 CE + EE)

Mapa de los **~106 módulos POS oficiales de v18** (63 CE + 43 EE, tras descartar
falsos positivos por substring) como **ejemplares de extensión**: cuando quieras
construir X, aquí tienes de qué módulo copiar el patrón. Todo verificado contra
los `__manifest__.py` de la fuente (`$ODOO_SRC/odoo18/addons` y
`$ODOO_ENTERPRISE/18`). Los `summary` son textuales del manifest.

> **Cómo leerlo.** Elige la fila que hace lo más parecido a tu objetivo, abre ESE
> módulo en la fuente y copia su estructura (`static/src/**`, overrides Python,
> `__manifest__.py`). Los `depends` te dicen sobre qué se apila; las "superficies"
> son una **detección heurística por grep** (qué tipo de extensión toca), útil para
> filtrar candidatos, NO un contrato — confírmalo abriendo el código.
>
> Superficies: `data` = `_load_pos_data*` · `store` = `patch(PosStore)` ·
> `screen` = `pos_screens`/`pos_pages` · `payment` = `register_payment_method` ·
> `js_model` = modelo JS nuevo (`pos_available_models`) · `receipt` = recibo ·
> `button` = ControlButtons · `self` = self-order · `owl` = `patch()` de componente.

---

## 1. Mapa "quiero hacer X → estudia este módulo"

| Objetivo | Módulo ejemplar | Ed | Por qué |
|----------|-----------------|----|---------|
| Botón + acción simple en pantalla de pedido | **pos_discount** | CE | El más pequeño con `button`+`data`+`owl`; plantilla mínima para "añado un botón que hace algo". |
| Cupones / tarjetas regalo / fidelización + **modelo JS nuevo** | **pos_loyalty** | CE | Ejemplar canónico de `js_model` (registra modelo en Python+JS), `button`, recibo y reactividad en el store. |
| Restaurante (mesas, plano, cuenta, split, propina) | **pos_restaurant** | CE | El bridge más rico: `js_model`, `screen`, `store`, floors/tables. Base de casi todo lo "restaurant". |
| Kiosco / carta en móvil del cliente (**2ª app OWL**) | **pos_self_order** | CE | Otra app OWL con su bundle/router propio; patrón de referencia para self-order. Ver GOTCHA 13. |
| Terminal de pago (patrón canónico mínimo) | **pos_razorpay** / **pos_mercado_pago** | CE | `register_payment_method` + `PaymentInterface` sin ruido extra. Ver GOTCHAS 9-10. |
| Terminal de pago + servicios/estado en el store | **pos_stripe** / **pos_viva_wallet** / **pos_tyro** | CE/EE | Añaden `store` y `data`; útil cuando el terminal necesita config del servidor. |
| Propina "estilo americano" sobre un terminal | **pos_restaurant_adyen** / **pos_restaurant_stripe** | CE | Patrón de **bridge de 2 features** (`pos_<terminal>` + `pos_restaurant`). |
| Pago online / QR (sin terminal físico) | **pos_online_payment** | CE | `payment` vía `account_payment`; flujo QR/online. |
| Cobrar pago online en el self-order | **pos_online_payment_self_order** | CE | Combina online payment + self-order. |
| Pantalla de cocina / preparación | **pos_preparation_display** | EE | App de display de comandas; base de todo "preparation". |
| Pantalla de estado de pedido para el cliente | **pos_order_tracking_display** | EE | Display de seguimiento apoyado en preparation + self-order. |
| Enlazar POS con **Ventas** (pedidos, entregas) | **pos_sale** | CE | Bridge de referencia POS↔sale; `button`, `data`, recibo. |
| Login de cajero / empleados en POS | **pos_hr** | CE | Bridge POS↔HR; `screen` de selección de empleado, `store`. |
| Impuestos personalizados (fórmula Python) | **pos_account_tax_python** | CE | `data`+`receipt`; cómo inyectar cálculo fiscal propio. |
| Recibo por SMS | **pos_sms** | CE | `owl`+`receipt`; enviar recibo por SMS. |
| Recibo / notificación por WhatsApp | **whatsapp_pos** | EE | Igual patrón sobre `whatsapp`. |
| Dispositivos IoT (balanza, cajón, impresora IoT) | **pos_iot** | EE | `payment`, `screen`, `store`, `data`; puente IoT. |
| Impresora Epson ePOS / ESC-POS | **pos_epson_printer** / `hw_escpos` | CE | Impresión directa a impresora de red. |
| Alta de producto escaneando con cámara | **pos_barcodelookup** | EE | `owl`; crear producto desde código de barras. |
| Etiquetas electrónicas de estantería | **pos_pricer** | EE | Sync de precios/info a tags Pricer. |
| Saldar deuda del cliente en el POS | **pos_settle_due** | EE | `button`, `screen`, `store`, `data`; flujo de cobro de pendientes. |
| Vender entradas de evento en el POS | **pos_event** (+ **pos_event_sale**) | CE | Bridge POS↔event; `store`, recibo. |
| Reservas de mesa (citas) | **pos_restaurant_appointment** | EE | `screen`, `store`; reservas online de mesas. |
| Suscripciones / alquiler desde POS | **pos_sale_subscription** / **pos_sale_stock_renting** | EE | Extienden `pos_sale`. |
| Integración con delivery (UberEats, Zomato…) | **pos_urban_piper** (+ `_ubereats`/`_zomato`/`_swiggy`) | EE | `data`, `store`, `screen`; agregador de pedidos externos. |
| App móvil nativa del POS | **pos_mobile** | EE | Wrapper móvil sobre `pos_enterprise`. |
| Dashboards de spreadsheet | **spreadsheet_dashboard_pos_hr** / `_restaurant` | CE | Datos POS en dashboards. |
| Fiscalización / EDI por país | grupo **l10n_*_pos** (§4) | CE/EE | Certificación de caja registradora y factura electrónica por país. |

---

## 2. Núcleo y capa base

| Módulo | Ed | Summary | Superficies |
|--------|----|---------|-------------|
| **point_of_sale** | CE | User-friendly PoS interface for shops and restaurants | data, store, screen, payment, js_model, receipt, button, self, owl (TODAS) |
| pos_enterprise | EE | Advanced features for PoS (`web_enterprise`) | — (bridge fino) |
| pos_mobile | EE | Odoo Mobile Point of Sale module | owl |
| hw_drivers | CE | Connect the Web Client to Hardware Peripherals | — |
| hw_escpos | CE | Hardware Driver for ESC/POS Printers and Cashdrawers | — |
| hw_posbox_homepage | CE | A homepage for the IoT Box | — |

> `point_of_sale` es la fuente primaria de verdad para TODO patrón; los demás son
> variaciones. Ante cualquier duda de firma/registry, abre primero el core.

## 3. Extensiones funcionales (no fiscales)

### Promociones y ventas cruzadas
| Módulo | Ed | Summary | Sup. |
|--------|----|---------|------|
| pos_discount | CE | Simple Discounts in the Point of Sale | button, data, owl |
| pos_loyalty | CE | Use Coupons, Gift Cards and Loyalty programs in POS | data, store, js_model, button, receipt |
| pos_sale_loyalty | CE | Link module between pos_sale and pos_loyalty | owl, receipt |
| pos_restaurant_loyalty | CE | Link pos_restaurant ↔ pos_loyalty | store, owl |

### Ventas / MRP / Eventos / Suscripción
| Módulo | Ed | Summary | Sup. |
|--------|----|---------|------|
| pos_sale | CE | Link module between Point of Sale and Sales | data, store, button, receipt |
| pos_sale_margin | CE | Link module POS ↔ Sales Margin | — |
| pos_event | CE | Link module between Point of Sale and Event | data, store, receipt |
| pos_event_sale | CE | Link between pos_sale and pos_event | — |
| pos_mrp | CE | Link module between Point of Sale and Mrp | receipt |
| pos_sale_stock_renting | EE | Link between PoS and Stock Rental | receipt |
| pos_sale_subscription | EE | Link between PoS and Sale Subscription | — |

### Recursos humanos (cajeros)
| Módulo | Ed | Summary | Sup. |
|--------|----|---------|------|
| pos_hr | CE | Link module between Point of Sale and HR | data, store, screen, receipt |
| pos_hr_restaurant | CE | Link pos_hr ↔ pos_restaurant | store, owl |
| pos_hr_mobile | EE | POS Barcode scan in Mobile | owl |

### Restaurante
| Módulo | Ed | Summary | Sup. |
|--------|----|---------|------|
| pos_restaurant | CE | Restaurant extensions for the Point of Sale | data, store, screen, js_model, payment, button, receipt |
| pos_restaurant_appointment | EE | Online reservations for restaurant tables | data, store, screen |
| pos_restaurant_adyen | CE | American-style tipping on Adyen | data, owl |
| pos_restaurant_stripe | CE | American-style tipping on Stripe | owl |
| pos_restaurant_preparation_display | EE | Display Orders for Preparation stage | button, receipt, owl |
| pos_restaurant_urban_piper | EE | POS Restaurant Urban Piper | — |
| spreadsheet_dashboard_pos_restaurant | CE | Spreadsheet dashboards restaurante | — |

### Self-order / Kiosco
| Módulo | Ed | Summary | Sup. |
|--------|----|---------|------|
| pos_self_order | CE | Menú y pedido del cliente en su móvil (2ª app OWL) | data, store, js_model, self, receipt |
| pos_self_order_sale | CE | POS Self Order Sale | self, owl |
| pos_self_order_adyen | CE | Pago Adyen en self-order | self |
| pos_self_order_stripe | CE | Pago Stripe en self-order | self, owl |
| pos_self_order_razorpay | CE | Pago Razorpay en self-order | self, owl |
| pos_self_order_epson_printer | CE | Epson ePOS en kiosco | self, owl |
| pos_self_order_iot | EE | IoT en kiosco | self, owl |
| pos_self_order_preparation_display | EE | Self-order → preparation display | self, store, owl |
| pos_online_payment_self_order | CE | Pago online en self-order | self, store, owl |
| pos_online_payment_self_order_preparation_display | EE | Online self-order → preparation | self |
| pos_order_tracking_display | EE | Estado de pedido para el cliente | self |

### Preparación / cocina
| Módulo | Ed | Summary | Sup. |
|--------|----|---------|------|
| pos_preparation_display | EE | Display Orders for Preparation stage | data, store, receipt |
| pos_hr_preparation_display | EE | Preparation display + HR | — |

### Pagos (terminales y online)
| Módulo | Ed | Summary | Sup. |
|--------|----|---------|------|
| pos_online_payment | CE | Point of Sale online payment | data, store, payment |
| pos_stripe | CE | Stripe payment terminal | data, payment |
| pos_adyen | CE | Adyen payment terminal | data, store, payment |
| pos_razorpay | CE | Razorpay payment terminal | payment, owl |
| pos_paytm | CE | PayTM payment terminal | payment, owl |
| pos_mercado_pago | CE | Mercado Pago Smart Point terminal | payment, store, owl |
| pos_pine_labs | CE | Pine Labs payment terminals | payment, owl |
| pos_six | CE | Six payment terminal | data, payment |
| pos_viva_wallet | CE | Viva Wallet payment terminal | data, store, payment |
| pos_tyro | EE | Tyro payment terminal (AU) | data, store, payment |
| pos_iot_six | EE | Six a través de IoT | payment, owl |
| pos_settle_due | EE | Saldar deuda del cliente en el POS | data, store, screen, button, receipt |

> Capas de terminal: **`pos_<terminal>`** (base) → **`pos_restaurant_<terminal>`**
> (propina) → **`pos_self_order_<terminal>`** (pago en kiosco). Copia la capa que
> encaje con tu escenario.

### Hardware / IoT / periféricos
| Módulo | Ed | Summary | Sup. |
|--------|----|---------|------|
| pos_iot | EE | Use IoT Devices in the PoS | data, store, screen, payment |
| pos_epson_printer | CE | Epson ePOS Printers in PoS | data, store, owl |
| pos_barcodelookup | EE | Crear producto escaneando con cámara | owl |
| pos_pricer | EE | Etiquetas electrónicas Pricer | — |

### Mensajería / delivery / misc
| Módulo | Ed | Summary | Sup. |
|--------|----|---------|------|
| pos_sms | CE | POS - SMS (recibo por SMS) | owl, receipt |
| whatsapp_pos | EE | WhatsApp-POS | owl, receipt |
| pos_urban_piper | EE | POS - UrbanPiper (agregador delivery) | data, store, screen |
| pos_urban_piper_enhancements | EE | UrbanPiper Enhancements | data, store, owl |
| pos_urban_piper_ubereats | EE | UrbanPiper - Uber Eats | owl |
| pos_urban_piper_swiggy | EE | UrbanPiper - Swiggy | — |
| pos_urban_piper_zomato | EE | UrbanPiper - Zomato | — |
| pos_no_followup | EE | POS "no followup" handling | — |
| spreadsheet_dashboard_pos_hr | CE | Spreadsheet dashboards HR | — |

### Contabilidad / impuestos / reporting
| Módulo | Ed | Summary | Sup. |
|--------|----|---------|------|
| pos_account_tax_python | CE | Custom taxes in POS (fórmula Python) | data, receipt |
| pos_account_reports | EE | Bridge POS ↔ account_reports (informes fiscales) | — |
| pos_avatax | EE | Avatax support in POS | data, store, owl |
| pos_edi_ubl | CE | UBL in the Point of Sale | — |

## 4. Fiscalización / EDI por país (grupo l10n_*_pos)

34 módulos `l10n_*_pos` país-específicos de **certificación de caja registradora**
y **factura electrónica** (+ `l10n_test_pos_qr_payment`, de test — no lo uses como
ejemplar). Cuando el requisito sea fiscal-legal de un país, empieza por el l10n de
ese país; para el patrón de "override del recibo/EDI a partir del POS" son todos
variaciones del mismo esquema.

**España (relevante para nosotros):** `l10n_es_pos`, `l10n_es_edi_tbai_pos`
(TicketBAI), `l10n_es_edi_verifactu_pos` (Veri*factu).

**Cert. de caja (blackbox / fiscal cert):** `l10n_fr_pos_cert`, `l10n_fr_pdp_pos`,
`l10n_de_pos_cert`, `l10n_de_pos_res_cert`, `pos_blackbox_be` (EE),
`pos_l10n_se` (EE, Suecia).

**EDI factura electrónica:** `l10n_br_edi_pos`, `l10n_cl_edi_pos`,
`l10n_co_edi_pos`, `l10n_ec_edi_pos`, `l10n_mx_edi_pos`, `l10n_pe_edi_pos`,
`l10n_sa_edi_pos`, `l10n_jo_edi_pos`, `l10n_ke_edi_oscu_pos`,
`l10n_my_edi_pos`, `l10n_vn_edi_viettel_pos`.

**Otros país-específicos:** `l10n_ar_pos`, `l10n_be_pos_restaurant`,
`l10n_be_pos_sale`, `l10n_ch_pos`, `l10n_co_pos`, `l10n_gcc_pos`, `l10n_id_pos`,
`l10n_in_pos`, `l10n_in_reports_gstr_pos`, `l10n_it_pos`, `l10n_mt_pos`,
`l10n_pe_pos`, `l10n_pl_reports_pos_jpk`, `l10n_sa_pos`, `l10n_uy_pos`,
`l10n_account_withholding_tax_pos`.

> No hay dos regímenes fiscales iguales: NO copies un l10n de un país para otro.
> Úsalos solo como referencia del *punto de enganche* (dónde el estándar permite
> inyectar el dato fiscal en el recibo/pedido), no del contenido legal.

## 5. Lectura de la cadena de dependencias

- **`point_of_sale`** es la raíz. Casi todo depende directa o transitivamente de él.
- Los **bridges** (`pos_<A>_<B>`) existen para no acoplar A y B: p.ej.
  `pos_sale_loyalty` solo se instala si tienes `pos_sale` **y** `pos_loyalty`.
  Cuando tu feature cruza dos dominios, sigue este patrón (módulo puente) en vez
  de meter la lógica en uno de los dos.
- Muchos bridges son **auto_install** (aparecen solos al tener ambas dependencias):
  compruébalo en el manifest antes de asumir que el cliente lo instaló a mano.
- Para escenarios reales, el stack típico de un restaurante EE es:
  `point_of_sale` → `pos_restaurant` → `pos_hr(_restaurant)` →
  `pos_preparation_display` → `pos_self_order` → terminal de pago.
