# Template: Nota Interna de Configuracion

> Template HTML para `message_post` en la tarea. Se usa para documentar items CONFIG simples.
> Todas las configs de una ejecucion se agrupan en UNA SOLA nota.

## Parametros del message_post

```
model: project.task
method: message_post
args: [task_id]
kwargs:
  body: <HTML generado>
  message_type: "comment"
  subtype_xmlid: "mail.mt_note"
```

**IMPORTANTE**: `subtype_xmlid: "mail.mt_note"` = nota interna (NO notifica seguidores).

## Template HTML

```html
<p><strong>⚙️ Configuracion aplicada</strong> — <em>{PRODUCCION|TEST}</em></p>
<ul>
  <li>
    <strong>{nombre_item_1}</strong>: {descripcion_breve}
    <br/><small>Ruta: {Settings > Seccion > Opcion}</small>
  </li>
  <li>
    <strong>{nombre_item_2}</strong>: {descripcion_breve}
    <br/><small>Ruta: {Settings > Seccion > Opcion}</small>
  </li>
  <!-- ... mas items ... -->
</ul>
<p><small>Configurado por odoo-configurator | {fecha}</small></p>
```

## Ejemplo Completo

```html
<p><strong>⚙️ Configuracion aplicada</strong> — <em>TEST</em></p>
<ul>
  <li>
    <strong>Multi-moneda</strong>: Activada gestion de multiples monedas. Monedas activas: EUR, USD, GBP.
    <br/><small>Ruta: Settings > Accounting > Currencies > Multi-Currencies</small>
  </li>
  <li>
    <strong>Tarifas</strong>: Activadas listas de precios con modo "Multiple prices per product".
    <br/><small>Ruta: Settings > Sales > Pricing > Pricelists</small>
  </li>
  <li>
    <strong>Unidades de medida</strong>: Activadas para permitir conversion entre unidades.
    <br/><small>Ruta: Settings > Sales > Product Catalog > Units of Measure</small>
  </li>
</ul>
<p><small>Configurado por odoo-configurator | 2026-03-14</small></p>
```
