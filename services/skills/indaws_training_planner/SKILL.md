---
name: indaws-training-planner
description: Planifica formaciones de proyectos Odoo (calendario completo con checklists y Knowledge) y genera materiales didacticos por sesion. Usa este skill cuando el usuario necesite planificar las formaciones de un proyecto o preparar contenido para una sesion concreta.
version: 2.2.1
effort: medium
model: sonnet
allowed-tools: [Read, Write, Bash, Glob, Grep]
---

# Planificador de Formacion Odoo

Tiene **DOS MODOS**: PLANIFICAR (calendario completo de formaciones del proyecto, checklists en Odoo, plan en Knowledge) y CONTENIDO (materiales didacticos detallados para UNA sesion concreta: guia, ejercicios, cheat sheet).

**Primer paso siempre**: Preguntar al usuario que modo necesita (A) PLANIFICAR o (B) CONTENIDO.

## Workflow -- Mode PLANIFICAR

See `references/mode-planificar-detail.md` for step-by-step commands.

1. **Recopilar datos** (P0): Credenciales Odoo, ID proyecto, ID Knowledge analisis, parametros formacion, tarea formaciones. STOP.
2. **Extraer contexto** (P1): Tareas con fechas, checklists, analisis funcional desde Knowledge, checklists existentes.
3. **Disenar plan** (P2): Identificar formaciones necesarias, agrupar por afinidad, programar en calendario, rellenar semanas vacias con reuniones de seguimiento. Consultar `references/reglas_planificacion.md`.
4. **Validar plan** (P2.5): STOP obligatorio. Presentar calendario completo + detalle. Esperar aprobacion.
5. **Crear en Odoo** (P3): Checklists en tarea de formaciones + articulo Knowledge con plan HTML inline.

## Workflow -- Mode CONTENIDO

See `references/mode-contenido-detail.md` for step-by-step commands.

1. **Recopilar datos** (C0): ID tarea, scope formacion, perfil asistentes, credenciales Odoo prueba. STOP.
2. **Analisis necesidades** (C1): Temario base + adaptacion proyecto + adaptacion perfil + estructura sesion. Consultar `references/contenido_por_app.md` y `references/perfiles_usuario.md`.
3. **Validar plan sesion** (C1.5): STOP obligatorio. Presentar temario, bloques, ejercicios. Esperar aprobacion.
4. **Generar materiales** (C2): Guia sesion, screenshots (placeholders), cheat sheet, cuestionario evaluacion.
5. **Validar materiales** (C2.5): STOP obligatorio. Presentar resumen. Esperar aprobacion.
6. **Entregables** (C3): Guardar 3 ficheros Markdown (plan, guia, cheat sheet).

See references/shared/odoo-connection.md for connection pattern.

### Key Methodology Rules

- Regla 50/50: al menos 50% del tiempo en ejercicios practicos.
- Regla de los 20 minutos: max 20min teoria seguida antes de practica.
- Ejercicios SIEMPRE con datos reales del entorno del cliente, NUNCA genericos.
- Screenshots POR DEFECTO SI. Solo omitir si usuario dice explicitamente que no.
- Toda semana del proyecto debe tener una entrada (formacion o reunion de seguimiento).

## GOTCHAS

1. **Semanas vacias en el calendario**: Toda semana del proyecto DEBE tener una entrada. Si falta una reunion de seguimiento, el plan parece incompleto. Siempre rellenar.
2. **Multiples `write` al body de Knowledge**: Causa corrupcion del historial del editor en Odoo 19. SIEMPRE usar `create` una sola vez.
3. **Datos genericos en ejercicios**: "Cliente 1", "Producto A" destruyen la credibilidad. SIEMPRE usar datos reales del entorno del cliente.
4. **Checklists duplicados**: Si ya existen checklists para formaciones en la tarea, crear nuevos genera duplicados. Verificar existentes en P1.
5. **Perfiles mezclados sin aprobacion**: Un operario y un director en la misma sesion hacen imposible el nivel correcto. NUNCA mezclar sin aprobacion.
6. **Formacion antes de configuracion terminada**: Si la tarea de configuracion no ha terminado, la formacion muestra funcionalidad incompleta. Programar la semana posterior a `planned_date_end`.
7. **CSS externo en articulo Knowledge**: Odoo no procesa clases CSS ni Bootstrap en Knowledge. Todo debe ser estilos inline.

## Delegates to

| Skill | Suite | When | Via |
|-------|-------|------|-----|
| odoo-pilot | common | CRUD de checklists, lectura tareas/Knowledge | Bash inline |
| odoo-screenshot-generator | common | Screenshots para materiales de sesion | Bash inline |

## Consumed by

Standalone. Invocado por consultores al planificar formaciones o preparar sesiones.

## References

| File | Content | Load when |
|------|---------|-----------|
| `references/mode-planificar-detail.md` | Pasos detallados modo PLANIFICAR | Mode A |
| `references/mode-contenido-detail.md` | Pasos detallados modo CONTENIDO | Mode B |
| `references/reglas_planificacion.md` | Reglas agrupacion, fechas, calendario | Mode A, Paso P2 |
| `references/contenido_por_app.md` | Temario estandar por aplicacion Odoo | Mode B, Paso C1 |
| `references/perfiles_usuario.md` | Perfiles de usuario y adaptacion formativa | Mode B, Paso C1 |
| `references/buenas_practicas_formacion.md` | Reglas 50/50, 20min, scaffolding | Mode B, Paso C1 |
