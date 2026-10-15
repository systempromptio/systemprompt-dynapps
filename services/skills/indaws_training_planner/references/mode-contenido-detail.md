# Training Planner -- Mode CONTENIDO Detail

## Paso C0: Recopilacion de Datos (CRITICO)

### Preguntas obligatorias:

1. **ID de la tarea en Odoo**: Pide el ID de la tarea del proyecto.
   - Lee la tarea via `odoo-pilot`: descripcion + mensajes del chatter
   - Extrae: app/funcionalidad, perfil asistentes, contexto proyecto, campos custom, GAPs

2. **Fuente de informacion completa**: Pregunta:
   > "He leido la tarea. Esta toda la informacion aqui o necesitas adjuntar el documento de analisis funcional?"
   Si falta info, pedir documento de analisis funcional (Markdown).

3. **Scope de la formacion**: Pregunta que es la formacion:
   - App completa (ej: "Ventas")
   - Funcionalidad concreta (ej: "Aprobacion de pedidos de compra")
   - Modulo custom (ej: "Modulo de gestion de propiedades")
   - Area transversal (ej: "Facturacion electronica")

4. **Perfil de los asistentes**: Si no esta claro, preguntar.
   Consulta `./references/perfiles_usuario.md`.

5. **Credenciales del Odoo de prueba** (OBLIGATORIO):
   > "Necesito las credenciales del Odoo de prueba: URL, base de datos, usuario y contrasena."
   - La formacion SIEMPRE se prepara sobre entorno de prueba con datos reales
   - Sin esto no se pueden generar ejercicios realistas ni screenshots

6. **Formato y duracion** (opcional):
   - Formato: presencial / remoto / mixto (default: presencial)
   - Duracion maxima por sesion (default: 3h)

**DETENTE** y espera.

## Paso C1: Analisis de Necesidades Formativas

1. **Identificar temario base**: Consulta `./references/contenido_por_app.md`.

2. **Adaptar al proyecto**: Cruzar temario con la tarea/analisis:
   - Campos Studio/custom -> incluir en ejercicios
   - GAPs -> explicar workarounds
   - Modulos Indaws Core -> funcionalidad adicional
   - Integraciones -> mencionar si afectan al flujo

3. **Adaptar al perfil**: Consulta `./references/perfiles_usuario.md`:
   - Operarios: 100% practico, solo lo que usan a diario
   - Responsables: flujo completo + informes + aprobaciones
   - Direccion: dashboards, KPIs, vision ejecutiva
   - Contables: detalle tecnico, conciliacion, cierres
   - Admin sistema: configuracion, permisos, Studio

4. **Calcular estructura de la sesion**:
   - Regla 50/50: al menos 50% del tiempo en ejercicios practicos
   - Consulta `./references/buenas_practicas_formacion.md`
   - Si scope grande: dividir en bloques de max 20min teoria + practica
   - Si scope pequeno: sesion corta (1-1.5h)

5. **Identificar datos necesarios**:
   - Verificar via `odoo-pilot` que existen datos de prueba
   - Si faltan: avisar al consultor
   - SIEMPRE datos reales del entorno, NUNCA genericos

## Paso C1.5: Validacion del Plan de Sesion (OBLIGATORIO)

**DETENTE** y presenta plan propuesto:
- Datos de la formacion (nombre, app, perfil, duracion, formato)
- Temario propuesto con bloques (teoria + practica por bloque)
- Ejercicios incluidos
- Adaptaciones al proyecto
- Prerequisitos

Espera: Si / Ajustes. **NUNCA** generar materiales sin esta validacion.

## Paso C2: Generacion de Materiales

### a) Guia de Sesion
Template: `./templates/guia_sesion.md`. Incluye:
- Objetivos de aprendizaje
- Agenda detallada bloque a bloque
- Ejercicios practicos con instrucciones paso a paso
- Resultado esperado de cada ejercicio
- Preguntas frecuentes anticipadas
- Cuestionario de evaluacion post-sesion (3-5 preguntas practicas)

**Ejercicios**: SIEMPRE con datos reales. Nombres de productos, clientes, almacenes reales.

### b) Screenshots
Preguntar: "Incluir screenshots? (si por defecto)".
- Si SI: Generar placeholders `[SCREENSHOT:app_accion_contexto]` en puntos clave.
- Si NO: Sin placeholders.

### c) Cheat Sheet
Template: `./templates/cheat_sheet.md`. Max 1 pagina:
- Accesos directos (menu -> accion)
- Flujo principal en 5-7 pasos
- Tips y atajos de teclado
- Problemas comunes y soluciones

### d) Cuestionario de Evaluacion
Al final de la guia: 3-5 preguntas PRACTICAS ("Como harias X?", "Crea un Y con..."), cada una con resultado esperado.

## Paso C2.5: Validacion de Materiales (OBLIGATORIO)

**DETENTE** y presenta resumen:
- Guia: paginas, bloques, ejercicios, screenshots, preguntas evaluacion
- Cheat Sheet: accesos directos, pasos flujo principal

Espera: Si / Ajustes.

## Paso C3: Entregables Finales

Guardar en directorio de trabajo:
```
{app}_plan_formacion.md       -> Plan/resumen de la sesion
{app}_guia_sesion.md          -> Guia completa con ejercicios
{app}_cheat_sheet.md          -> Referencia rapida 1 pagina
```

Naming: snake_case minusculas (ej: `ventas_guia_sesion.md`, `aprobacion_compras_guia_sesion.md`).
