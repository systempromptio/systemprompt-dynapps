# Importation Data Assistant - Workflow Detail

## Paso 1: Localizacion del Archivo

### 1a. Extraer contexto de la tarea

Ejecutar script `scripts/fetch_import_context.sh <task_id> "<tema_importacion>"`:
- Extrae datos de la tarea y proyecto
- Busca en `xx.checklist.item` items cuyo `name` contenga keywords de importacion
- Si encuentra checklist match -> busca `x_client_request` vinculado
- Si encuentra client request con `x_has_response = True` -> busca adjuntos (`.xlsx`, `.xls`, `.csv`)

### 1b. Descargar adjunto (si encontrado)

Si el script encuentra adjuntos candidatos:
- Ejecutar `scripts/download_attachment.py --attachment-id <id>`
- Descarga via `ir.attachment.datas` (base64 -> archivo local)

### 1c. Mostrar resultado al usuario

- **Si encontro archivo**: "He encontrado el archivo `{nombre}` adjunto al client request `{x_name}`. Es este el archivo correcto?"
- **Si encontro multiples**: Listar todos y preguntar cual usar
- **Si no encontro**: Pedir ruta al archivo manualmente

DETENTE y espera verificacion del usuario.

## Paso 2: Analisis del Archivo

### 2a. Parsear el archivo

Ejecutar `scripts/analyze_file.py <ruta_archivo>`:
- Lee Excel (openpyxl) o CSV (modulo csv)
- Detecta encoding y separador automaticamente
- Si Excel con multiples hojas -> reportar para que el usuario elija
- Genera JSON con estadisticas

### 2b. Revisar analisis

El script reporta:
- **Estructura**: filas, columnas, nombres, tipos inferidos
- **Completitud**: % de datos por columna
- **Duplicados**: filas duplicadas por columnas clave
- **Inconsistencias**: variantes de paises, provincias, fechas, telefonos
- **Correcciones propuestas**: mapa valor_original -> valor_corregido

## Paso 3: Mapeo de Campos (VALIDACION OBLIGATORIA)

### 3a. Obtener campos del modelo destino

Via odoo-pilot: ejecutar `fields_get` del modelo destino.

### 3b. Mapeo automatico

Consultar `references/mapeo_campos_comunes.md` para mapeos conocidos.
Para cada columna:
1. Buscar coincidencia exacta (nombre columna = label o nombre tecnico)
2. Buscar coincidencia parcial / sinonimos
3. Inferir por tipo de dato
4. Si no hay match -> marcar como "no mapeado"

### 3c. Presentar tabla de mapeo

Presentar mapeo con columna, campo Odoo, tipo, confianza y notas.

Para campos no mapeados, opciones:
- **(a) Ignorar**: Se omite la columna
- **(b) Mapear a campo existente**: Indica el campo de Odoo
- **(c) Crear campo nuevo**: Se creara checklist item para odoo-configurator

DETENTE y espera aprobacion del usuario.

**Si el usuario decide crear un campo nuevo**:
- Crear item en `xx.checklist.item` con `type = 'studio'` (Studio / automatizacion)
- Este skill NUNCA crea campos directamente

## Paso 4: Preparacion de la Importacion

### 4a. Preguntar entorno

Si es PRODUCCION -> advertencia explicita con numero de registros. DETENTE y espera confirmacion.

### 4b. Generar archivo normalizado

Aplicar correcciones, eliminar duplicados, generar external IDs (`__import__.{modelo}_{id}`), convertir relaciones a XML IDs. Guardar CSV normalizado como auditoria.

### 4c. Test con muestra

Importar primeras 5-10 filas con metodo `load`. Si errores -> mostrar y proponer correccion. Si OK -> pedir confirmacion para importar todo.

## Paso 5: Importacion

Seguir directrices del skill `odoo-importation`:
1. Preparar arrays `fields` y `data` para `load`
2. Importar en batches de maximo 1000 registros
3. Mostrar progreso por batch
4. Recoger errores por fila
5. Si errores parciales -> continuar e informar al final

## Paso 6: Informe y Documentacion

### 6a. Informe por pantalla

Resumen: modelo, entorno, total filas, duplicados eliminados, importados OK, errores, batches.

### 6b. Nota interna en la tarea

IMPORTANTE (Bug JSON2): Usar proceso de 2 pasos:
1. Crear mensaje con `message_post` (body vacio, `subtype_xmlid: mail.mt_note`)
2. Escribir body HTML real via `mail.message/write` usando template `templates/informe_importacion.md`

### 6c. Actualizar client request

Si existe `x_client_request` vinculado:
1. Buscar por nombre (`x_name`) y `x_type = 'data_file'`. El vinculo directo por
   `x_checklist_id` desaparecio: xx.checklist.item no tiene campo de vuelta hacia
   x_client_request, asi que no hay forma de resolver la peticion desde el item
2. Editar `x_description` via `update_record` (`x_response_notes` ya no existe)
3. Anadir HTML con template `templates/validacion_cliente.md`
   - Lenguaje de negocio (sin tecnicismos)
   - Que se importo y cuantos registros
   - Correcciones aplicadas en lenguaje simple
   - Solicitud de validacion al cliente
