# Reglas no negociables — migration-project-creator

## Cuando abortar

Aborta SIEMPRE y avisa al usuario:

1. El proyecto Odoo no existe (`project_id` invalido).
2. La etapa del proyecto no es "Migraciones" y el usuario no confirma proceder.
3. Falta cualquiera de los 6 inputs obligatorios (project_id, date_start, date_golive, strategy, hosting_type, analysis_path).
4. `date_golive <= date_start`.
5. `(date_golive - date_start).days < 42` (menos de 6 semanas) y el usuario no confirma explicitamente.
6. El inventario de modulos esta vacio o no se puede parsear.
7. El campo `x_studio_knowledge` no existe en `project.project` y no se puede crear.
8. Falta cualquier credencial Odoo (URL, DB, KEY).
9. El usuario rechaza el resumen del Paso 3.

---

## Cuando NO crear nada y solo avisar

1. Ya existen tareas (`task_count > 0`) y el usuario elige "Cancelar".
2. Ya existe el articulo raiz en Knowledge con contenido manual (avisar y proponer fusion antes de anadir hijos).

---

## Reglas de creacion

### Tareas

- **Crear via el script** `create_migration_tasks.py` de `migration-assistant`. NUNCA crear tareas con `odoo-pilot` directamente en este skill.
- **Asignacion**: el script ya pasa `user_ids: [[5]]` (sin asignar).
- **Tag Migration**: solo en [MIG]/[INFRA]/[GATE]. NUNCA en tareas de gestion.
- **Fechas SIEMPRE obligatorias**: este skill EXIGE `date_start` y `date_golive` antes de continuar. SIEMPRE las pasa al script. NUNCA aceptar invocacion sin las dos fechas. Todas las 18 tareas DEBEN quedar planificadas en Odoo.
- **Flags por defecto**: este skill SIEMPRE pasa `--with-checklists --with-gates`. Las tareas de gestion van por defecto (no hace falta flag).
- **Gestion del proyecto + Reuniones SIEMPRE**: nunca usar `--no-management`. Estas 2 tareas son base operativa del proyecto y deben estar activas desde el dia 1 hasta date_golive + 14d.

### Knowledge

- **Search-before-create**: SIEMPRE buscar si existe antes de crear (idempotencia).
- **Multiple ejecuciones**: el skill debe poder ejecutarse N veces sin duplicar articulos.
- **Interno**: SIEMPRE con `is_desynchronized=true`. Sin esto, el cliente ve documentos internos.
- **Portal**: compartir SIEMPRE con `commercial_partner_id` (empresa), no con un contacto individual.
- **Solo primer nivel**: el skill crea raiz + 6 hijos. Los sub-articulos los rellenan los devs/PMs durante la migracion.

### Etapas (project.task.type)

- **11 etapas standard**: enlazar al proyecto con `[4, project_id]` (link sin reemplazar).
- **NUNCA usar `[6, 0, ids]`** que reemplazaria los proyectos vinculados a la etapa.

---

## Validaciones automaticas

### Antes de crear (Paso 3 ya las hace, refuerza aqui)

1. ✓ Proyecto existe y es accesible.
2. ✓ Etapa es "Migraciones" o usuario confirma.
3. ✓ Inventario parseado correctamente (al menos 1 modulo).
4. ✓ `date_golive > date_start`.
5. ✓ Margen >= 42 dias o confirmacion explicita.
6. ✓ Tamaño detectado coincide con expectativa del usuario.

### Despues de crear (Paso 5 reporta)

1. ✓ Numero de tareas creadas = esperado (19 o 18 si no Studio).
2. ✓ Cada tarea tiene `planned_date_begin` y `date_deadline`.
3. ✓ Ninguna tarea tiene `date_deadline < planned_date_begin`.
4. ✓ Ninguna tarea tiene `user_ids` no vacio.
5. ✓ Tag Migration en todas las [MIG]/[INFRA]/[GATE].
6. ✓ Articulo raiz creado y vinculado a project (`x_studio_knowledge`).
7. ✓ Cliente anadido como miembro del raiz con permiso "read".
8. ✓ Articulo "Interno" tiene `is_desynchronized = true`.

---

## Reglas de comunicacion con el usuario

- **Tono**: profesional, directo. Sin emojis (excepto los que ya estan en el HTML de las tareas).
- **Mostrar progreso**: en cada paso del Paso 4, indicar que se esta haciendo.
- **Errores**: NO continuar. Detener, reportar y proponer solucion.
- **Logs largos del script**: capturar solo el resumen (no spamear el chat con cada tarea creada).

---

## Success criteria

Al final del skill, el usuario debe poder:

1. Abrir el proyecto en Odoo y ver 19 (o 18) tareas con fechas y descripciones HTML.
2. Filtrar por tag Migration y ver solo tareas de migracion (no las de gestion).
3. Ver el Gantt del proyecto con las fechas distribuidas en cascada.
4. Abrir cualquier tarea [GATE] y ver el checklist de entregables a verificar.
5. Abrir el Knowledge raiz desde el campo `x_studio_knowledge` del proyecto.
6. Si tiene portal, ver los articulos compartidos (sin Interno).
7. Ejecutar el skill otra vez sin que pase nada destructivo (idempotencia Knowledge; tareas si pediria primero que las elimine).
