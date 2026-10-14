---
name: indaws-report-designer-author
version: 0.1.0
effort: low
model: sonnet
description: >-
  Guía funcional para crear y editar informes con el editor visual de Odoo
  indaws_report_designer (tipo Google Docs, campos arrastrables, salida PDF vía
  QWeb). Recetas: campos, tablas de registros, columnas, bordes, condiciones,
  agrupaciones, cabecera/pie, QR, imágenes, fórmulas, texto traducible, saltos de
  página, vista previa y publicación. Para consultor/usuario que NO toca código.
  NOT desarrollar el addon (→ report-designer-developer); NOT plantillas DOCX
  (→ report-docx-template).
  Triggers: "crear un informe con el editor visual", "cómo pongo un campo en el
  informe", "editor de informes Odoo", "diseñar informe sin código", "tabla de
  líneas en el informe", "vista previa del informe", "generar el informe".
allowed-tools: [Read, AskUserQuestion]
---

# Report Designer Author v0.1

Guía **funcional** para diseñar informes con el editor visual `indaws_report_designer`
(Odoo v18). El editor funciona como un **Google Docs**: escribes texto y arrastras/insertas
**campos de Odoo** que se rellenan al imprimir. No hace falta tocar código ni XML.

> Salida = **PDF** (motor QWeb). Si topas con un límite del editor o algo falla en el
> render, es tema de desarrollo → deriva a `report-designer-developer`.
>
> Todo ocurre dentro de una sesión Odoo autenticada (contrato: `references/shared/odoo-connection.md`); esta skill no ejecuta código ni RPC, solo guía el uso del editor.

## Conceptos base

- **Campo**: un dato de Odoo (nombre del cliente, total…). Aparece como una "pastilla"
  (chip) en el documento; al imprimir se sustituye por el valor real.
- **Tabla de registros**: una tabla que se repite por cada línea (las líneas de un
  presupuesto, factura…). Es el "loop".
- **Documento vs línea**: los campos del documento (cliente, fecha) van en el cuerpo;
  los campos de las líneas solo tienen sentido **dentro** de la tabla de registros.
- **Menú contextual**: **clic derecho sobre el papel** abre todas las acciones. También
  hay `＋ Insertar ▾` en la barra y el comando **"/"** (escribe `/campo`, `/tabla`, `/gráfico`).

## Arrancar

1. Crear un registro **`indaws.report.designer`**: elegir el **modelo** (sobre qué se
   imprime: `sale.order`, `account.move`, …) y un **nombre**.
2. Abrir el **editor** (botón del formulario).
3. Elegir un **registro de vista previa** (arriba) para ver datos reales mientras diseñas.

## Recetas

Ver **`references/recipes.md`** para el paso a paso de cada elemento. Resumen del índice:

| Quiero… | Cómo (resumen) |
|---------|----------------|
| Poner un dato del documento | `＋ Insertar ▸ Campo` (o `/campo`) → elegir campo (drill-down m2o: `partner_id ▸ name`) |
| Tabla de líneas | `＋ Insertar ▸ Tabla de registros` → elegir el one2many (order_line…) |
| Campo dentro de la tabla | Pon el cursor en una celda de la tabla → `Campo` (el ámbito pasa a la línea) |
| Añadir/quitar columna | Clic derecho en la tabla → Añadir/Quitar columna |
| Combinar celdas / filas | Clic derecho → Combinar (o Añadir/Quitar fila) |
| Bordes | Panel Celda/Tabla → pluma (grosor/color) + clic en un lado o "todos" |
| Ocultar columna según condición | Clic derecho en celda → Condición de columna… |
| Ocultar fila/sección | Panel Fila → Condición de fila |
| Agrupar y subtotales | Pestaña **Agrupar** de la tabla → campo por el que agrupar |
| Cabecera / pie | Pinchar en la zona de cabecera/pie y editar; o modo (custom/odoo/none) |
| QR o código de barras | `＋ Insertar ▸ Código` → tipo (QR/Code128/EAN13) + valor |
| Nº de línea | Dentro de la tabla → `＋ Insertar ▸ Nº de línea` |
| Imagen | `＋ Insertar ▸ Imagen` (subir) o campo imagen |
| Fórmula / cálculo | `＋ Insertar ▸ Expresión` (p.ej. `sum(l.price_subtotal for l in doc.order_line)`) |
| Función guardada | `＋ Insertar ▸ Función` (las define un admin) |
| Texto en varios idiomas | `＋ Insertar ▸ Texto traducible` |
| Salto de página | `＋ Insertar ▸ Salto de página` |
| Columnas / caja | `＋ Insertar ▸ Columnas` / `Caja` |
| Formato de un campo (moneda, fecha…) | Clic en el chip → panel **Propiedades** → Formato/Widget |
| Ocultar si vacío / mostrar-si / estilo-si | Panel **Propiedades** del chip |

## Publicar y gestionar

- **Vista previa** (paged.js) — aproximada, para ir viendo el diseño paginado.
- **🖨 Imprimir prueba** — genera el **PDF real** del registro elegido. Fíate de este, no de la vista previa.
- **Generar** — crea el `ir.actions.report` de verdad → el informe aparece como opción de
  impresión en Odoo para ese modelo.
- **Plantillas** — Guardar como plantilla / Duplicar / aplicar una plantilla (blank, carta, líneas).
- **Historial** — `↶ Historial ▾` para volver a una versión anterior.
- **QWeb fuente** (avanzado) — pestaña QWeb para ver/editar el código generado.

> **¿Prefieres editar tipo Word (DOCX)?** Hay un flujo hermano en DOCX: la plantilla se **genera**
> desde un mockup (→ `report-docx-template`) y se **edita tipo Word** en el editor OnlyOffice del
> módulo `indaws_docx_designer` (menú *DOCX Designer*). Úsalo cuando quieras maquetar libremente en
> Word; este editor QWeb es para PDF vía QWeb.

## GOTCHAS

1. **Campo de línea fuera de la tabla → sale vacío o no aparece.** Los campos de las líneas
   (`price_subtotal`, cantidad…) solo funcionan **dentro** de la tabla de registros. Para
   insertarlos, pon el cursor en una celda de la tabla antes de insertar.
2. **La vista previa NO es el PDF final.** paged.js usa el navegador; el PDF real usa otro
   motor (wkhtml). Diferencias de maquetación son normales → valida con **Imprimir prueba**.
3. **No esperes diseños tipo Flexbox/Grid.** El motor de PDF es antiguo: para poner cosas en
   columnas usa **Columnas** o una **tabla de maquetación**, no cajas flotantes modernas.
4. **Elige un registro de vista previa** con datos representativos; uno vacío o atípico
   engaña sobre cómo quedará.
5. **Un campo m2o sin "entrar" muestra algo feo** (p.ej. el ID). Usa el **drill-down**:
   `partner_id ▸ name`, no `partner_id` a secas.
6. **"Ocultar si vacío"** trata el 0 como vacío (0 es "falsy"): útil para importes, ojo si
   un 0 legítimo debe verse.

## NEVER

- Editar la vista/plantilla del informe por XML a mano cuando el editor puede hacerlo —
  el editor es la fuente; editar el XML por fuera se pierde al regenerar.
- Confiar en la vista previa para dar por bueno el informe → siempre Imprimir prueba.

## ALWAYS

- Fijar el **modelo** correcto al crear el designer (no se cambia fácil después).
- Insertar campos de línea **dentro** de la tabla de registros.
- Validar el resultado con **Imprimir prueba** (PDF real) antes de **Generar**.

## Delegates to

| Skill | Suite | Cuándo |
|-------|-------|--------|
| `report-designer-developer` | developer | El editor no llega a algo, falla el render, o hace falta una función/helper nuevo (código) |
| `report-docx-template` | consultant | El informe debe ser un **DOCX/Word** editable (otro flujo, motor docxtpl) |
| `odoo-configurator` | consultant | Dar de alta el informe / permisos / menús alrededor del report |

## Consumed by

_(ninguna por ahora — skill de cara al usuario)_

## REFERENCES

- `references/recipes.md` — Paso a paso de cada elemento insertable y de publicación
