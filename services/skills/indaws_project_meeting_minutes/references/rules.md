# project-meeting-minutes: Rules and Guidelines

## Content Rules

- **PERSONALIZAR TODO**: Cada acta debe reflejar exactamente lo discutido, nunca contenido generico
- **Tono**: Profesional, objetivo, factual. No interpretar mas alla de lo dicho
- **Decisiones**: Siempre indicar QUIEN decidio QUE
- **Acciones**: Siempre con responsable Y fecha limite. Si no hay fecha, proponer una razonable
- **Cambios de alcance**: Marcar con banner visual amarillo. Ser claro: "requiere presupuesto separado"
- **NO inventar** informacion que no este en las notas. Si falta algo, marcar como "No mencionado"

## Scope Detection Rules

- Cotejar contra: documento de analisis + tareas + checklists. NUNCA contra sale.order
- **Adiciones menores**: Solo items de checklist en tareas existentes. Con aprobacion
- **Cambios de alcance**: NUNCA crear tareas ni presupuestos. Solo senalar
- **En caso de duda**: Preguntar al usuario, no asumir

## Knowledge Article Rules

- **Actas en 1 solo articulo**: Todas van en "Actas de Reunion"
- **Nuevas arriba**: La mas reciente siempre primero
- **Estilos inline**: Obligatorio
- **Search-before-create**: Para cuestionarios, siempre buscar si ya existe
- **No sobreescribir cuestionarios**: Si ya tiene respuestas, solo rellenar huecos

## General Rules

- **NUNCA escribir en Odoo sin validacion** del usuario
- **NUNCA crear tareas** (project.task). Solo checklist items
- **NUNCA crear sale.order** ni ampliaciones. Solo senalar la necesidad
- **NUNCA ejecutar client-request-manager**. Solo sugerir
- **Fecha por defecto**: Hoy
- **Asistentes por defecto**: Extraer de las notas si no se indican

## Validations

**Interactivas (obligatorias)**:
1. Paso 0: proyecto, tipo y notas
2. Paso 2b: cuestionario rellenado (solo tipo analisis)
3. Paso 3: deteccion de alcance y clasificacion
4. Paso 5: acta completa antes de publicar

**Automaticas**:
5. Proyecto existe con x_studio_knowledge vinculado
6. Articulo "Actas de Reunion" existe bajo Knowledge
7. Notas no vacias
8. Al menos un asistente identificado
9. Fecha valida
10. HTML con tags balanceados y estilos inline
11. App reconocida (tipo analisis)
12. Items de checklist con tarea destino valida
13. Cambios de alcance correctamente clasificados

## Success Criteria

- Acta refleja fielmente lo discutido (sin inventar ni omitir)
- Cambios de alcance correctamente identificados y senalados
- Cuestionario actualizado (tipo analisis)
- Items de checklist creados en tareas correctas
- Acta acumulada en Knowledge con la mas reciente arriba
- Usuario tiene claro que acciones requieren seguimiento
- Multiples ejecuciones no duplican actas (data-acta-fecha unico)
