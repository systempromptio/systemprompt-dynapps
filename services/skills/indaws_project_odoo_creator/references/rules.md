# project-odoo-creator: Rules and Guidelines

## Task Creation Rules

- **Una tarea por modulo** (agrupada): NO crear tareas granulares
- **Tareas detalladas** ([DEV], [MIGR] complejas): SI son tareas individuales
- **Tareas genericas** (Config Generales, Soporte, Reuniones): tareas separadas
- **Fechas**: Convertir DD/MM/YYYY a YYYY-MM-DD HH:MM:SS (ambos campos son Datetime en v19)
- **Asignacion**: Pasar `user_ids: [[5]]` para evitar auto-asignacion al creador
- **Checklists**: NUNCA crear tarea sin items de checklist
- **Orden**: Respetar orden secuencial del plan

## Knowledge Article Rules

- **Search-before-create**: SIEMPRE buscar si existe antes de crear
- **Idempotencia**: Multiples ejecuciones no duplican articulos
- **Permisos**: "Interno" SIEMPRE con `is_desynchronized=true`
- **Portal**: Compartir con `commercial_partner_id` como miembro del raiz con "read"
- **Solo primer nivel**: Este skill crea raiz + 7 hijos. Subarticulos los crean otros skills

## General Rules

- **NUNCA crear el proyecto**: Solo se configura, ya existe
- **NUNCA eliminar datos** sin confirmacion explicita
- **NUNCA continuar si hay errores**: Informar y proponer solucion
- **Tono**: Profesional, directo, informativo. Mostrar progreso

## Validations

**Interactivas (obligatorias)**:
1. Paso 0: ID del proyecto y plan
2. Paso 1c: Si hay tareas existentes, decidir como proceder
3. Paso 3: Aprobar estructura completa antes de crear

**Automaticas (antes de crear)**:
4. Proyecto existe y es accesible
5. Plan contiene tareas con horas y fechas validas
6. Fechas coherentes (inicio < fin)
7. Total horas razonable (no difiere > 10% del plan)
8. Cliente existe con commercial_partner_id
9. Items de checklist con nombre y tipo valido

**Post-creacion**:
10. Todas las tareas creadas (verificar task_count)
11. Knowledge articles con jerarquia correcta
12. x_studio_knowledge vinculado al proyecto
13. Cliente es miembro del raiz con "read"
14. "Interno" tiene is_desynchronized=true

## Success Criteria

- Todas las tareas del plan creadas con horas, fechas y checklists
- Knowledge: articulo raiz + 7 hijos de primer nivel
- Articulo raiz vinculado via x_studio_knowledge
- Articulos accesibles desde portal (excepto Interno)
- Idempotente: segunda ejecucion no duplica nada
- Resumen permite verificar todo lo creado
