# project-analysis: Workflow Details

## Recognized Applications (23)

Configuraciones generales, CRM, Ventas, Compras, Inventario, Facturacion,
Contabilidad, Integraciones, Recursos humanos, TPV, Fabricacion, Proyectos,
Helpdesk, Flota, Servicio externo, Firma, E-learning, Web, E-commerce,
E-mail Marketing, Calidad, Planificacion, App movil.

---

## Paso 0: Informacion Inicial (STOP)

Solicita al usuario ANTES de cualquier analisis:

1. **Version de Odoo**: 14.0-19.0 (CRITICO para funcionalidades estandar disponibles)
2. **Modo de trabajo**: A (Google Drive) o B (Manual)

### Modo A - Google Drive (Recomendado)

Lee cuestionarios desde una carpeta de Google Drive y genera un documento de analisis completo (Markdown) con todas las apps encontradas.

Requisitos:
- Nombre del cliente/proyecto
- ID de carpeta de Drive (de la URL de Drive)

### Modo B - Manual

Analiza UNA sola aplicacion. El usuario selecciona la app y pega el cuestionario en el chat.

**DETENTE** y espera la respuesta del usuario.

---

## MODO A: Flujo Google Drive

### Paso 1A: Escaneo de Cuestionarios

1. **Listar archivos**: `listFolder` con el folderId proporcionado
   - Si falla, informar al usuario y sugerir verificar el ID
2. **Filtrar cuestionarios**: Archivos cuyo nombre empiece por `"Cuestionario - "`
3. **Extraer nombres de apps**: Del titulo (ej: "Cuestionario - Ventas" -> "Ventas")
4. **Validar apps**: Contra la lista de 23 apps conocidas. Ignorar y avisar los no reconocidos
5. **Confirmar con usuario**: Mostrar lista y pedir confirmacion

**DETENTE** y espera confirmacion.

### Paso 2A: Lectura y Generacion del Analisis

**Validar referencias**:
1. `./references/guias_por_aplicacion.md` (OBLIGATORIO, detente si no existe)
2. `references/shared/paquetes_indaws_core.md` (opcional, continuar sin paquetes si falta)

Para cada cuestionario confirmado:
1. Leer cuestionario via `getGoogleDocContent`
2. Leer guias de la app en `guias_por_aplicacion.md`
3. Cargar paquetes Core de `references/shared/paquetes_indaws_core.md`
4. Cargar referencias adicionales: `rules.md`, `categorias_y_pesos.md`, `ejemplos_gaps.md`
5. Generar analisis usando template `./templates/analisis_app.md` (7 secciones)
6. Acumular en memoria

### Paso 3A: Generacion del Documento Compilado

Estructura:
```markdown
# ANALISIS FUNCIONAL DE IMPLANTACION
## {Nombre Cliente}
*Indaws Business Solutions S.L. - Odoo Gold Partner*
Fecha: {fecha actual DD/MM/YYYY}

## INDICE DE APLICACIONES
1. {App 1}
2. {App 2}

# ANALISIS FUNCIONAL - {APP 1}
## 1. Introduccion
...7 secciones...
```

### Paso 4A: Confirmacion

Presentar resumen: numero de apps analizadas, sugerir `project-plan` como siguiente paso.

---

## MODO B: Flujo Manual

### Paso 0B: Recopilacion

1. Aplicacion a analizar (de la lista de 23)
2. Respuestas del cuestionario (pegado o archivo)

**DETENTE** y espera respuesta.

### Paso 1B: Lectura de Referencias

1. Validar existencia de `guias_por_aplicacion.md` y `paquetes_indaws_core.md`
2. Leer guias de la app seleccionada
3. Cargar referencias: `rules.md`, `categorias_y_pesos.md`, `ejemplos_gaps.md`

### Paso 2B: Generacion

Generar analisis con template `./templates/analisis_app.md` siguiendo las 7 secciones y aplicando los criterios de `rules.md`. Salida directamente en chat.

---

## Paso 6 (Modo A y B): Traspaso a Micaela (opcional)

Se ofrece SIEMPRE al terminar el analisis (tras Paso 4A o Paso 2B), sin importar el modo.

1. **Preguntar**: "¿Preparamos las preguntas para Micaela?" (traspaso a Crol). Si el usuario dice que no, terminar aqui sin generar nada mas.
2. **Rellenar lo ya conocido por el analisis**: usar `./templates/ficha_traspaso_micaela.md` como checklist. Los campos `[ANÁLISIS]` se sacan del analisis recien generado (descripcion del proyecto, modulos, fases, desarrollos previstos, duracion si `project-plan` ya corrio).
3. **Consultar la oportunidad en Odoo**: para TODOS los campos `[OPORTUNIDAD]` de la plantilla, busca antes de preguntar. Sigue `references/shared/odoo-connection.md` (MCP primero, odoo-pilot si no aplica). Pasos:
   - Identifica al cliente en Odoo (nombre/CIF ya conocidos del analisis o del chat; si no, pregunta solo el nombre para buscar).
   - `odoo_describe_model` sobre `crm.lead` y `res.partner` para saber que campos existen de verdad en ESTA instancia (no asumas nombres — evita el error de hardcodear un campo que no esta).
   - `orm_search_read` en `crm.lead` (contacto, cargo, correo, telefono, fecha de apertura, descripcion/notas) y en `res.partner` vinculado (CIF, sector/industry_id, ubicacion, alta en base de datos = create_date).
   - Lee tambien la descripcion/notas libres de la oportunidad (y el chatter si hace falta): ahi suele estar lo que no es un campo estructurado — herramientas actuales, subvencion, consultoria previa, crecimiento, presupuesto, predisposicion a multianualidad, cliente conflictivo. Criba esa info y repartela en los bloques correspondientes de la ficha.
   - Si no hay match de oportunidad, el MCP no esta conectado, o un campo/dato concreto no aparece: ese campo cae a `[PREGUNTAR]`. No bloquees el traspaso por esto — no es una precondicion dura, solo reduce lo que hay que preguntar despues.
4. **Preguntar en bloque** (una sola tanda, no una a una) SOLO los campos que sigan como `[PREGUNTAR]` tras los pasos 2 y 3. **DETENTE** y espera respuesta.
5. **Generar la ficha**: con las respuestas + lo ya conocido (analisis + oportunidad), rellenar la plantilla completa (sin los marcadores de origen `[ANÁLISIS]`/`[OPORTUNIDAD]`/`[PREGUNTAR]`).
6. **Redactar el borrador de correo a Micaela**: mismo formato y tono que `./references/golden-example-traspaso-micaela.md` (tabla "Datos de cliente" + bloques narrativos 1/2/3). Ver GOTCHA en `../SKILL.md`: nunca incluir el detalle tecnico de GAPs (seccion 2 del analisis), solo el resumen a nivel de decision.
7. **Presentar el borrador** al consultor y esperar confirmacion explicita antes de darlo por enviado — el consultor es quien lo pega/envia, o pide ajustes.
