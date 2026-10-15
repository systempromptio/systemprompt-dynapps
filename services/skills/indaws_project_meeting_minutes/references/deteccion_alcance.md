# Deteccion de Cambios de Alcance

Algoritmo para detectar si lo discutido en una reunion esta dentro del alcance del proyecto o representa un cambio que requiere presupuesto separado.

---

## Principio Fundamental

> Todo lo que NO esta documentado en el analisis funcional NI tiene tarea/checklist asociado es potencialmente un cambio de alcance. Un cambio de alcance SIEMPRE requiere presupuesto separado.

---

## Fuentes de Cotejo (3 niveles)

### Fuente A: Documento de Analisis

**Donde esta**: Articulos hijos de "Analisis" en Knowledge del proyecto (uno por app).

**Que contiene**: Requisitos funcionales aprobados, flujos de negocio, GAPs documentados, soluciones propuestas.

**Como cotejar**:
- Buscar coincidencias semanticas (no literales) entre lo discutido y lo documentado
- Un requisito esta cubierto si aparece en:
  - La descripcion de flujos del analisis
  - La lista de GAPs con solucion propuesta
  - Las decisiones funcionales documentadas
  - El checklist de implementacion (seccion 6 del analisis)
  - Las tareas detalladas (seccion 7 del analisis)

**Si NO existe documento de analisis** (proyecto sin fase de analisis completada): Usar solo Fuentes B y C. Ser mas conservador en la clasificacion (mas items como "ambiguos").

### Fuente B: Tareas del Proyecto (project.task)

**Donde esta**: Tareas activas del `project.project`.

**Que contiene**: Tareas por modulo (agrupadas) y tareas detalladas (desarrollos, migraciones).

**Como cotejar**:
- Comparar el tema discutido con el nombre y descripcion de cada tarea
- Una tarea "Ventas" cubre todo lo que sea configuracion estandar de ventas
- Una tarea "[DEV] Tarificacion Dinamica" cubre solo ese desarrollo especifico

### Fuente C: Checklists (xx.checklist.item)

**Donde esta**: Items de checklist vinculados a las tareas del proyecto.

**Que contiene**: Acciones concretas de implementacion con `type`, una seleccion CERRADA de ocho valores: `analysis`, `migration`, `development`, `report`, `test`, `training`, `studio`, `config`.

**Como cotejar**:
- Comparar el tema discutido con el nombre (`name`) de cada item
- Los items de checklist son el nivel mas granular de lo que esta planificado
- Si un tema coincide con un item de checklist, esta claramente dentro del alcance

---

## Algoritmo de Clasificacion

Para cada tema/requisito/solicitud mencionado en la reunion:

```
1. Buscar en Fuente C (checklists)
   → Match exacto o muy similar? → DENTRO DEL ALCANCE

2. Buscar en Fuente B (tareas)
   → Match con nombre/descripcion de tarea? → DENTRO DEL ALCANCE

3. Buscar en Fuente A (analisis)
   → Match con requisito/flujo/GAP documentado? → DENTRO DEL ALCANCE

4. No hay match en ninguna fuente
   → Evaluar complejidad:
     a. Es una configuracion menor que encaja en una tarea existente?
        → ADICION MENOR (proponer checklist item)
     b. Es un requisito nuevo significativo (desarrollo, integracion, modulo)?
        → CAMBIO DE ALCANCE (requiere presupuesto)
     c. No esta claro?
        → AMBIGUO (preguntar al usuario)
```

---

## Criterios de Clasificacion Detallados

### DENTRO DEL ALCANCE

Ejemplos:
- "Configurar 3 equipos de ventas" → Match con checklist "[CONFIG] Configurar equipos de ventas" en tarea Ventas
- "Migrar los 500 productos del Excel" → Match con tarea "[MIGR] Migracion de productos"
- "Añadir campo de descuento en el pedido" → Match con analisis: GAP documentado con solucion Studio

**Accion**: Solo documentar en el acta. No requiere ninguna accion adicional.

### ADICION MENOR

Criterios para clasificar como adicion menor:
- El tema es una variacion o extension de algo ya planificado
- Se resuelve con configuracion estandar o Studio (no requiere desarrollo)
- Encaja logicamente en una tarea existente (misma area funcional)
- El esfuerzo estimado es menor a 2 horas

Ejemplos:
- "Necesitamos un tipo de pago mas ademas de los que ya discutimos" → Item de checklist en tarea Contabilidad
- "Añadir un campo de referencia interna en el producto" → Item de checklist `type: studio` en tarea Inventario
- "Configurar una etapa adicional en el pipeline de CRM" → Item de checklist `type: config` en tarea CRM

**Accion**: Proponer nuevo `xx.checklist.item` en la tarea correspondiente. SIEMPRE con aprobacion del usuario.

**Campos del item propuesto** (verificados contra `fields_get`):

| Campo | Obligatorio | Valor al proponer |
|-------|-------------|-------------------|
| `name` | SI | Lo acordado en la reunion, sin prefijo de tipo |
| `task_id` | SI | Tarea existente destino |
| `weight` | SI | 1-5 (Odoo pone 1 por defecto) |
| `state` | SI | `pending` (default de Odoo para un item nuevo) |
| `project_id` | no | Proyecto de la reunion |
| `type` | no | Uno de los OCHO valores cerrados |
| `sequence` | no | Odoo pone 10 por defecto |
| `user_id` | no | Responsable, si se acordo en la reunion |
| `date_deadline` | no | Fecha limite, si se acordo |

`date_done` NO se rellena al crear: es la fecha real de finalizacion.

**Si lo hablado no encaja en ninguno de los ocho valores de `type`: PARARSE y
preguntar al consultor.** No forzar un valor ni dejarlo vacio para salir del paso.

### CAMBIO DE ALCANCE

Criterios para clasificar como cambio de alcance:
- El tema NO aparece en ninguna de las 3 fuentes
- Requiere desarrollo a medida (codigo Python/JS)
- Implica una integracion con sistema externo no contemplada
- Requiere un modulo nuevo (no estandar, no Core, no Studio)
- El esfuerzo estimado supera las 4 horas
- Implica un area funcional no incluida en el proyecto (ej: piden Fabricacion pero no esta contratado)

Ejemplos:
- "Necesitamos una API con nuestro sistema de almacen" → Integracion no contemplada
- "Queremos un portal para que los clientes vean sus pedidos en tiempo real" → Desarrollo nuevo
- "Necesitamos generar ficheros SEPA para los pagos" → Modulo/funcionalidad no incluida
- "Queremos gestionar la fabricacion con Odoo" → App no contratada

**Accion**: BANDERA ROJA en el acta. Indicar claramente:
1. Que NO esta en el alcance actual
2. Que requiere presupuesto separado
3. Sugerir que el PM prepare una ampliacion con `quotation-builder`

**NUNCA crear tareas, checklists ni presupuestos para cambios de alcance.**

### AMBIGUO

Criterios para clasificar como ambiguo:
- Match parcial (el tema es similar pero no identico a algo existente)
- La complejidad no esta clara (podria ser configuracion o desarrollo)
- El usuario hablo de algo de forma imprecisa y no se sabe si es nuevo o existente
- El analisis menciona algo parecido pero con diferente enfoque

Ejemplos:
- "Queremos un informe de ventas personalizado" → Puede ser una vista pivot (estandar) o un desarrollo
- "Necesitamos aprobar los pedidos de compra" → Puede estar en el flujo estandar o ser un workflow custom
- "Queremos que el stock se actualice automaticamente" → Puede ser configuracion de rutas o integracion

**Accion**: Preguntar al usuario como clasificarlo. Proporcionar contexto de por que es ambiguo.

---

## Presentacion de Resultados

Organizar la deteccion en 4 secciones claras:

1. **Dentro del alcance**: Lista simple con referencia a donde esta cubierto
2. **Adiciones menores**: Tabla con item propuesto + tarea destino + tipo. Pedir aprobacion
3. **Cambios de alcance**: Banner amarillo prominente. Lista con impacto estimado. Indicar que requiere presupuesto
4. **Ambiguos**: Lista con contexto y pregunta al usuario

---

## Reglas de Seguridad

- **Ante la duda, clasificar como ambiguo** y preguntar. Es mejor preguntar de mas que dejar pasar un cambio de alcance
- **No minimizar cambios de alcance**: Si algo claramente no esta en el proyecto, no buscar excusas para meterlo. La transparencia protege al equipo
- **No maximizar cambios de alcance**: Configuraciones menores que encajan en tareas existentes no son cambios de alcance. No generar alarma innecesaria
- **Contexto importa**: Un "campo extra" en un modulo ya contratado probablemente es una adicion menor. Un "campo extra" en un modulo no contratado es un cambio de alcance
- **El analisis es la verdad**: Si algo esta en el documento de analisis aprobado, esta dentro del alcance aunque no tenga checklist item aun
