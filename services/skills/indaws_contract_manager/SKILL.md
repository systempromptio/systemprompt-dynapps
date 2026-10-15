---
name: indaws-contract-manager
description: >-
  Genera contratos de cliente desde un pedido de Odoo y los deja listos para firmar en Firmas
  (Sign): lee datos del cliente y los hitos de pago del proyecto, rellena la plantilla Dynapps del
  tipo indicado, la convierte a PDF y crea un Documento de firma (sign.request) con el recuadro del
  cliente, SIN enviarlo. Varios tipos (marco/implantación, mantenimiento, hosting), ampliable por
  config. Triggers: "genera el contrato del pedido PRxxxx", "contrato de mantenimiento para firmar",
  "sube el contrato a firmas", "contrato de hosting del pedido", "/contract-manager".
version: 1.2.0
effort: medium
model: sonnet
allowed-tools: [Bash, Read, AskUserQuestion]
---

# Contract Manager

Automatiza el proceso de administración: al cerrar un pedido, generar el contrato con los datos
del cliente y subirlo a Firmas de Odoo con el recuadro para que firme el cliente. El **envío lo
lanza la persona** desde Odoo (no se envía nada automáticamente).

## When to Use This Skill

- "genera el contrato marco del pedido PR3785"
- "prepara el contrato de mantenimiento de PR3999 para firmar"
- "sube el contrato de hosting del PR4001 a firmas"

Un contrato por invocación. El **tipo lo indica el usuario** (no se autodetecta).

## Requisitos

- **Odoo**: autentica primero con la skill `odoo-pilot` (`eval $(auth.sh)` deja
  `ODOO_URL/ODOO_DB/ODOO_KEY` en el entorno). Nunca hardcodees credenciales.
- **LibreOffice** (`soffice`) para convertir docx→PDF, y **python-docx** + **PyMuPDF** (`fitz`).

## Workflow

### 1. Determinar tipo y pedido
El usuario da el nº de pedido (PRxxxx o id) y el tipo. Tipos en `config.json > contract_types`
(hoy: `marco`, `mantenimiento`, `hosting`). Si no indica el tipo, **pregúntaselo** (AskUserQuestion)
listando los tipos disponibles.

### 2. Pedir el firmante del cliente [STOP obligatorio: no generes sin esto]
Pregunta el **nombre y cargo** de quien firma por parte del cliente (va en `[Nombre o Cargo]`)
con AskUserQuestion. **Si falta cualquier dato pendiente (nombre, cargo, …), PÁRATE y
pregúntalo; no generes el contrato hasta tenerlo.** El script rechaza ejecutarse sin
`--signer-name` (no se generan contratos con placeholders pendientes).

### 2b. Preguntar la forma de pago [STOP obligatorio]
Pregunta con AskUserQuestion si el pago es **Domiciliación SEPA B2B** (`sepa`, texto por
defecto de la plantilla) o **Transferencia bancaria** (`transferencia`). Con `transferencia`
el bloque SEPA se sustituye por el texto de transferencia (IBAN/banco desde `config.json >
payment.transfer`). El script no se ejecuta sin `--payment-method`.

### 3. Generar y subir
```bash
python3 scripts/generate_contract.py --order <PRxxxx|id> --type <clave> \
  --signer-name "<Nombre Cargo>" --payment-method <sepa|transferencia>
```
Lee el pedido + hitos, rellena la plantilla, convierte a PDF y crea el **Documento** en Firmas
(sign.request) asignado al cliente, **sin enviar**. Devuelve JSON con los campos rellenados, los
placeholders que quedan, los ids de Sign y la `review_url`.

Para probar sin escribir en Odoo: añade `--fill-only` (genera docx+pdf, no sube).

### 4. Revisar y enviar (lo hace el usuario en Odoo)
Pasa al usuario la `review_url` (o "Firmas → Documentos"). Ahí revisa el PDF, **ajusta el recuadro**
si hace falta y pulsa **Enviar** para que le llegue al cliente. Recuérdale que hasta ese momento el
cliente no ha recibido nada.

## Rules and Guidelines

- **No se envía nada** automáticamente (`auto_send: false`). Crear el Documento NO emaila al cliente.
- **El tipo lo indica el usuario**; nunca lo deduzcas del pedido.
- Idioma ES; lecturas de Odoo con contexto `lang: es_ES` (país/provincia en español).
- Fechas: contrato = hoy; Fecha de Efectos = fecha del pedido (configurable en `config.json > dates`).
- Fecha Pago del calendario = vencimiento del hito + `fecha_pago_offset_days` (por defecto 7).
- Añadir un tipo nuevo = plantilla en `templates/` + entrada en `config.json` (ver
  `references/fill-mapping.md`); no toca código salvo placeholders nuevos.
- **Plantillas SOLO desde OneDrive**: `config.json > templates_dir` (carpeta OneDrive
  "Contratos Indaws") es la **única** fuente. **No se usan las plantillas guardadas en el
  skill** (`templates/` es histórico). Si no hay acceso a la plantilla en OneDrive, el script
  **para y NO genera** el contrato (avisa al usuario de que sincronice/comparta el OneDrive).
- **Hitos agrupados por día**: en la tabla de pagos, los hitos de la **misma fecha de factura**
  se agrupan en **una sola línea** con el importe sumado (da igual si ya se han facturado).
- **Forma de pago**: la elige el usuario (`sepa` | `transferencia`). `sepa` deja el texto de
  Domiciliación SEPA B2B de la plantilla; `transferencia` lo reemplaza por el texto de
  transferencia con IBAN/banco de `config.json > payment.transfer` (plazo = `fecha_pago_offset_days`).

## GOTCHAS

1. **Odoo Sign solo acepta PDF** — el .docx se rechaza (*"Is it a valid PDF?"*). Hace falta
   LibreOffice; si `soffice` no está, la conversión falla.
2. **Crear el Documento NO envía email** (verificado). Nace en estado "To Sign" pero el cliente no
   recibe nada; el envío real se hace en Odoo (Enviar/Reenviar).
3. **No se puede borrar la plantilla de respaldo** (el Documento comparte su PDF): se **archiva**
   (`active=False`) para que no ensucie Plantillas.
4. **El recuadro de firma se ancla al texto "Firma:" del cliente** (columna izquierda) y se
   coloca justo **debajo** de él (posición calculada con PyMuPDF, no fija). `posX/posY` del
   config solo se usan como respaldo si no hay PyMuPDF o no se encuentra el ancla. Revisa en
   Odoo la primera vez por tipo y ajusta `dy` si hiciera falta.
5. **`write` en JSON2 usa la clave `vals`, no `values`** (si no: "missing required argument 'vals'").
6. **El importe del hito está en `x_studio_price`** (campo Studio), no en un campo estándar.
7. **Placeholders que quedan a propósito**: `[A DETERMINAR EUR]`, anexo RGPD (`[p. ej., ...]`),
   `[Se proporcionará en documento separado]`. No los inventes. (`[Nombre o Cargo]` ya NO se deja
   pendiente: el firmante es obligatorio, ver paso 2.)

## Delegates to
| Skill | Suite | When |
|-------|-------|------|
| odoo-pilot | common | Autenticación/conexión a Odoo (deja ODOO_URL/DB/KEY en el entorno) |

## Consumed by
Ninguna — skill terminal de administración/dirección.

## REFERENCES
- `references/fill-mapping.md` — datos leídos, mapeo de placeholders y cómo añadir un tipo.
- `references/odoo-sign-api.md` — modelo Sign 19, JSON2 y gotchas (crear ≠ enviar, archivar plantilla).
- `config.json` — registro de tipos de contrato, política de fechas, pago y firma.
- `references/shared/odoo-connection.md` — patrón canónico de conexión/auth a Odoo (vía odoo-pilot).

## Success Criteria
- El contrato correcto se genera con los datos del cliente y el calendario de pagos (hitos) bien.
- Queda un Documento en Firmas asignado al cliente con el recuadro de firma, **sin enviar**.
- El usuario revisa en Odoo y envía cuando quiere; nada llega al cliente antes.
