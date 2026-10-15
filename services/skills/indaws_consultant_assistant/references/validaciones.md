# Validaciones

## Qué es

El proceso de generar documentos de validación para que el cliente confirme formalmente que lo implementado funciona según lo acordado. Traduce resultados técnicos a lenguaje de negocio.

## Cuándo aplica

- Fase 3 (Pre-proyecto): PLANIFICAR — identificar validaciones necesarias y crear requests
- Fase 4.5 (Ejecución): DOCUMENTAR — generar documentos de validación completos

## Qué debe saber el consultor

### 4 Tipos de validación

| Tipo | Cuándo | Trigger |
|------|--------|---------|
| **App** | Al completar todos los checklists de una app + formación | Checklists CONFIG/STUDIO completados |
| **Migración** | Después de importar datos del cliente | importation-data-assistant completado |
| **Desarrollo/Integración** | Después de completar un desarrollo custom | Módulo en producción |
| **Preproducción** | Antes del Go-Live | Todas las validaciones anteriores aprobadas |

### Documento de validación (6 secciones)

1. **Qué se ha entregado**: Lista de funcionalidades implementadas
2. **Cómo funciona ahora**: Descripción del flujo en lenguaje de negocio
3. **Criterios de aceptación**: Condiciones que deben cumplirse para aprobar
4. **Evidencia visual**: Capturas de pantalla (placeholders para odoo-screenshot-generator)
5. **Limitaciones**: Qué no se ha incluido y por qué
6. **Aprobación**: Sección para firma/confirmación del cliente

### Flujo de validación

```
PLANIFICAR: Identificar → Crear requests [VALIDACION] → Asignar deadlines
DOCUMENTAR: Leer análisis → Generar documento → Validar → Enviar al cliente
```

Las solicitudes de validación al cliente se canalizan a través del skill `client-request-manager`, que gestiona el envío, seguimiento y recepción de las peticiones de tipo validación en el modelo `x_client_request`.

## Qué debe hacer el consultor

1. En fase pre-proyecto: ejecutar modo PLANIFICAR para crear requests de validación
2. Al completar una app/módulo: ejecutar modo DOCUMENTAR para generar el documento
3. Complementar con `odoo-flow-tester` para generar evidencia de testing
4. Revisar y personalizar el documento antes de enviarlo
5. Enviar al cliente via `client-request-manager`

## Skills relacionados

- `project-validation-drafting`: Skill principal (modos PLANIFICAR y DOCUMENTAR)
- `odoo-flow-tester`: Testing de flujos como evidencia
- `client-request-manager`: Envío del documento al cliente
- `odoo-screenshot-generator`: Capturas para la sección de evidencia visual

## Errores comunes

1. **Usar tecnicismos con el cliente**: Los documentos deben estar en lenguaje de negocio comprensible
2. **No incluir criterios de aceptación**: Sin criterios claros, el cliente no sabe qué aprobar
3. **Validar sin evidencia**: Los screenshots y resultados de test dan confianza al cliente
4. **No planificar validaciones desde el inicio**: Los deadlines de validación deben alinearse con la planificación
5. **Enviar validación sin revisar**: El documento debe ser comprensible y completo antes de enviar
