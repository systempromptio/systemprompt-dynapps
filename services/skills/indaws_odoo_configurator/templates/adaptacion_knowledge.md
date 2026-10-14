# Template: Articulo Knowledge para Adaptaciones

> Template HTML para crear articulos `knowledge.article` bajo "Adaptaciones" del proyecto.
> Se usa para documentar cambios Studio/AUTO y configuraciones complejas.

## Parametros del knowledge.article

```
model: knowledge.article
method: create
values:
  name: "{Nombre descriptivo de la adaptacion}"
  parent_id: {id_articulo_adaptaciones}
  body: <HTML generado>
```

## Template HTML

```html
<h2>{Nombre de la Adaptacion}</h2>

<h3>Descripcion</h3>
<p>{Que se ha hecho y por que. Referencia al analisis funcional si existe.}</p>

<h3>Detalles Tecnicos</h3>
<table>
  <thead>
    <tr>
      <th>Elemento</th>
      <th>Detalle</th>
    </tr>
  </thead>
  <tbody>
    <tr><td>Modelo</td><td><code>{modelo_odoo}</code></td></tr>
    <tr><td>Tipo</td><td>{Campo / Server Action / Automated Action / Pagina}</td></tr>
    <tr><td>Nombre tecnico</td><td><code>{nombre_tecnico}</code></td></tr>
    <tr><td>Configuracion</td><td>{detalles: tipo de campo, opciones selection, trigger, condicion...}</td></tr>
  </tbody>
</table>

<h3>Como Funciona</h3>
<p>{Explicacion desde la perspectiva del usuario final. Que ve, que hace, que resultado obtiene.}</p>

<p><em>[SCREENSHOT: {identificador_para_captura}]</em></p>

<h3>Entorno</h3>
<p>Configurado en: <strong>{PRODUCCION|TEST}</strong> | Fecha: {fecha}</p>
```

## Ejemplo Completo

```html
<h2>Campo "Referencia Cliente" en Pedidos de Venta</h2>

<h3>Descripcion</h3>
<p>Se ha creado un campo para almacenar la referencia interna del cliente en cada pedido de venta. Esto permite al equipo comercial identificar rapidamente el pedido segun la nomenclatura del cliente.</p>

<h3>Detalles Tecnicos</h3>
<table>
  <thead>
    <tr>
      <th>Elemento</th>
      <th>Detalle</th>
    </tr>
  </thead>
  <tbody>
    <tr><td>Modelo</td><td><code>sale.order</code></td></tr>
    <tr><td>Tipo</td><td>Campo</td></tr>
    <tr><td>Nombre tecnico</td><td><code>x_studio_ref_cliente</code></td></tr>
    <tr><td>Configuracion</td><td>Tipo: Char, Requerido: No, Visible en: formulario (despues de client_order_ref) y lista</td></tr>
  </tbody>
</table>

<h3>Como Funciona</h3>
<p>Al crear o editar un pedido de venta, el usuario encontrara un nuevo campo "Referencia Cliente" justo debajo de la referencia del cliente existente. Este campo es de texto libre y permite introducir cualquier codigo o referencia que el cliente utilice internamente para identificar el pedido.</p>

<p><em>[SCREENSHOT: sale_order_ref_cliente_field]</em></p>

<h3>Entorno</h3>
<p>Configurado en: <strong>TEST</strong> | Fecha: 2026-03-14</p>
```

## Notas

- Los placeholders `[SCREENSHOT: ...]` se pueden procesar posteriormente con `odoo-screenshot-generator`
- Si hay multiples cambios relacionados (ej: 3 campos en el mismo modelo), agrupar en un solo articulo
- El nombre del articulo debe ser descriptivo y comprensible para el consultor y el cliente
