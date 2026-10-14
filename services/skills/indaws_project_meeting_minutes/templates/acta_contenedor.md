# Template: Contenedor del Articulo Acumulativo

Template HTML del wrapper que envuelve todas las actas en el articulo "Actas de Reunion" de Knowledge. Este contenedor se crea una sola vez (con la primera acta) y se actualiza solo la fecha de ultima actualizacion.

---

## HTML del Contenedor

```html
<div style="font-family: Montserrat, Verdana, sans-serif; max-width: 900px; margin: 0 auto;">
  <p style="color: #999; font-size: 14px; margin-bottom: 4px;">
    Proyecto: {nombre_proyecto} | Cliente: {nombre_cliente}
  </p>
  <p style="color: #999; font-size: 12px; margin-bottom: 0;">
    Ultima actualizacion: {DD/MM/YYYY}
  </p>
  <hr style="border-top: 3px solid #6B68FA; margin: 20px 0;">

  {actas_content}

</div>
```

---

## Notas para el LLM

### Placeholders

- `{nombre_proyecto}`: Nombre del proyecto Odoo (project.project.name)
- `{nombre_cliente}`: Nombre del cliente (res.partner.name del partner_id del proyecto)
- `{DD/MM/YYYY}`: Fecha de la ultima acta anadida (actualizar cada vez que se inserta una nueva)
- `{actas_content}`: Todas las entradas de acta concatenadas, separadas por `<hr>` punteado

### Primera creacion

Cuando se crea el articulo por primera vez, `{actas_content}` contiene una sola entrada de acta (la primera).

### Insercion posterior

Al insertar nuevas actas, el contenido se modifica asi:
1. El `{DD/MM/YYYY}` de "Ultima actualizacion" se actualiza con la fecha de la nueva acta
2. La nueva acta se inserta justo despues del `<hr style="border-top: 3px solid #6B68FA">` (el separador del header)
3. Se anade un `<hr>` punteado despues de la nueva acta y antes de las existentes

### Punto de insercion

El regex para encontrar el punto de insercion es:

```
Buscar: <hr style="border-top: 3px solid #6B68FA; margin: 20px 0;">
Insertar DESPUES de este elemento: nueva acta + separador
```

Si no se encuentra este patron exacto, buscar variaciones razonables del `<hr>` con `#6B68FA`.
