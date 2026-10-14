# Kick-Off Presentation - Workflow Detail

## Paso 1: Lectura de Datos de Odoo

### a) Datos del sale.order

Campos a leer del sale.order:
- name (referencia)
- partner_id (cliente)
- x_studio_resumen (resumen del proyecto)
- x_studio_toma_de_datos (toma de datos completa)
- x_studio_condiciones_proyecto (condiciones contractuales)
- x_studio_migracion_datos (datos a migrar)
- x_studio_mantenimiento (condiciones de soporte)
- amount_untaxed (importe sin impuestos)
- order_line (lineas del pedido)
- project_id (proyecto vinculado)
- x_studio_planning_initial_date (fecha inicio planificacion)

Lee tambien las lineas del pedido (`sale.order.line`) para identificar:
- Horas de implantacion vendidas (producto tipo servicio con horas)
- Modulos Indaws Core contratados (productos con prefijo [CORE])
- Fase de analisis (producto "Analisis de proyecto")

### b) Datos del cliente (res.partner)

Campos: name, street, city, state_id, zip, country_id, phone, email, website, vat

### c) Datos del equipo (hr.employee)

Campos: name, job_title, work_email, work_phone, department_id, image_128 (base64)

### d) Datos del proyecto (project.project)

Campos: name, date_start, date, xx_golive_date, tag_ids

## Paso 2: Generacion de Estructura (con Validacion)

Consultar `./references/estructura_kickoff.md` para la definicion detallada de cada slide.

Presentar al usuario resumen de 20 slides en 5 bloques:

### BLOQUE 1 - CONEXION (slides 1-4)
Portada, Agenda, Indaws en cifras, Equipo Indaws

### BLOQUE 2 - ALINEACION DEL PROYECTO (slides 5-9)
Equipo del cliente, Vision del proyecto, Aplicaciones Odoo, Modulos Indaws Core, Alcance

### BLOQUE 3 - METODOLOGIA (slides 10-13)
Nuestra metodologia, El Analisis, Plan de comunicacion, Gestion de cambios

### BLOQUE 4 - COMPROMISOS Y FECHAS (slides 14-17)
Timeline, Calendario sesiones, Reglas del juego, Impacto de retrasos

### BLOQUE 5 - PROXIMOS PASOS Y CIERRE (slides 18-20)
Client requests, Proximos pasos, Cierre + Q&A

DETENTE y espera aprobacion del usuario.

## Paso 3: Generacion de los 3 Entregables

### Entregable 1: Presentacion HTML

- Template: `./templates/presentacion_kickoff.md`
- Marca: Colores y tipografia de dynapps-brand (Blue Space #1C265D, Blue Lilac #6B68FA, Warm Yellow #E5B92B, Light Sky #8AC2DB, Montserrat)
- Navegacion: Sistema de slides con scroll-snap + teclado + dots laterales
- Imagenes del equipo: Embebidas en base64 desde hr.employee.image_128
- Sin dependencias externas excepto Google Fonts
- Guardar como: `kickoff_{nombre_cliente_slug}_{fecha}.html`

### Entregable 2: Knowledge Article

Estructura en Odoo Knowledge:

```
{Articulo raiz del proyecto}
+-- Kick-Off                    <- Contenido del kick-off
+-- Interno                     <- Contenedor (crear si no existe)
    +-- Guion Kickoff           <- Guion del consultor
```

**Articulo "Kick-Off"**: Buscar si existe, crear o actualizar. Adjuntar HTML de presentacion como ir.attachment.

**Contenedor "Interno"**: Buscar si existe, crear si no.

**Guion Kickoff**: Buscar si existe, crear o actualizar. Conversion Markdown a HTML con estilos inline (Odoo no soporta CSS externo).

### Entregable 3: Guion del consultor

- Tiempos realistas: Total 60-90 minutos
- Preguntas anticipadas: Minimo 2-3 por bloque tematico
- Respuestas sugeridas: Concretas, no evasivas
- Transiciones naturales: Frases puente entre slides
