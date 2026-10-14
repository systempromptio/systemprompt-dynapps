# Configuración

## Qué es

La ejecución del día a día del consultor: configurar Odoo según los checklists de cada tarea. Incluye tanto configuraciones estándar (settings, módulos) como cambios via Studio (campos, acciones automatizadas).

## Cuándo aplica

Fase 4.1 (Ejecución): Es el subflujo principal de la fase de ejecución.

## Qué debe saber el consultor

### Skill principal: odoo-configurator

`odoo-configurator` es EL skill de configuraciones. Internamente llama a `odoo-pilot` (para settings, módulos, datos maestros) y a `odoo-studio-fields` (para campos, acciones automatizadas y Studio). El consultor solo interactúa con `odoo-configurator`.

### Tipos de configuración

| Tipo checklist | Herramienta interna | Skill externo |
|----------------|---------------------|---------------|
| CONFIG | odoo-pilot | odoo-configurator |
| STUDIO/AUTO | odoo-studio-fields | odoo-configurator |
| DEV | — (se ignoran silenciosamente) | dev-assessment |
| MIGR | — (se ignoran silenciosamente) | importation-data-assistant |

### Documentación

- **Configuraciones estándar (CONFIG)**: No se documentan de forma explícita.
- **Adaptaciones Studio y automatizaciones**: SÍ se documentan en Knowledge, en el artículo "Adaptaciones", por el propio skill `odoo-configurator`.

### Entrenamiento continuo

El skill `odoo-configurator` se irá entrenando con configuraciones habituales a medida que se ejecuten proyectos.

### Flujo de trabajo

```
1. Leer tarea + checklists pendientes (fetch_task_checklist.sh)
2. Clasificar items por tipo (CONFIG vs STUDIO/AUTO)
3. Proponer plan de ejecución al consultor (VALIDACIÓN)
4. Ejecutar CONFIG via odoo-pilot
5. Ejecutar STUDIO/AUTO via odoo-studio-fields (VALIDACIÓN)
6. Documentar (nota interna o Knowledge)
7. Verificar con odoo-flow-tester (opcional)
```

## Qué debe hacer el consultor

1. Identificar la tarea con checklists pendientes
2. Ejecutar `odoo-configurator` con el ID de la tarea
3. Aprobar el plan de ejecución propuesto
4. Revisar las configuraciones realizadas
5. Marcar items como completados solo después de verificar
6. Si faltan datos del cliente, usar `client-request-manager`

## Skills relacionados

- `odoo-configurator`: Skill principal de configuración
- `client-request-manager`: Para solicitar datos faltantes al cliente
- `odoo-flow-tester`: Para verificar que la configuración es correcta
- `training-planner`: Para preparar formación sobre lo configurado

## Errores comunes

1. **Ejecutar items DEV/MIGR como CONFIG**: Son tipos diferentes con workflows distintos
2. **No documentar adaptaciones Studio**: Los cambios Studio sin documentar en Knowledge generan problemas de mantenimiento
3. **Marcar como completado sin verificar**: El checklist pierde su función de control
4. **Configurar sin datos del cliente**: Si faltan datos, solicitar via client-request-manager
