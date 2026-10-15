# Gotchas de datos, fuentes y conectores

Todos medidos: cada uno ha costado al menos una vez un diagnóstico falso o una guía ilegible. Las definiciones de proyecto activo, tarea terminada y checklist hecho, y los límites conocidos de los datos, están en el artículo de doctrina y mandan sobre esta lista.

## Doctrina

1. **El id del artículo no es estable entre instancias.** Nunca se escribe fijo ni se recuerda de otra ejecución: se resuelve cada vez por título y ruta.
2. **Un identificador desnudo no comunica.** Ni `D9` ni `31221` significan nada para quien lee, y tampoco vale nombrarlo una vez y luego citar el número. Sin las reglas de forma, la guía obliga a abrir Odoo y deja de usarse.

## Fuentes fuera de Odoo

3. **Una reunión del calendario no siempre es del proyecto.** Se cruza por los asistentes con los contactos del cliente, no por el título. Una llamada interna que menciona al cliente no es una reunión con el cliente.
4. **Un correo reenviado parece del cliente y no lo es.** Mira el remitente real, no el asunto ni la cadena.
5. **Un fichero de la carpeta no dice lo que contiene por su nombre.** Si no se puede afirmar qué aporta, se nombra y se pregunta; nunca se propone algo basándose en el nombre.

## Odoo

6. **`xx_burn_variance` es consumo menos avance, no consumo menos tiempo.** Para comparar consumo con calendario hay que restar a mano las dos métricas de porcentaje.
7. **`project.task.state` no dice si una tarea está terminada**: se queda en `01_in_progress` en tareas cerradas hace meses. Manda si la etapa está plegada.
8. **Los ids de etapa de proyecto se reutilizaron** al pasar de catorce a siete. Un filtro por id sigue ejecutándose y significa otra cosa. Se selecciona por nombre.
9. **El campo de checklist no se llama igual en los dos modelos**: en el proyecto, `xx_checklist_item_ids`; en la tarea, `xx_checklist_ids`.
10. **La fecha límite de una tarea es datetime y la de un checklist es date.** No se comparan sin normalizar.
11. **En proyectos migrados, todos los checklists tienen como fecha de creación la de la migración.** Una comprobación que mida "creado después del arranque" no aplica ahí: se dice, en vez de dar un falso positivo masivo.
12. **`effective_hours` del proyecto no está almacenado**: no se puede filtrar, ordenar ni agrupar. El consumo se calcula sumando los partes.
13. **Más de la mitad de los checklists no tienen tipo.** Como el tipo decide a qué rama va cada hecho documentado, el bloque de documentación se queda corto en muchos proyectos. Se reporta, no se adivina.

## Ejecución programada

14. **En una tarea programada no hay nadie para autorizar nada.** Un conector que pide login o permiso deja su bloque sin comprobar. Se dice en la cabecera; no se interpreta como "no hay nada".
