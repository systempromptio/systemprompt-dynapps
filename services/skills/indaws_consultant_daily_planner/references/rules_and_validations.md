# Consultant Daily Planner - Rules and Validations

## Rules

### Generales
- NUNCA ejecutar un skill automaticamente -- solo proponer con IDs concretos
- SIEMPRE incluir el skill a ejecutar y los parametros necesarios para invocarlo
- SIEMPRE verificar estructura Knowledge de cada proyecto
- SIEMPRE detectar dependencias entre skills (ej: weekly-update antes de health-fixer)
- SIEMPRE calcular deadlines en dias laborables (lunes-viernes)
- SIEMPRE indicar tiempo estimado por accion
- NUNCA mostrar datos internos de otros proyectos en modo B
- SIEMPRE obtener el empleado automaticamente desde ODOO_USER via hr.employee

### Priorizacion
- Consultar `references/reglas_priorizacion.md` para las reglas completas
- Consultar `references/patrones_dia.md` para reglas del dia de la semana
- Si hay conflicto entre niveles, el contexto cruzado manda (health <40, go-live proximo)
- Dentro de cada nivel, ordenar por impacto al cliente

### Client Requests
- Solo proponer envio si la tarea vinculada ya ha empezado (`planned_date_begin <= hoy`)
- Si la tarea no ha empezado, el request draft NO se incluye aunque tenga deadline cercano
- Requests vencidos >3 dias son SIEMPRE urgentes
- Requests con respuesta (x_has_response=true) sin procesar son IMPORTANTES

### Reuniones
- Detectar reuniones buscando checklists con nombres que contengan "Reunion", "Formacion", "Seguimiento"
- Una reunion esta "sin preparar" si:
  - Es de seguimiento y no se ha ejecutado odoo-flow-tester recientemente
  - Es de formacion y no se ha ejecutado training-planner CONTENIDO
- Una reunion esta "sin acta" si:
  - El checklist tiene fecha pasada
  - No hay articulo correspondiente en "Actas de reunion" de Knowledge

### Tickets
- Solo considerar tickets con stage no plegado (`fold=false` = abierto)
- Tickets con prioridad alta/urgente suben automaticamente a URGENTE
- Si hay mas de 3 tickets abiertos en un proyecto, avisar como problema sistematico

### Skills Futuros
Para `technical_dev_validation` y `go-live-validation` que aun no existen:
- Incluir la accion en la agenda igualmente
- Marcar como: "Skill no disponible aun. Accion manual requerida."

## Validaciones

### Interactivas
Ninguna -- la agenda es informativa.

### Automaticas
1. El empleado existe en `hr.employee` con el email proporcionado
2. Al menos 1 proyecto activo encontrado (modo A)
3. El proyecto existe y esta activo (modo B)
4. `x_studio_knowledge` configurado en cada proyecto (avisar si no)
5. Estructura Knowledge verificada contra `references/estructura_knowledge.md`
6. Las fechas de deadline son coherentes
7. Los skills referenciados existen en el sistema (excepto futuros)
8. No se proponen acciones duplicadas para el mismo item
