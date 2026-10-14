# Mapeo de relleno y cómo añadir un tipo de contrato

## Datos que se leen de Odoo (por pedido)

De `sale.order` (por `name` PRxxxx o id): `name`, `partner_id`, `date_order`,
`amount_untaxed`, `project_id`.
De `res.partner`: `name`, `vat`, `street`, `street2`, `zip`, `city`, `state_id`
(provincia, se le quita " (ES)"), `country_id`, `email`, `phone`.
De `project.milestone` (del `project_id`): hitos con `deadline` e importe en
`x_studio_price`. En la tabla de pagos los hitos se **agrupan por día de factura**
(`group_hitos_by_day`): los del mismo `deadline` se funden en **una sola línea** con
el importe sumado (independientemente de si ya se han facturado).

> Contexto `lang: es_ES` obligatorio en las lecturas o país/provincia salen en inglés.

## Cómo se rellena la plantilla (`fill_contract.py`)

Reemplazo **run-aware** (Word parte una palabra en varios *runs*; se opera a
nivel de carácter para no romper coincidencias ni formato).

1. **Placeholders con nombre** (globales, en todo el documento):
   `[PXXXXX]`→nº pedido · `[Entidad del Cliente]`/`[Entidad Legal del Cliente]`/
   `[ENTIDAD LEGAL DEL CLIENTE]`→razón social · `[Dirección/País/Teléfono/Email del Cliente]`
   · `[Nombre o Cargo]`→firmante (**obligatorio** vía `--signer-name`; el script no
   se ejecuta sin él).
2. **Corchetes vacíos** `[   ]` (regex `\[\s*\]`): se mapean por la **etiqueta de
   su línea** (case-insensitive): `Fecha:`→fecha contrato · `Cliente:`→[razón social, CIF]
   (dos en la misma línea) · `Razón social:` · `Domicilio social:` · `CIF/NIF:` ·
   línea con "efectivo desde"→Fecha de Efectos.
2b. **Forma de pago** (`apply_payment_method`, según `--payment-method`):
   - `sepa` (por defecto): no toca la plantilla (queda el bloque Domiciliación SEPA B2B).
   - `transferencia`: sustituye el bloque SEPA (desde "Domiciliación bancaria SEPA B2B"
     hasta "Titular de la cuenta:") por el texto de transferencia (IBAN/banco de
     `config.json > payment.transfer`, plazo = `fecha_pago_offset_days`) y quita la mención
     "mediante instrucciones SEPA B2B" de la línea que introduce el calendario.
3. **Tabla de calendario de pagos** (solo tipos con `has_payment_table: true`):
   se localiza la tabla cuyo encabezado tiene *Importe* + *Fecha Factura*; los hitos
   se **agrupan por día** (una fila por fecha, importe sumado) y se genera una fila
   por factura (Importe · Fecha Factura=deadline · Fecha Pago=deadline +
   `fecha_pago_offset_days`).

## Qué queda SIN rellenar a propósito

- `[Se proporcionará en documento separado]` (texto intencionado).
- `[A DETERMINAR EUR]` (importe negociado del contrato de mantenimiento).
- Anexo RGPD del Marco (placeholders `[p. ej., ...]`, `[Prestación de los Servicios...]`):
  contenido legal — se deja para revisión.
- Fechas de referencia de anexos ("CONTRATO MARCO DE FECHA [ ]"): no empiezan por
  "Fecha:", se dejan.

## Añadir un tipo de contrato nuevo

1. Coloca la plantilla `.docx` (ya rebrandeada a Dynapps) en la carpeta OneDrive
   `Contratos Indaws` (`config.json > templates_dir`). **No** se usan plantillas
   locales del skill.
2. Añade una entrada en `config.json > contract_types`:
   ```json
   "<clave>": {
     "label": "Nombre visible del contrato",
     "template": "Mi_Plantilla.docx",
     "has_payment_table": false,
     "signature": {"anchor": "Firma:", "dy": 0.006, "page_anchor": "Nombre: Julio Pons", "posX": 0.12, "posY": 0.5, "width": 0.2, "height": 0.05},
     "validated": false
   }
   ```
3. Los placeholders estándar (`[Entidad Legal del Cliente]`, `[PXXXXX]`, `[   ]`
   con etiqueta) funcionan sin tocar código. Si la plantilla usa placeholders
   nuevos, añádelos al dict `named` de `fill_contract.py`.
4. El recuadro de firma se ancla automáticamente bajo `"Firma:"`. Genera una vez y
   **revisa en Odoo**; si hace falta, ajusta `dy` (separación) o el `anchor`.
