# Patrones por Dia de la Semana

## Reglas Fijas

### Viernes

- **OBLIGATORIO**: Ejecutar `project-weekly-update` en CADA proyecto activo
  - Aparece como **URGENTE** automaticamente
  - `project-weekly-update` lanza internamente `project-health-monitor`
  - Tiempo estimado: 15-20min por proyecto
  - Si hay 4 proyectos activos → reservar ~1h para weekly updates

### Lunes

- **RECOMENDADO**: Revision completa del dashboard de client requests
  - Revisar todas las peticiones vencidas del fin de semana
  - Detectar respuestas recibidas durante el fin de semana
  - Aparece como **IMPORTANTE**

### Dia anterior a reunion

- **URGENTE**: Preparar la reunion del dia siguiente
  - Si es reunion de seguimiento: ejecutar `odoo-flow-tester` modo B para verificar estado
  - Si es formacion: verificar que `training-planner` CONTENIDO esta ejecutado
  - Preparar puntos de agenda basados en checklists completados y pendientes

### Dia de reunion

- **URGENTE** (post-reunion): Generar acta con `project-meeting-minutes`
  - No esperar al dia siguiente — hacerlo inmediatamente despues

## Reglas de Contexto

### Semana de Go-Live

- Todas las acciones del proyecto Go-Live suben a URGENTE
- Priorizar verificaciones y validaciones sobre configuraciones nuevas

### Primera semana del mes

- Verificar que `project.project.plan.hour.line` del mes actual tiene las horas correctas
- Revisar si hay nuevos proyectos asignados

### Ultima semana del mes

- Verificar consumo de horas vs planificadas
- Si hay desvio significativo (>20%), incluir como IMPORTANTE

## Horarios Sugeridos

La agenda no impone horarios pero sugiere bloques logicos:

| Bloque | Hora sugerida | Tipo de trabajo |
|--------|--------------|-----------------|
| Primera hora | 08:30-10:00 | URGENTE: tickets, requests vencidos, preparar reuniones |
| Media manana | 10:00-12:00 | IMPORTANTE: configuraciones, importaciones, dev-assessments |
| Despues de comer | 13:00-15:00 | PLANIFICADO: validaciones, documentacion, formaciones |
| Ultima hora | 15:00-16:30 | Seguimiento: actas, emails, revisiones |
| Viernes tarde | 15:00-17:00 | Weekly updates de todos los proyectos |
