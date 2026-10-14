# Registros de Odoo del MVI

El skill **no crea** estos registros en cada oferta: los resuelve en tiempo de ejecución (plantilla
por nombre, productos por `default_code`). Se crean **una vez** por base de datos y solo con
confirmación explícita. Si falta alguno, el skill se para y remite aquí.

**Creados en la base de España (`portal.dynapps.es`) el 28-sep-2026.** Los ids son de esa base:
en otra se buscan por nombre o código, nunca se copian.

## Productos

Nombre y descripción de venta (`description_sale`, la que sale en la línea del presupuesto) viven
en `config/mvi_offer.json` (`lineas.<id>.producto` y `precios.plataforma_productos`), no aquí.
Todos empiezan por «MVI ·» y llevan las apps entre paréntesis, para encontrarlos buscando «MVI»,
el nombre de la app o el código.

| `default_code` | Plantilla / variante | Precio | Categoría | Cuenta de ingreso | Recurrente |
|---|---|---:|---|---|---|
| `MVI-L2O` | 356 / 354 | 1.500 € | Servicios Odoo | `705011` Implantación ERP | no |
| `MVI-WEB` | 357 / 355 | 1.500 € | Servicios Odoo | `705011` Implantación ERP | no |
| `MVI-O2C` | 351 / 349 | 1.500 € | Servicios Odoo | `705011` Implantación ERP | no |
| `MVI-P2P` | 352 / 350 | 1.500 € | Servicios Odoo | `705011` Implantación ERP | no |
| `MVI-R2R` | 353 / 351 | 1.500 € | Servicios Odoo | `705011` Implantación ERP | no |
| `MVI-H2R` | 358 / 356 | 1.500 € | Servicios Odoo | `705011` Implantación ERP | no |
| `MVI-P2R` | 359 / 357 | 1.500 € | Servicios Odoo | `705011` Implantación ERP | no |
| `MVI-SRV` | 354 / 352 | 90 € | Recurrentes | `705015` Servidores Odoo | sí, mensual |
| `MVI-MNT` | 355 / 353 | 59 € | Recurrentes | `705012` Mantenimiento ERP | sí, mensual |

Comunes: `type=service`, `sale_ok=True`, `purchase_ok=False`, `taxes_id=[2]` (IVA 21 % venta),
UdM `Unidades`, compañía 1. Todas con `service_policy=ordered_prepaid` (se facturan por pedido).
Líneas de proceso: `service_tracking=project_only` con la plantilla de proyecto «Plantilla · MVI»
(id 1469): al confirmar el pedido nace un proyecto con cinco tareas, una por fase (0 · Inicio …
4 · Arranque), donde se imputan las horas. Servidor y mantenimiento: `service_tracking=no`.

**Facturación 50/50** (decidido 29-sep-2026): al confirmar el pedido, factura de anticipo del 50 %
(asistente de anticipo de Odoo); en el arranque, la factura final descuenta el anticipo.

### Las dos reglas de Dyn Brain que aplican

Fuente: *A qué cuenta de ingreso va cada venta* (doc-administration).

1. **Un producto nuevo no se publica sin cuenta de ingreso.** Las categorías `Servicios Odoo` y
   `Recurrentes` caen en la `705000`, que no se usa nunca: la cuenta va **en la ficha de cada
   producto**. Al crear un producto MVI nuevo, `property_account_income_id` es obligatorio.
2. **Si una línea vende dos cosas distintas, son dos líneas.** Por eso la plataforma de
   149 €/mes son dos productos: servidor (90 €, `705015`) y mantenimiento esencial (59 €,
   `705012`). Al cliente la oferta le enseña la suma.

## Plantilla de presupuesto: `Implantación MVI` (id 21)

- `plan_id`: `Mantenimiento mensual` (id 2), para que servidor y mantenimiento se facturen cada
  mes y las líneas de proceso una sola vez.
- `require_signature=True`, `require_payment=False`, compañía 1.
- Líneas: las nueve de arriba, cantidad 1, en el orden del metro (L2O, WEB, O2C, P2P, R2R, H2R,
  P2R) y la plataforma al final. Las cuatro últimas líneas de proceso se añadieron el 28-sep-2026. **El skill borra las líneas de proceso que el cliente
  no ha elegido**; servidor y mantenimiento se quedan siempre.
- `note`: condiciones de partida, regla del plazo de deberes y licencias fuera, copiadas de
  `config/mvi_offer.json`.

## Veri*factu

La línea O2C incluye Veri*factu con el módulo estándar **`l10n_es_edi_verifactu`** (Odoo 19,
instalado en la base de España el 28-sep-2026). En la base del cliente se instala con la
localización española; `indaws_l10n_es_edi_verifactu` y `indaws_l10n_es_reav_verifactu` no
entran en el MVI.

## Cambiar un precio

Dos sitios, siempre juntos: `config/mvi_offer.json` y el `list_price` del producto (si es la
plataforma, que la suma de los dos siga siendo `plataforma_mensual`). Luego
`python3 scripts/sync_embeds.py` y `tests/mvi_smoke.py`, que falla si /qualify o las páginas del
concepto no coinciden con la configuración.
