<!-- verified: v18.0 @ 2026-06 (l10n_es_edi_verifactu CE + _pos; anclas verificadas vs fuente) -->
# Veri\*Factu (`l10n_es_edi_verifactu`) — Sistema antifraude AEAT

Sistema de **registro de facturación encadenado** que Odoo envía a la AEAT (modo Veri\*Factu, remisión
voluntaria inmediata). Obligatorio en territorio común (fuera de País Vasco/Navarra, que usan
[TicketBAI](ticketbai.md)). Está en **CE** (`l10n_es_edi_verifactu`) + puente POS
(`l10n_es_edi_verifactu_pos`). Gotchas en [`gotchas.md`](gotchas.md) (ESG41-55).

## Modelo `l10n_es_edi_verifactu.document` (`l10n_es_edi_verifactu/models/verifactu_document.py:97`, `L10nEsEdiVerifactuDocument`)

Cada factura posteada genera un **registro de facturación** (alta o anulación). Guarda el JSON en un
`ir.attachment`, envía el XML a la AEAT y almacena la respuesta.

| Campo | Notas |
|-------|-------|
| `state` (`:154`) | **lo fija la respuesta AEAT**: `rejected` / `registered_with_errors` (AceptadoConErrores) / `accepted` (Correcto). `False` mientras está pendiente. No hay "borrador". |
| `document_type` (`:118`) | `submission` (RegistroAlta) / `cancellation` (RegistroAnulacion). |
| `chain_index` (`:112`) | índice en la cadena de la compañía (`ir.sequence` `no_gap`); **solo se asigna si el JSON se generó OK** (`_create_for_record`, `:420`). |
| `json_attachment_id` (`:128`) | el JSON del registro; el XML enviado se deriva de él. |

## Encadenamiento (huella)

`_fingerprint` (`:862`) + `_update_render_vals_with_chaining_info` (`:833`): cada registro lleva
`Encadenamiento` — `PrimerRegistro=S` si es el primero, o `RegistroAnterior` con la **Huella** SHA-256
(`TipoHuella="01"`) del documento previo (`res_company._l10n_es_edi_verifactu_get_last_document`,
`l10n_es_edi_verifactu/models/res_company.py:89`, `ResCompany._l10n_es_edi_verifactu_get_last_document`). La huella concatena `IDFactura` + (alta) `TipoFactura/CuotaTotal/ImporteTotal` +
`Huella` anterior + `FechaHoraHusoGenRegistro` unidos por `&` y hasheados.

## Flujo: factura → documento → AEAT

1. Al enviar la factura con el EDI `es_verifactu`: `account_move_send.py::_call_web_service_before_invoice_pdf_render`
   (`:66`) → `account.move._l10n_es_edi_verifactu_mark_for_next_batch` (`:342`) crea los documentos y
   dispara `trigger_next_batch`.
2. **Envío por lotes**: `verifactu_document.py::trigger_next_batch` (`:885`) agrupa pendientes por
   compañía, ordena por `chain_index`, manda lotes de `BATCH_LIMIT=1000` (`_send_batch`, `:955`, SOAP
   zeep). Mapea `EstadoRegistro`→`state` y guarda `response_csv`.
3. **QR de cotejo**: `_get_qr_code_img_url` (`:293`) construye la URL `ValidarQR` de la AEAT
   (nif/numserie/fecha/importe); expuesto en `account.move.l10n_es_edi_verifactu_qr_code` (`:38`).

## Config (`res.company` / `res.config.settings`)

- `l10n_es_edi_verifactu_required` (`l10n_es_edi_verifactu/models/res_company.py:12`, `class ResCompany`) — activa Veri\*Factu por compañía.
- `l10n_es_edi_verifactu_test_environment` (`:16`, **default True**) — endpoints `prewww*.aeat.es`
  (pruebas) vs `www*.agenciatributaria.gob.es` (producción) (`_l10n_es_edi_verifactu_get_endpoints`, `:43`).
- **Certificado** scope `verifactu` (`certificate/models/certificate.py:7`, `x509`); sin él, falla en creación y en envío.
- `l10n_es_edi_verifactu_next_batch_time` (`:27`) — espera impuesta por la AEAT entre lotes (~60s).

## Cancelación = ANULACIÓN

`account_move.py::l10n_es_edi_verifactu_button_cancel` (`:237`) crea un documento `cancellation`
(RegistroAnulacion) encadenado como un alta; tras aceptarse, `_cancel_after_sending` (`:1204`) cancela
el asiento. **Los documentos encadenados no se borran** (`_never_unlink_chained_documents`, `:181`).

## POS (`l10n_es_edi_verifactu_pos`)

Un documento por **ticket** al cobrar si no se factura (`pos_order.py::action_pos_order_paid`); ticket
simplificado salvo abono con partner; si supera `l10n_es_simplified_invoice_limit` exige factura. POS
**no se puede Veri\*Factu-cancelar** (la vía es facturar el ticket → anulación + factura).

Ver también: [SII](sii.md) (suministro inmediato de info, distinto de Veri\*Factu),
[TicketBAI](ticketbai.md) (País Vasco), [Facturae](facturae.md) (B2G).
