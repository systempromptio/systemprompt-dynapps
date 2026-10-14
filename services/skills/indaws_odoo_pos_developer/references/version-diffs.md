# POS v18 <-> v19: cheat-sheet de diffs y patrones canonicos

> Citas verificadas en el entorno del autor. Resolver tu fuente via
> `references/shared/odoo-source.md`. v17 ~ v18 salvo donde se indique; ante
> duda en v17 [delega] en `odoo-version-api-reference`.

## Import paths JS (lo que mas rompe)

| Que | v18 | v19 |
|-----|-----|-----|
| PosStore | `@point_of_sale/app/store/pos_store` | `@point_of_sale/app/services/pos_store` |
| `usePos` | `@point_of_sale/app/store/pos_hook` | `@point_of_sale/app/hooks/pos_hook` (`point_of_sale/static/src/app/hooks/pos_hook.js:6`, `usePos`) |
| PaymentInterface | `@point_of_sale/app/payment/payment_interface` | `@point_of_sale/app/utils/payment/payment_interface` |
| makeAwaitable / ask | `@point_of_sale/app/store/make_awaitable_dialog` | **`@point_of_sale/app/utils/make_awaitable_dialog`** (MOVIDO; `app/utils/make_awaitable_dialog.js`) |
| OrderReceipt | `@point_of_sale/app/screens/receipt_screen/receipt/order_receipt` | idem (verificado: misma ruta en v19) |
| Popups | v18 NO tiene `app/components/popups/` (solo `css/popups`) | `app/components/popups/` (nuevo dir) |
| router | `app/store/router.js` | `app/services/pos_router_service.js` (activo) + `app/hooks/pos_router_hook.js` (nuevo); `app/store/router.js` sigue existiendo (legacy) |

> Rutas v19 verificadas contra `point_of_sale/static/src` de la fuente v19
> (2026-07): `app/hooks/pos_hook.js`, `app/services/pos_store.js`,
> `app/utils/payment/payment_interface.js`, `app/utils/make_awaitable_dialog.js`,
> `app/components/popups/`, `app/services/pos_router_service.js`.

Regla: en v19 muchos `app/store/*` se han movido a `app/services/*` y los utilitarios
de pago a `app/utils/payment/*`. Un import v18 en v19 = "module not found".

## Registries y firmas que cambian

| Concepto | v18 | v19 |
|----------|-----|-----|
| Registry de pantallas | `pos_screens`, valor = Clase | `pos_pages`, valor = `{component, ...}` |
| Navegacion entre pantallas | `pos.showScreen(name, props)` | router (`pos_router_service`) |
| `register_payment_method` | `point_of_sale/static/src/app/store/pos_store.js:2395`, `register_payment_method` | `point_of_sale/static/src/app/services/pos_store.js:3112`, `register_payment_method` |
| `PosStore` base class | `extends Reactive` (`:52`) | `extends WithLazyGetterTrap` (`:57`) |
| `_load_pos_data_*` | reciben `config_id` (int) | reciben `config` (recordset) |
| Lectura de datos | `_load_pos_data(data)` | `_load_pos_data_search_read(data, config)` |

Firmas Python detalladas en `data-loading.md`.

## Cambios de MODULOS v18 -> v19 (parque de addons)

Verificado por diff de manifests CE (`odoo19/addons`) y EE (`odoo_enterprise/19`)
contra v18 (2026-07). Impacta a QUE módulo heredas y a QUE está disponible por edición.

### Reestructuraciones grandes (no es un simple rename)

1. **Hardware `hw_*` -> `iot_*` (CE).** v19 CE ELIMINA `hw_drivers`, `hw_escpos`,
   `hw_posbox_homepage` y los sustituye por `iot_base`, `iot_box_image`,
   `iot_drivers`. Todo desarrollo que tocara `hw_*` (drivers, IoT Box, ESC/POS)
   se reescribe contra los `iot_*`.
2. **Preparation display absorbido en `pos_enterprise` (EE).** v19 ELIMINA los
   módulos base `pos_preparation_display` y `pos_hr_preparation_display`; su código
   (modelos `pos_prep_display/line/order/stage/state.py`, componente OWL
   `app/components/preparation_display`) vive ahora DENTRO de `pos_enterprise`.
   Los bridges `pos_restaurant_preparation_display` / `pos_self_order_preparation_display`
   dependen ya de `pos_enterprise` (no del módulo base). Un override de
   `pos_preparation_display` en v18 NO tiene módulo destino en v19: se parchea sobre
   `pos_enterprise`.
3. **`pos_enterprise` gana infra de datos propia (EE).** Nuevos en v19:
   `models/pos_load_mixin.py` y `models/data_validator.py` — capa de carga/validación
   adicional a la del core. Si extiendes carga de datos en EE, revísala.
4. **Delivery: `pos_urban_piper_*` -> arquitectura `pos_platform_order` (EE).** v19
   ELIMINA `pos_urban_piper_swiggy` y `pos_urban_piper_zomato`; introduce
   `pos_platform_order`, `pos_platform_order_gofood`, `pos_platform_order_grabfood`
   (sistema de pedidos de plataforma nuevo). `pos_urban_piper`,
   `_enhancements`, `_ubereats` siguen. Para integrar delivery nuevo en v19, mira
   `pos_platform_order`, no el patrón UrbanPiper.
5. **Blackbox BE explotado en `l10n_be_pos_blackbox*` (EE).** v18 `pos_blackbox_be`
   -> v19 `l10n_be_pos_blackbox` + submódulos `_hr`, `_loyalty`, `_self_order`,
   `_settle_due`, `_urban_piper`. La certificación fiscal belga pasa a familia l10n.
6. **Suecia: `pos_l10n_se` -> `l10n_se_pos` (rename, EE).**

### Renames de terminales / módulos (mismo feature, otro nombre)

- **`pos_viva_wallet` -> `pos_viva_com`** (CE) + nuevo `pos_self_order_viva_com`.

### Módulos ELIMINADOS en v19 (feature retirado o movido)

- CE: `pos_epson_printer`, `pos_self_order_epson_printer`, `pos_paytm`, `pos_six`
  (Six queda solo vía IoT en EE: `pos_iot_six`). Impresión Epson/ESC-POS pasa por
  la vía IoT (`iot_drivers`).
- EE: `pos_no_followup`, `l10n_be_reports_post_wizard` (además de los absorbidos
  en los puntos 2 y 4).

### Módulos NUEVOS en v19

- **Cajones automáticos de efectivo (CE, categoría nueva):** `pos_glory_cash`,
  `pos_cashdro`, `pos_cashmatic` (dispositivos de pago en efectivo automático).
- **Terminales de pago nuevos (CE):** `pos_mollie`, `pos_qfpay`, `pos_dpopay`,
  `pos_safaricom`, `pos_imin`, `pos_viva_com` (+ self-order: `pos_self_order_qfpay`,
  `pos_self_order_pine_labs`, `pos_self_order_viva_com`).
- **Puentes nuevos (CE):** `pos_repair` (POS <-> Repair).
- **IoT device-específicos (EE):** `pos_iot_ingenico`, `pos_iot_worldline`,
  `pos_iot_adam_scale`, `pos_event_iot`.
- **Otros EE:** `pos_appointment` (citas genéricas; `pos_restaurant_appointment`
  sigue aparte), `pos_enterprise_sms_whatsapp`, `pos_platform_order*` (ver punto 4).
- **Fiscal nuevo:** `l10n_ae_pos` (CE, EAU), `l10n_at_pos` (Austria),
  `l10n_gt_edi_pos` (Guatemala), `l10n_in_pos_urban_piper`,
  `l10n_br_edi_pos_fiscal_reform`, `l10n_us_direct_deposit`.

> **España v19:** los l10n POS de España siguen siendo `l10n_es_pos`,
> `l10n_es_edi_tbai_pos`, `l10n_es_edi_verifactu_pos` (sin cambios de nombre
> detectados en el diff v18->v19).

### OCA para España (v19) — estado real

- El repo **OCA `pos` v19** tiene HOY solo **2 módulos**: `pos_edit_order_line`
  (editar líneas de pedido) y `pos_partner_birthdate` (fecha de nacimiento en la
  pantalla de cliente). Cobertura POS de OCA en v19 = mínima (arranque del ciclo v19).
- El repo **OCA `l10n-spain` v19 NO tiene módulos POS** (ninguno con `pos` en el
  nombre ni que dependa de `point_of_sale`). Para POS-España en v19 se depende del
  estándar (`l10n_es_pos` + TicketBAI/Veri*factu) y del custom; NO hay puente OCA.
- Verificado en `$OCA/v19/pos` y `$OCA/v19/l10n-spain` (2026-07). Antes de asumir un
  módulo OCA v19, comprueba que existe: la mayoría aún no está portado desde v18.

## Patrones canonicos (variante v18 y v19)

### A) Anadir un campo a un modelo cargado

v18:
```python
def _load_pos_data_fields(self, config_id):
    return super()._load_pos_data_fields(config_id) + ["mi_campo"]
```
v19:
```python
def _load_pos_data_fields(self, config):
    return super()._load_pos_data_fields(config) + ["mi_campo"]
```
(Igual en ambos: parchear ESTE metodo es obligatorio, el campo no viaja solo.)

### B) Anadir un modelo nuevo

Python (ambas versiones) — `pos.load.mixin` + override de `_load_pos_data_models`
(ajustando firma: `config_id` en v18, `config` en v19).
JS (ambas versiones, mismo patron):
```js
export class MiModelo extends Base { static pythonModel = "mi.modelo"; }
registry.category("pos_available_models").add(MiModelo.pythonModel, MiModelo);
```
Solo cambia el import de `Base`/`related_models` si la ruta se movio (verificar).

### C) Anadir una screen

v18:
```js
import { registry } from "@web/core/registry";
registry.category("pos_screens").add("MiScreen", MiScreen);
// navegar: this.pos.showScreen("MiScreen", props)
```
v19:
```js
registry.category("pos_pages").add("MiScreen", { component: MiScreen });
// navegar: via el pos router service
```

### D) Terminal de pago

v18:
```js
import { PaymentInterface } from "@point_of_sale/app/payment/payment_interface";
import { register_payment_method } from "@point_of_sale/app/store/pos_store";
register_payment_method("mi_terminal", PaymentMiTerminal);
```
v19:
```js
import { PaymentInterface } from "@point_of_sale/app/utils/payment/payment_interface";
import { register_payment_method } from "@point_of_sale/app/services/pos_store";
register_payment_method("mi_terminal", PaymentMiTerminal);
```
Python identico en ambas: override de `_get_payment_terminal_selection`
(`point_of_sale/models/pos_payment_method.py:11`, `PosPaymentMethod._get_payment_terminal_selection`); el tech_name debe casar con el primer arg JS.

## Roturas conocidas al migrar v18 -> v19

1. `import` de `app/store/pos_store` -> mover a `app/services/pos_store`.
2. `registry.category("pos_screens")` -> `pos_pages` con descriptor `{component}`.
3. `pos.showScreen(...)` -> usar el router service.
4. Overrides Python de `_load_pos_data*` con firma `config_id` -> `config`.
5. `_load_pos_data(data)` -> `_load_pos_data_search_read(data, config)`.
6. PaymentInterface import path cambiado (`app/utils/payment/...`).
7. Popups reubicados a `app/components/popups/`.

Ante cualquier firma/API concreta no listada aqui, verifica en fuente o [delega] en
`odoo-version-api-reference`.
