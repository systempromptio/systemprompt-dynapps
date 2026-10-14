# Template: Validacion Cliente (Client Request)

> Template HTML para actualizar la descripcion o notas del `x_client_request`.
> Lenguaje de negocio, sin tecnicismos. El objetivo es que el cliente pueda validar la importacion.

## Uso

Actualizar `x_description` del `x_client_request` vinculado a la importacion:
```
model: x_client_request
method: write
args: [[request_id], {"x_description": "<HTML generado>"}]
```

## Template HTML

```html
<h3>Resultado de la importacion</h3>

<p>Hemos procesado el archivo <strong>{nombre_archivo}</strong> que nos proporcionaste. A continuacion te resumimos el resultado:</p>

<h4>Datos importados</h4>
<ul>
  <li><strong>Tipo de datos:</strong> {descripcion_tipo} (ej: "Contactos de clientes y proveedores")</li>
  <li><strong>Registros importados:</strong> {importados_ok} de {total_filas}</li>
  {li_errores}
</ul>

<h4>Ajustes realizados</h4>
<p>Para garantizar la calidad de los datos, hemos aplicado las siguientes correcciones automaticas:</p>
<ul>
  {lista_correcciones}
</ul>

{seccion_no_importados}

<h4>Validacion</h4>
<p>Te pedimos que revises los datos importados en el sistema y nos confirmes que todo esta correcto. En particular:</p>
<ul>
  <li>Verifica que los {descripcion_tipo_plural} aparecen correctamente</li>
  <li>Comprueba que los datos principales (nombre, contacto, etc.) son correctos</li>
  {li_validacion_extra}
</ul>

<p>Si detectas algun dato incorrecto o faltante, por favor indicanoslo para corregirlo.</p>
```

## Secciones condicionales

### li_errores (si hay errores)
```html
<li><strong>Registros no importados:</strong> {errores} — ver detalle abajo</li>
```

### lista_correcciones (ejemplos)
```html
<li>Se han unificado los nombres de paises (por ejemplo, "Spain" y "Espana" se han corregido a "España")</li>
<li>Se han añadido los prefijos telefonicos internacionales donde faltaban</li>
<li>Se han eliminado {N} registros duplicados</li>
<li>Se han corregido los formatos de fecha al estandar del sistema</li>
```

### seccion_no_importados (si hay errores)
```html
<h4>Datos no importados</h4>
<p>Los siguientes {N} registros no se pudieron importar por errores en los datos:</p>
<ul>
  <li>Fila {fila}: {descripcion_simple_del_error}</li>
  <!-- ... -->
</ul>
<p>Si necesitas que estos datos se importen, proporcionanos la informacion corregida.</p>
```

### li_validacion_extra (segun tipo de datos)
```html
<!-- Para contactos -->
<li>Verifica que las direcciones y datos fiscales son correctos</li>
<!-- Para productos -->
<li>Verifica que los precios y categorias son correctos</li>
<!-- Para facturas -->
<li>Verifica que los importes y fechas coinciden con tus registros</li>
```

## Reglas de redaccion

1. **NUNCA** usar terminologia tecnica (modelos, campos, XML IDs, API)
2. **SIEMPRE** describir en terminos de negocio ("contactos", "productos", no "res.partner")
3. **NUNCA** mencionar errores tecnicos literales del sistema
4. **SIEMPRE** ofrecer la opcion de corregir y reimportar
5. Tono: profesional, claro, orientado a la accion
