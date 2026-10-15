# Reglas del Acta Acumulativa

Patron para gestionar todas las actas de un proyecto en un unico articulo `knowledge.article`, con las mas recientes arriba.

---

## Principio

Todas las actas de reunion de un proyecto se almacenan en el body del articulo "Actas de Reunion" del Knowledge del proyecto. No se crea un articulo hijo por cada reunion.

**Ventajas**:
- Una sola pagina con el historial completo del proyecto
- Facil de consultar: scroll down = viajar en el tiempo
- No genera decenas de subarticulos
- Compatible con busqueda en Knowledge (todo en un body)

---

## Estructura HTML

### Contenedor (wrapper fijo)

El articulo tiene un header fijo que NO se modifica al anadir nuevas actas:

```html
<div style="font-family: Montserrat, Verdana, sans-serif; max-width: 900px; margin: 0 auto;">
  <p style="color: #999; font-size: 14px; margin-bottom: 4px;">
    Proyecto: {nombre_proyecto} | Cliente: {nombre_cliente}
  </p>
  <p style="color: #999; font-size: 12px; margin-bottom: 0;">
    Ultima actualizacion: {DD/MM/YYYY}
  </p>
  <hr style="border-top: 3px solid #6B68FA; margin: 20px 0;">

  <!-- ACTAS AQUI: nuevas arriba, antiguas abajo -->

</div>
```

### Entrada de acta (una por reunion)

Cada acta es un bloque `<div>` autocontenido con atributos `data-` para identificacion:

```html
<div data-acta-fecha="YYYY-MM-DD" data-acta-tipo="{tipo}" style="margin-bottom: 20px;">
  <!-- Contenido del acta (ver template acta_reunion.md) -->
</div>
<hr style="border: 0; border-top: 2px dashed #ddd; margin: 30px 0;">
```

### Separador entre actas

Un `<hr>` punteado separa visualmente cada acta:

```html
<hr style="border: 0; border-top: 2px dashed #ddd; margin: 30px 0;">
```

---

## Algoritmo de Prepend

### Caso 1: Articulo vacio (primera acta)

Si el `body` del articulo "Actas de Reunion" esta vacio o es `false`:

1. Generar el HTML completo: wrapper + primera acta
2. Escribir todo el body con `odoo_write`

### Caso 2: Articulo con contenido (actas previas)

Si el `body` ya tiene contenido:

1. Leer el `body` actual
2. Localizar el punto de insercion: despues del primer `<hr style="border-top: 3px solid #6B68FA`
3. Insertar: nueva acta HTML + separador `<hr>` punteado
4. Actualizar la fecha de "Ultima actualizacion" en el header
5. Escribir el `body` actualizado con `odoo_write`

**Patron de insercion**:

```
ANTES:
<hr solid #6B68FA>        ← header separator
<div acta_antigua_1>      ← acta mas reciente hasta ahora
<hr dashed>
<div acta_antigua_2>

DESPUES:
<hr solid #6B68FA>        ← header separator (sin cambios)
<div acta_nueva>          ← NUEVA acta insertada aqui
<hr dashed>               ← nuevo separador
<div acta_antigua_1>      ← antes era la mas reciente
<hr dashed>
<div acta_antigua_2>
```

### Caso 3: HTML corrupto o formato inesperado

Si el body no tiene el formato esperado (no se encuentra el punto de insercion):

1. Envolver el contenido existente en el wrapper estandar
2. Prepend la nueva acta
3. Advertir al usuario que el formato se ha normalizado

---

## Identificacion de Actas

Cada acta tiene atributos `data-` que permiten identificarla:

- `data-acta-fecha`: Fecha de la reunion en formato YYYY-MM-DD
- `data-acta-tipo`: Tipo de reunion (seguimiento, kick-off, validacion, formacion, analisis)

Esto permite:
- Detectar duplicados (misma fecha + mismo tipo)
- Buscar actas especificas por fecha o tipo
- Referenciar actas desde otros documentos

### Prevencion de duplicados

Antes de insertar una nueva acta, verificar que no exista ya una con la misma fecha Y tipo:
- Buscar `data-acta-fecha="{fecha}"` Y `data-acta-tipo="{tipo}"` en el body
- Si existe: PREGUNTAR al usuario si quiere reemplazarla o anadir una segunda acta del mismo dia

**Excepcion**: Puede haber multiples reuniones del mismo dia si son de tipos diferentes (ej: una reunion de seguimiento y una de analisis el mismo dia).

---

## Limites y Advertencias

### Tamano del articulo

- **Tamano recomendado**: Hasta 50 actas (aproximadamente 200-500 KB)
- **Advertencia a 100 actas**: Informar al usuario que el articulo esta creciendo y puede afectar al rendimiento de Knowledge
- **No hay limite duro**: Odoo Knowledge soporta bodies grandes, pero la experiencia de usuario se degrada con cientos de actas

### Rendimiento

- Leer el body completo antes de cada insercion (necesario para prepend)
- Escribir el body completo despues de cada insercion
- Con actas acumuladas, cada operacion mueve mas datos

---

## Ejemplo de Body Completo

```html
<div style="font-family: Montserrat, Verdana, sans-serif; max-width: 900px; margin: 0 auto;">
  <p style="color: #999; font-size: 14px; margin-bottom: 4px;">
    Proyecto: Hotel Portomar | Cliente: Hotel Portomar S.L.
  </p>
  <p style="color: #999; font-size: 12px; margin-bottom: 0;">
    Ultima actualizacion: 18/03/2026
  </p>
  <hr style="border-top: 3px solid #6B68FA; margin: 20px 0;">

  <div data-acta-fecha="2026-03-18" data-acta-tipo="seguimiento" style="margin-bottom: 20px;">
    <!-- Acta de seguimiento del 18/03 -->
  </div>
  <hr style="border: 0; border-top: 2px dashed #ddd; margin: 30px 0;">

  <div data-acta-fecha="2026-03-14" data-acta-tipo="analisis" style="margin-bottom: 20px;">
    <!-- Acta de analisis del 14/03 -->
  </div>
  <hr style="border: 0; border-top: 2px dashed #ddd; margin: 30px 0;">

  <div data-acta-fecha="2026-03-12" data-acta-tipo="kick-off" style="margin-bottom: 20px;">
    <!-- Acta de kick-off del 12/03 -->
  </div>

</div>
```
