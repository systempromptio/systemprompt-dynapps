# Formato de entrega

Lo que Víctor recibe en el chat. Se pega en DatoCMS campo a campo, así que todo lo
que no sea texto pegable va fuera de la columna «propuesto».

## Cabecera

```
## /es/services · Servicios Odoo
Extraído el <fecha> · 42 bloques · <n> keywords posicionadas · [requiere aprobación de dirección — solo si toca /es, <head> o pie]
Keyword principal: «servicios odoo» (vol. <n>/mes, hoy sin posición)
Lint: <n> bloqueantes, <n> avisos
```

## Tabla de campos

Una fila por campo que cambia. Los que no cambian **no se listan**: una tabla de 40
filas donde 30 dicen «igual» no se lee.

| id | campo DatoCMS | actual | propuesto | por qué |
|---|---|---|---|---|
| — | `seo_title` (42→57) | Dynapps España \| Socio Implementación Odoo | Servicios Odoo: implantación, soporte y hosting \| Dynapps | Anexo B del plan (C12); «Socio»→«Partner»; title repetido con `/es/contact` |
| B16 | `card_title` | Tren | Formación | Calco de «Train» |

Reglas de la tabla:

- **El propuesto se pega tal cual.** Sin comillas de adorno, sin `[...]`, sin
  comentarios dentro de la celda. Si hace falta explicar, va en «por qué».
- En `seo_title` y `seo_description`, los caracteres entre paréntesis: `(42→57)`.
  Si el propuesto se pasa del límite, no se entrega: se recorta antes.
- Los textos largos (`section_body` de 300+ caracteres) van fuera de la tabla, en
  bloques aparte con su id como encabezado, para que se puedan seleccionar enteros.

## Bloques largos

```
### B05 · `section_body` · sección 1 «Los pilares de un vistazo»

**Actual** (302 car.)
> …

**Propuesto** (287 car.)
> …

Por qué: «implementación» → «implantación» (una palabra por concepto); primera
frase responde al H2; enlace interno a /es/services/odoo-implementation.
```

## Lo que falta

Va después de los campos y antes de lo técnico, porque son propuestas nuevas y no
correcciones. Uno o dos por página, no una lista de deseos. Cada una dice si se pega o
si es un encargo, que es lo que decide si se puede hacer hoy:

```
### Tabla · «Odoo frente a SAP, Dynamics y NetSuite» · tras B30
**Se pega.** El párrafo ya afirma la comparación sin sostenerla.

| Criterio | Odoo | SAP Business One | Microsoft Dynamics 365 | NetSuite |
|---|---|---|---|---|
| … | … | … | … | … |

Fuente de cada celda: <de dónde sale>. Frase que la precede: «<una línea>».
```

```
### Cita · «/es/cases/plastimyr» · tras el primer párrafo
**Encargo.** Pedir a <nombre, cargo> dos frases sobre <qué>. Con consentimiento escrito.
No se redacta un ejemplo: una cita inventada es una declaración falsa de un tercero.
```

```
### Imagen · «/es/services», sección de fases
**Encargo.** Qué debe mostrar: … · Fichero: `fases-implantacion-odoo.webp`
Alt: «…» · Quién la produce: marketing / captura de Odoo (skill odoo-screenshot-generator)
```

Los textos alternativos de imágenes que **ya están** en la página van en la tabla de
campos, con una marca cuando no se han podido ver: `alt: HAY QUE VER LA FOTO`. Y se
avisa de que el alt se edita en el asset de DatoCMS, no en la página: cambiarlo afecta a
todas las páginas que usen esa imagen.

## Esto no se arregla pegando

Lista corta, al final, con destinatario. Es lo que la copia no puede resolver:

```
- No hay `rel=canonical` en esta página (T8 del plan) → equipo web, layout Next.js
- El title de /es/contact es el de esta página → revisar la colección en DatoCMS
- Faltan 12 alt de imagen (C18) → se pueden pegar, pero el campo está en el asset
```

## Cierre

Una línea con lo que queda pendiente y la siguiente página del lote. Nada de
resúmenes de lo que ya está en la tabla.
