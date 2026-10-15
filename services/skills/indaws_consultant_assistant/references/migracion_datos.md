# Migración de Datos

## Qué es

El proceso de importar datos del cliente al sistema Odoo. Incluye análisis de ficheros, normalización, mapeo de campos e importación masiva via API.

## Cuándo aplica

Fase 4.4 (Ejecución): Cuando existen items de tipo MIGR en los checklists, o cuando el cliente proporciona ficheros de datos.

## Qué debe saber el consultor

### Fuente de datos

Los ficheros del cliente llegan como adjuntos en `x_client_request`:
- El consultor crea requests de tipo `data_file` durante la fase de pre-proyecto
- El cliente adjunta los ficheros en el chatter del request
- El skill detecta las respuestas con adjuntos via `x_has_response`

### Capacidades del skill

El skill `importation-data-assistant` tiene modelos mapeados para los objetos principales de Odoo (contactos, productos, facturas, pedidos de venta, pedidos de compra, oportunidades) e incluye normalizaciones automáticas para países, provincias, teléfonos, fechas y NIF/VAT españoles. No es necesario detallar el mapeo aquí; el skill lo gestiona internamente.

### Método de importación

- **Método**: `load` (API nativa de Odoo)
- **Batches**: 1000 registros por llamada
- **External IDs**: Prefijo `__import__.*` para idempotencia
- **Test obligatorio**: Importar muestra de 5-10 filas antes de importación completa
- Si falta un campo necesario → crear checklist item para `odoo-configurator` (NUNCA crear campos directamente)

### Documentación de resultado

1. **Nota interna** en la tarea: Resumen técnico (registros importados, errores, advertencias)
2. **Actualización del x_client_request**: Descripción en lenguaje de negocio para que el cliente valide los datos importados

## Qué debe hacer el consultor

1. Crear requests de tipo `data_file` para cada conjunto de datos necesario
2. Cuando el cliente adjunte los ficheros, ejecutar `importation-data-assistant`
3. Revisar el análisis del fichero (encoding, duplicados, inconsistencias)
4. Aprobar el mapeo de campos propuesto
5. Ejecutar el test con muestra reducida
6. Aprobar la importación completa
7. Solicitar validación del cliente sobre los datos importados

## Skills relacionados

- `importation-data-assistant`: Skill principal de migración de datos
- `client-request-manager`: Para solicitar los ficheros al cliente
- `odoo-configurator`: Si falta un campo necesario
- `odoo-flow-tester`: Para verificar que los datos importados funcionan en los flujos

## Errores comunes

1. **Importar sin test previo**: Siempre probar con muestra reducida
2. **No normalizar datos**: Los datos del cliente suelen tener inconsistencias (países, fechas, teléfonos)
3. **Crear campos manualmente**: Si falta un campo, crear item para odoo-configurator
4. **No usar external IDs**: Sin external IDs, no se puede reimportar sin duplicar
5. **No pedir validación al cliente**: Los datos importados deben ser validados por el cliente
