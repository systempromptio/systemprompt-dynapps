# Odoo 19 Sign (Firmas) vía JSON2 — modelo y gotchas

## Protocolo JSON2 (Odoo 19)

`POST {ODOO_URL}/json/2/{model}/{method}` con headers `Authorization: bearer {KEY}`
y `X-Odoo-Database: {DB}`. Cuerpos por método:
- `create` → `{"vals_list": [ {vals} ]}` (devuelve `[id]`)
- `write`  → `{"ids": [...], "vals": {...}}`  ⚠️ la clave es **`vals`**, NO `values`
- `unlink` → `{"ids": [...]}`
- `read` / `search_read` → `{"ids"|"domain", "fields", "limit", "context"}`

Autentica primero con la skill **odoo-pilot** (`eval $(auth.sh)` deja
`ODOO_URL/ODOO_DB/ODOO_KEY`). Nunca hardcodear credenciales.

## Modelo de datos (Sign 19, Enterprise)

Estructura en 3 niveles:
`sign.template` → `sign.document` (el PDF) → `sign.item` (los campos).

- **sign.template**: `name` (req). Contenedor. Se puede archivar (`active=False`).
- **sign.document**: `template_id` (req), `attachment_id` (req → ir.attachment con
  el PDF), `num_pages` (autocalculado). **Solo acepta PDF** (un .docx da
  *"Is it a valid PDF?"*).
- **sign.item** (un campo): `document_id` (req), `type_id` (req → sign.item.type;
  Signature=`item_type='signature'`), `responsible_id` (→ sign.item.role; cliente
  = "Customer"), `page` (1-based), `posX`, `posY`, `width`, `height` (fracciones
  0..1, origen arriba-izquierda), `alignment` (req; vale "left"), `required`.
  Recuadro de firma típico: `width=0.2 height=0.05`.
- **sign.request** (el "Documento" a firmar): `template_id` (req), `reference`
  (nombre visible, req), `send_channel` (req, "email"), `request_item_ids`
  (firmantes: `[[0,0,{"role_id":1,"partner_id":<cliente>}]]`).
  Estados: `shared/sent/signed/canceled/expired` — **no hay "borrador"**; nace en
  `sent` (To Sign). Exige firmantes con **email válido**.

## Gotchas críticos

1. **Crear `sign.request` NO envía email** (verificado: 0 `mail.mail` nuevos al
   crear; los `message_ids` son chatter interno). El cliente no recibe nada hasta
   que se usa **Enviar/Reenviar** en Odoo o se comparte el enlace. Por eso
   `auto_send=false` en config: dejamos el Documento listo y el envío lo lanza el
   usuario.
2. **El Documento comparte el `sign.document` con su plantilla** (`template_document_ids`
   apunta al mismo doc). NO se puede borrar la plantilla sin romper el Documento
   → se **archiva** (`active=False`) para que no aparezca en Plantillas.
3. **Sign solo PDF**: convertir el .docx con LibreOffice
   (`soffice --headless --convert-to pdf`). Odoo NO convierte al subir.
4. **Coordenadas**: se calculan sobre el PDF con PyMuPDF (`fitz`). El recuadro se
   **ancla al texto "Firma:" del cliente** (columna izquierda = ocurrencia más a la
   izquierda en la última página donde aparece) y se coloca **justo debajo**
   (`posY = base_de_Firma/alto + dy`). Si no hay PyMuPDF o no se halla el ancla, se
   usan `page_anchor` + `posX/posY` del config como respaldo. posX/posY son fracciones
   del tamaño de página (origen arriba-izquierda). Revisar en Odoo la primera vez por tipo.
5. **`X-Odoo-Database` header obligatorio**; sin él, auth falla en silencio.

## Registro real de la prueba (PR3785 / TFT Food)

sign.template 514 (archivada) · sign.document 456 (PDF 23 pág) · sign.item 8094
(firma Customer, pág 18) · sign.request 479 (Documento, firmante TFT Food, sin enviar).
