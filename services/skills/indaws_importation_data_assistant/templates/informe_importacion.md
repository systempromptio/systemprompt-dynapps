# Template: Informe de Importacion (Nota Interna)

> Template HTML para `message_post` en la tarea. Nota interna con resumen de la importacion.

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
<p><strong>📥 Importacion realizada</strong> — <em>{PRODUCCION|TEST}</em></p>
<table style="border-collapse:collapse;width:100%;margin:8px 0;">
  <thead>
    <tr style="background:#f5f5f5;">
      <th style="border:1px solid #ddd;padding:6px;text-align:left;">Metrica</th>
      <th style="border:1px solid #ddd;padding:6px;text-align:left;">Valor</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td style="border:1px solid #ddd;padding:6px;">Modelo</td>
      <td style="border:1px solid #ddd;padding:6px;"><code>{modelo}</code></td>
    </tr>
    <tr>
      <td style="border:1px solid #ddd;padding:6px;">Archivo origen</td>
      <td style="border:1px solid #ddd;padding:6px;">{nombre_archivo}</td>
    </tr>
    <tr>
      <td style="border:1px solid #ddd;padding:6px;">Total filas</td>
      <td style="border:1px solid #ddd;padding:6px;">{total_filas}</td>
    </tr>
    <tr>
      <td style="border:1px solid #ddd;padding:6px;">Importados OK</td>
      <td style="border:1px solid #ddd;padding:6px;color:green;"><strong>{importados_ok}</strong></td>
    </tr>
    <tr>
      <td style="border:1px solid #ddd;padding:6px;">Errores</td>
      <td style="border:1px solid #ddd;padding:6px;{color_errores}">{errores}</td>
    </tr>
    <tr>
      <td style="border:1px solid #ddd;padding:6px;">Duplicados eliminados</td>
      <td style="border:1px solid #ddd;padding:6px;">{duplicados}</td>
    </tr>
    <tr>
      <td style="border:1px solid #ddd;padding:6px;">External ID prefix</td>
      <td style="border:1px solid #ddd;padding:6px;"><code>{external_id_prefix}</code></td>
    </tr>
  </tbody>
</table>
{seccion_correcciones}
{seccion_errores}
<p><small>Importado por importation-data-assistant | {fecha}</small></p>
```

## Secciones condicionales

### Seccion correcciones (si hay)
```html
<p><strong>Correcciones aplicadas:</strong></p>
<ul>
  <li>{N} paises normalizados</li>
  <li>{N} telefonos con prefijo añadido</li>
  <!-- ... -->
</ul>
```

### Seccion errores (si hay)
```html
<p><strong>Errores ({N}):</strong></p>
<table style="border-collapse:collapse;width:100%;margin:8px 0;">
  <thead>
    <tr style="background:#fff3f3;">
      <th style="border:1px solid #ddd;padding:4px;">Fila</th>
      <th style="border:1px solid #ddd;padding:4px;">Campo</th>
      <th style="border:1px solid #ddd;padding:4px;">Error</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td style="border:1px solid #ddd;padding:4px;">{fila}</td>
      <td style="border:1px solid #ddd;padding:4px;">{campo}</td>
      <td style="border:1px solid #ddd;padding:4px;">{mensaje_error}</td>
    </tr>
  </tbody>
</table>
```
