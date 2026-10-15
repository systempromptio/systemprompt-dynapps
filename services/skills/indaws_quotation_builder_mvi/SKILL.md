---
name: indaws-quotation-builder-mvi
description: >
  Cotiza la Implantación Mínima Viable (MVI) de Odoo a precio cerrado: de una a tres líneas de
  proceso a 1.500 € cada una más 149 €/mes de plataforma. Lee el veredicto «Encaje MVI» de
  /qualify, crea el sale.order y genera la oferta HTML + PDF con el mapa de metro. Lo usa el SDR
  sin pasar por comercial. NO para desarrollos, integraciones o más de 10 usuarios →
  quotation-builder-value-based. Triggers: cotiza el MVI, oferta MVI, quote-mvi, implantación
  mínima viable, presupuesto cerrado básico.
version: 1.1.1
model: sonnet
effort: medium
allowed-tools: [Read, Write, Bash, AskUserQuestion, Skill, mcp__88fb5415-35bd-4791-a827-814b99059299__orm_search_read, mcp__88fb5415-35bd-4791-a827-814b99059299__orm_create, mcp__88fb5415-35bd-4791-a827-814b99059299__orm_write, mcp__88fb5415-35bd-4791-a827-814b99059299__orm_unlink, mcp__88fb5415-35bd-4791-a827-814b99059299__orm_execute_kw, mcp__88fb5415-35bd-4791-a827-814b99059299__odoo_request_upload, mcp__88fb5415-35bd-4791-a827-814b99059299__whoami]
---

# Cotizador MVI (Implantación Mínima Viable)

El MVI es la modalidad «tú te lo guisas» de Dynapps: Odoo estándar montado como las demos que ya
entregamos, tres iteraciones de mejora que el cliente tiene que ir superando con sus deberes,
importación de maestros, formación y arranque. **Y fin.** Lo que pida después va por el cauce
habitual. La oferta completa (precios, condiciones, fases, qué incluye) vive en
**`config/mvi_offer.json`**: no se escribe ningún precio ni texto de oferta fuera de ese fichero.

## Importante (no negociable)

- **Precio fijo**: 1.500 € por línea + 149 €/mes de plataforma. **Sin franja de ajuste.** Si el
  cliente negocia, deja de ser un MVI y pasa a comercial.
- **El SDR elige las líneas en /qualify, de una a tres** (`limites.max_lineas`), entre las siete de
  `config/mvi_offer.json`: Lead-to-Order, Website, Order-to-Cash, Procure-to-Pay,
  Record-to-Report, Hire-to-Retire y Project-to-Result. Con cuatro o más, a comercial.
- **Sin condiciones, no hay oferta.** Las ocho condiciones del formulario (estándar, tres
  iteraciones, ≤10 usuarios, 1 empresa/almacén/moneda, español o inglés, sin desarrollos, sin fabricación ni inventario complejo, solo maestros) se cumplen
  todas o el lead va a `quotation-builder-value-based`. `render` se niega por diseño.
- **Nunca horas al cliente**, nunca la comparativa interna con el value-based.
- **Veri*factu va incluido en Order-to-Cash** (`l10n_es_edi_verifactu`, estándar de Odoo 19) y el
  certificado digital de la empresa es un deber del inicio. Sin O2C la oferta no lo menciona.
- **Licencias de Odoo fuera** del pedido y del importe.
- **Nada se envía al cliente desde aquí**: el SDR envía el presupuesto desde Odoo.

## Workflow

Scripts relativos a este skill. `deal.json` de trabajo en el scratchpad.

### Paso 0: lead y operador
1. Lead de `$ARGUMENTS` (id o nombre del `crm.lead`); si no, pregúntalo. Varias coincidencias →
   `AskUserQuestion`. `whoami` → nombre y email del operador (van a la oferta como contacto).

### Paso 1: veredicto de /qualify
1. Lee los `mail.message` del lead (`model='crm.lead'`, `res_id=<id>`, `body ilike 'MVI:'`,
   orden `date asc`), vuelca los `body` a un fichero y ejecuta
   `python3 scripts/mvi_quote.py from-qualify <fichero>`.
2. Salida:
   - `null` (exit 1): el lead no ha pasado por la sección 7 de /qualify. **Para** y pide al SDR
     que la rellene (`/qualify <id>`). Si el lead ya está con comercial y tiene toma de datos
     (`xx_data_collection`), aplica la tabla de `references/qualify_contract.md` §Toma de datos:
     solo va a `true` lo que la toma dice explícitamente, el resto queda sin responder, y enseña
     el mapa al SDR antes de seguir. Es el **camino borrador**: pasos 2-4 con `--borrador`.
   - `NO ELEGIBLE` (exit 2): **para**. Di qué condición falla y que el lead sigue con comercial.
   - `ELEGIBLE` (exit 0): sigue.
   Contrato del formato: `references/qualify_contract.md`.

### Paso 2: deal y comprobación
1. Monta `deal.json`: `cliente` (`partner_name` del lead o su empresa), `lineas` y `condiciones`
   del paso 1, `usuarios` (del resumen de /qualify, «Usuarios Odoo»; si no está, **pregúntalo**),
   `contexto` (una frase sobre el negocio del cliente, **en el idioma de la oferta**, sin adjetivos
   de folleto), `lang` (`es` o `en`, el que pida el cliente), `comercial` (el comercial del lead),
   `fecha` (hoy), `validez_dias: 30` y, si la oferta recorta lo que se habló con el cliente,
   `nota_alcance`: un párrafo con lo hablado, lo que cubre y lo que queda para otra fase.
2. `python3 scripts/mvi_quote.py check deal.json` (en el camino borrador, `--borrador`): tiene que
   salir 0. Si no, **para** y enseña los fallos. `contexto` sale del lead o de la toma; si no hay
   nada, pregúntalo al SDR, nunca lo inventes.
3. `python3 scripts/mvi_quote.py price deal.json` y presenta al SDR: cliente, líneas, implantación,
   cuota y total del año 1. **🛑 STOP: no escribas en Odoo hasta que confirme.**

### Paso 3: sale.order
1. Resuelve la plantilla `Implantación MVI` por nombre y los productos por `default_code`
   (`lineas.<id>.producto` y `precios.plataforma_productos` del config). Si falta algo, **para** y remite a
   `references/odoo_setup.md` (se crean una vez, con confirmación).
2. ¿Ya hay un pedido en borrador con esa plantilla en el lead? → `AskUserQuestion`: actualizarlo o
   crear uno nuevo. Nunca dupliques en silencio.
3. Oferta en inglés: pon antes el contacto en `en_US` si el cliente lo pide (gotcha 14). Crea el
   `sale.order` con `partner_id`, `opportunity_id`, `sale_order_template_id`, `plan_id` de la
   plantilla, `user_id` (el comercial del lead) y **solo las líneas elegidas más servidor y
   mantenimiento** en `order_line` (la nota se hereda de la plantilla; en inglés, escribe la de
   `mvi_quote.py nota deal.json --lang en`). En el camino borrador el pedido queda en borrador,
   no se envía nunca y lleva nota interna con las condiciones pendientes. Relee `amount_untaxed`:
   tiene que ser `implantacion + plataforma_mensual`; si no cuadra, para y enséñalo.

### Paso 4: oferta
1. Añade `sale_order` (el `name` del pedido) a `deal.json` y ejecuta
   `python3 scripts/mvi_quote.py render deal.json --out <dir> --pdf [--lang en] [--borrador]`.
   Corta si hay placeholders, horas visibles, rayas, mayúsculas o español en una oferta inglesa;
   `--borrador` solo admite condiciones sin confirmar y marca la oferta como interna. La oferta lleva el **mapa de metro** del
   value-based, con la red completa de Odoo: las líneas de /qualify encendidas, las otras líneas
   MVI en gris (se pueden añadir) y el resto (web, CRM, fabricación, RRHH…) en discontinuo, no
   disponible en MVI. Sin precios.
2. Sube el PDF con `odoo_request_upload` **sin modelo** (a `sale.order` no se puede apuntar),
   pásalo en `attachment_ids` de un `message_post` interno (`mail.mt_note`) en el pedido y luego
   escribe `res_model`/`res_id` del adjunto para que quede en el pedido (gotcha 13). Otra nota en
   el lead con el enlace al pedido; en borrador, las condiciones pendientes.
3. Da al SDR el enlace del pedido y dile que lo envíe desde Odoo (Enviar por correo) con el PDF.

## RULES

1. Todo importe sale de `mvi_quote.py price`; nunca calcules a mano ni cambies el precio del pedido.
2. Cualquier cambio en `config/mvi_offer.json` (precio, condición, estación) va seguido de
   `python3 scripts/sync_embeds.py`, que lo copia a /qualify y a las páginas del concepto, y de
   `tests/mvi_smoke.py`, que falla si algo quedó desfasado. Un precio, además, en el producto.
3. Ni Studio, ni campos nuevos, ni plantillas de Odoo creadas sobre la marcha. Los registros de
   `references/odoo_setup.md` solo se crean con un «sí» explícito en el chat.
4. Nunca correo al cliente ni cambios de etapa en el lead: eso lo hace el SDR.

## GOTCHAS

Una línea cada uno; el detalle está en `references/gotchas.md`.
1. Chrome headless no termina en macOS: `render_pdf()` ya lo resuelve, no lo cambies por `run()`.
2. La plantilla trae todas las líneas: borra las no elegidas y relee el total.
3. Sin `plan_id` en el pedido, la plataforma se factura una sola vez.
4. Usuarios: manda el número, no la casilla del formulario.
5. «1 hora funcional al mes» sí se enseña; «33 h» nunca.
6. Precio fijo: un descuento lo convierte en proyecto de comercial.
7. Licencias de Odoo (plan Personalizado con Dyn Cloud): siempre fuera.
8. Veri*factu sin certificado digital no envía nada: sin él no hay arranque de O2C.

## Delegates to

- **MCP Dynapps** (proxy ORM, `88fb5415-...`): lectura del lead, `sale.order`, adjuntos y notas.
  Patrón de conexión y de chatter (`orm_execute_kw message_post`): `references/shared/odoo-connection.md`.
- **quotation-builder-value-based** (mismo plugin): el mapa de metro lee su red, geometría e iconos.
- **utilities:dynapps-brand**: identidad visual de la oferta. Logos oficiales resueltos de `dynapps-brand` y verificados por hash, y reglas de marca en `lint_html()` (gotcha 18).

## Consumed by

- **qualification-form** (`/qualify`): la sección 7 «Encaje MVI» emite la línea que lee el paso 1 y
  propone `/quote-mvi <id>` como siguiente paso cuando es elegible.

## References

| Fichero | Contenido | Cuándo |
|---|---|---|
| `config/mvi_offer.json` | precios, líneas, condiciones, fases y deberes, incluye/no incluye | siempre (lo lee el script) |
| `references/qualify_contract.md` | formato de la línea «MVI:» y reglas de lectura | paso 1 |
| `references/odoo_setup.md` | productos y plantilla que el skill espera en Odoo | paso 3, base nueva |
| `references/gotchas.md` | gotchas en detalle | cuando algo falla |
| `examples/airvolut.json` | deal de ejemplo (O2C, P2P, R2R) | pruebas |
| `docs/mvi/` (raíz del repo) | concepto interno y para clientes (ES/EN) y la entrada pendiente para el catálogo de `doc-sales` | explicar el MVI, cambiar precios o productos |
| `tests/mvi_smoke.py` | condiciones, precio, render, parser y sincronía con el formulario | antes de cada cambio |
