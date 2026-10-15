# Desarrollo a Medida

## Qué es

El proceso de gestión de desarrollos custom en un proyecto de implantación Odoo. El consultor no desarrolla, sino que prepara la documentación técnica, coordina con el equipo de desarrollo y valida el resultado.

## Cuándo aplica

Fase 4.3 (Ejecución): Cuando existen items de tipo DEV en los checklists del proyecto.

## Qué debe saber el consultor

### Flujo completo

```
dev-assessment → Equipo técnico desarrolla (manual) → Documentación → Validación con cliente
```

1. **dev-assessment**: El consultor prepara la documentación técnica detallada para el equipo de desarrollo. Incluye requisitos funcionales, flujos esperados y criterios de aceptación.
2. **Desarrollo** (manual): El equipo técnico realiza el desarrollo. El consultor NO programa.
3. **Documentación**: Cuando el desarrollo está listo, documentar con `odoo-module-documenter`.
4. **Validación**: Usar `odoo-flow-tester` para verificar y `project-validation-drafting` para generar el documento de validación para el cliente.

### Responsabilidades del consultor

| Tarea | Responsable | Herramienta |
|-------|-------------|-------------|
| Documentar requisitos | Consultor | dev-assessment |
| Desarrollar el módulo | Equipo técnico | — |
| Probar en staging | Consultor + Técnico | odoo-flow-tester |
| Documentar el módulo | Consultor | odoo-module-documenter |
| Validar con el cliente | Consultor | project-validation-drafting |

### Criterios para dev-assessment

El dev-assessment verifica antes de proponer desarrollo:
1. ¿Se puede resolver con configuración estándar de Odoo?
2. ¿Existe un paquete Indaws Core que lo cubra?
3. ¿Se puede resolver con Studio/campos automatizados?
4. Solo si las 3 anteriores son negativas → desarrollo custom

### UAT (User Acceptance Testing)

- Probar en un entorno de staging, NUNCA en producción
- Usar datos reales del cliente, no datos de demo
- Verificar flujos completos, no solo la funcionalidad aislada
- Comprobar permisos de usuario para cada rol
- Validar informes y vistas que usa el cliente

## Qué debe hacer el consultor

1. Preparar dev-assessment para cada item DEV
2. Coordinar con el equipo técnico (plazos, prioridades)
3. Cuando el desarrollo esté listo, probar con datos reales
4. Documentar el módulo con odoo-module-documenter
5. Generar documento de validación para el cliente
6. Obtener aprobación del cliente antes de subir a producción

## Skills relacionados

- `dev-assessment`: Evaluación y documentación técnica del desarrollo
- `odoo-module-documenter`: Documentación funcional del módulo
- `odoo-flow-tester`: Testing de flujos de negocio
- `project-validation-drafting`: Documento de validación para el cliente
- `client-request-manager`: Enviar validación al cliente

## Errores comunes

1. **Desarrollar sin dev-assessment**: El equipo técnico necesita requisitos claros y documentados
2. **No hacer UAT con datos reales**: Los bugs aparecen con datos reales, no con datos de demo
3. **Consultor programa directamente**: El consultor documenta y valida, no programa
4. **Subir a producción sin validación del cliente**: El cliente debe aprobar formalmente
5. **No documentar el módulo**: Un módulo sin documentación genera dependencia del equipo que lo desarrolló
