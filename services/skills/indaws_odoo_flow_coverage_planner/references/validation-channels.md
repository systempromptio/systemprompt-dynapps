# Canales de validacion: IA (API, token-eficiente) vs Humano (visual, token-pesado)

**Principio rector de esta skill.** Solo se generan/incluyen validaciones que la IA + el
`odoo-flow-tester` pueden hacer **por API con coste de tokens bajo** (asserts sobre datos/estado).
Las validaciones **visuales** (leer PDFs renderizados, capturas, UI/UX, maquetacion) son
token-pesadas para la IA y **mas rapidas y baratas si las hace un humano** -> se derivan a
`human_visual_checklist.md` y NO se auto-validan.

Regla de bolsillo: **si para validar hay que MIRAR (PDF, captura, navegador) -> Humano.**
**Si se valida con un assert sobre un valor/estado via API -> IA.**

## Tabla de decision

| Tipo de validacion | Canal | Por que |
|--------------------|-------|---------|
| Transicion de estado (draft->sale->done) | **IA** | assert de campo via API, coste ~nulo |
| Valores calculados (amount_total, impuestos, descuentos) | **IA** | assert numerico |
| Campo computed/required custom (`x_studio_*`) | **IA** | assert de valor |
| Registro en cascada creado (picking/factura) | **IA** | `detect_cascade` + assert |
| Integridad de datos (relaciones, counts, secuencias) | **IA** | search/read |
| Permisos / record rules | **IA** | flow-tester `verify_data.py` |
| Menus que abren sin error JS/500 | **IA** | flow-tester `playwright_smoke` devuelve lista de errores, NO pixeles |
| Que un informe se GENERA sin excepcion (render OK ≠ aspecto OK) | **IA** | llamada al report + status 200 / no traceback |
| **Maquetacion / alineacion de informes PDF** | **Humano** | requiere LEER el PDF renderizado (imagen) = muchos tokens |
| **Fidelidad visual (p.ej. v17 vs v19)** | **Humano** | comparacion visual |
| **CSS / responsive / branding / fuentes** | **Humano** | pixeles |
| **eCommerce: efecto visual de variante, posicion de un bloque** | **Humano** | render en navegador |
| **Render de email (HTML visual)** | **Humano** | imagen |
| **Orden/posicion de columnas en informe (ej. columna "Pos.")** | **Humano** | layout visual |

## Zona gris: partir la validacion (informes)

Un informe tiene dos cosas: (a) **DATOS** correctos y (b) **ASPECTO** correcto. Se separan:

- **(a) DATOS -> IA.** En vez de leer el PDF, asertar los campos del registro/modelo que
  alimentan el informe (que existen las lineas de seccion, que `amount_total` cuadra, que el
  descuento aplica). Y verificar que el report **se genera sin excepcion**. Token-eficiente.
- **(b) ASPECTO -> Humano.** Alineacion, columna "Pos.", saltos de pagina, tipografia, encaje
  del diseno. Va a `human_visual_checklist.md` con instrucciones precisas.

Asi se cubre la **funcionalidad** del informe sin que la IA lea el PDF.

## Que NO debe hacer la IA (token-pesado -> siempre al checklist humano)

- Abrir y leer PDFs renderizados para juzgar maquetacion.
- Leer capturas de pantalla para validar diseno.
- Generar+interpretar screenshots para fidelidad visual.
- Inspeccionar visualmente paginas web (posicion, estilo, UX).

Estos items se vuelcan en `human_visual_checklist.md`. **OBLIGATORIO por item** (no opcional):
- **Ruta**: como llegar exactamente — menu completo (`Ventas → Pedidos → ...`) o URL
  (`http://host/odoo/sales`, o `/odoo/action-<id>` si solo se conoce la accion).
- **Como probar**: pasos numerados concretos para reproducir lo que hay que mirar
  (abrir registro X → boton Imprimir → abrir PDF; o /shop → producto con variantes → cambiar variante).
- **Criterio**: que debe verse para dar OK.
- **Dato de ejemplo**: registro/URL concretos (los `[TEST-...]` del flow-tester o uno real).

Nunca dejar Ruta/Como probar en blanco ni generico. Si no se conoce la ruta, derivarla del menu
del modelo (`ir.ui.menu` cuyo action apunta al modelo) o de la accion (`/odoo/action-<id>`). El
objetivo es que el humano tarde segundos en llegar y saber que hacer.

## Como clasificar en el Paso 2

Por cada funcionalidad candidata, etiquetar `channel = ai | human | split`:
- `ai` -> entra en la generacion de flujos (Paso 3) como assert API.
- `human` -> entra SOLO en el checklist humano (NO genera flujo, NO consume tokens de IA).
- `split` -> la parte de datos genera flujo (ai) y la parte de aspecto va al checklist (human).

Nota: el flow-tester puede generar capturas (delega en `odoo-screenshot-generator`). Esta skill
**no** solicita capturas para que la IA las interprete; si acaso, las adjunta como evidencia para
el humano. La interpretacion visual es del humano.
