# Recordatorios Proactivos de Gates

> Fuente: F13 del Migration Assistant
> Este fichero define umbrales de alerta, mensajes personalizados por rol y gate,
> alertas de proximidad de Go-Live y patrón de escalación.
> Se usa automáticamente dentro de F8 y F9.

---

## 1. Umbrales de alerta

| Condición | Nivel | Cuándo |
|-----------|-------|--------|
| Gate pendiente > 2 semanas | WARNING | `hoy - date_last_stage_update > 14 días` |
| Gate pendiente > 4 semanas | CRITICAL | `hoy - date_last_stage_update > 28 días` |
| Fase activa > 4 semanas sin avance en tareas | ALERT | Ninguna tarea completada en últimas 4 semanas |
| Go-Live en < 4 semanas y gates pendientes | URGENTE | `xx_golive_date - hoy < 28 días` AND gates sin aprobar |
| Gate rechazado sin re-evaluación > 1 semana | WARNING | Gate en estado "rechazado" > 7 días |
| Go-Live posterior a la retirada de Odoo.sh (solo hosting Odoo.sh) | CRITICAL | `date_golive > fecha_retirada_SH` de la versión origen — deadline externo duro |

> **Deadline externo (hosting Odoo.sh):** la versión origen sale de Odoo.sh en una fecha fija y el Go-Live DEBE ser
> anterior (el entorno deja de poder alojarse en Odoo.sh). Calendario de retirada por versión y regla del +25% de
> renovación Enterprise en `references/shared/odoo-lifecycle-licensing.md`.

---

## 2. Cálculo

- **Tiempo pendiente gate:** `hoy - date_last_stage_update` (en días, convertir a semanas)
- Si el gate no tiene `date_last_stage_update`, usar `create_date`
- **Días hasta Go-Live:** `xx_golive_date - hoy` (query a `project.project`)

### Query Go-Live

```python
# Obtener fecha Go-Live del proyecto
project = models.execute_kw(db, uid, password, 'project.project', 'read',
    [project_id], {'fields': ['xx_golive_date']})
go_live_date = project[0].get('xx_golive_date')
```

---

## 3. Tabla de recordatorios por rol y gate

### Gate 0 — Alcance y Propuesta Comercial

| Rol | Tipo | WARNING (> 2 sem) | CRITICAL (> 4 sem) |
|-----|------|--------------------|---------------------|
| PM | Ejecutor | Gate 0 lleva {N} semanas pendiente. Verifica que la propuesta comercial está revisada (BC + PM + TL), el inventario de módulos completado y la firma del cliente obtenida. | URGENTE: Gate 0 lleva {N} semanas sin avance. Sin propuesta firmada no se puede iniciar la migración. Escala a COO si hay bloqueo con el cliente. |
| COO | Aprobador | Gate 0 pendiente de tu aprobación desde hace {N} semanas. Revisa la propuesta comercial y el documento de alcance con PM. | URGENTE: Gate 0 bloqueado {N} semanas. El proyecto no puede arrancar sin tu aprobación. Agenda revisión inmediata con PM. |

### Gate 1 — Análisis Técnico

| Rol | Tipo | WARNING (> 2 sem) | CRITICAL (> 4 sem) |
|-----|------|--------------------|---------------------|
| Dev | Ejecutor | Gate 1 lleva {N} semanas pendiente. Verifica que el análisis de código está completado (APIs deprecadas, vulnerabilidades, rendimiento), módulos clasificados y rama release creada. | URGENTE: Gate 1 lleva {N} semanas. Sin análisis técnico completado no se puede empezar a migrar código. Completa la clasificación de módulos y la ficha de infraestructura. |
| TL | Aprobador | Gate 1 pendiente de tu aprobación desde hace {N} semanas. Revisa el análisis de código y la clasificación de módulos con Dev. | URGENTE: Gate 1 bloqueado {N} semanas. Agenda revisión del análisis técnico con Dev para desbloquear la migración. |

### Gate 2 — Código Migrado

| Rol | Tipo | WARNING (> 2 sem) | CRITICAL (> 4 sem) |
|-----|------|--------------------|---------------------|
| Dev | Ejecutor | Gate 2 lleva {N} semanas pendiente. Verifica que todos los módulos están migrados, pipeline CI/CD green, coverage ≥ 80% y code review aprobado. | URGENTE: Gate 2 lleva {N} semanas. La migración de código debe completarse para avanzar a BD. Revisa módulos pendientes y tests unitarios faltantes. |
| TL | Aprobador | Gate 2 pendiente de tu aprobación desde hace {N} semanas. Agenda revisión de code review con Dev. | URGENTE: Gate 2 bloqueado {N} semanas. Sin aprobación de código no se puede migrar la BD. Prioriza el code review. |

### Gate 3 — Base de Datos Migrada

| Rol | Tipo | WARNING (> 2 sem) | CRITICAL (> 4 sem) |
|-----|------|--------------------|---------------------|
| Dev | Ejecutor | Gate 3 lleva {N} semanas pendiente. Verifica que la BD está migrada en testing, scripts pre/post-migrate documentados y validación de datos completada. | URGENTE: Gate 3 lleva {N} semanas. Sin BD migrada no se puede iniciar testing funcional. Completa la validación de datos y resuelve errores del log. |
| TL | Aprobador | Gate 3 pendiente de tu aprobación desde hace {N} semanas. Revisa el log de migración y la validación de datos con Dev. | URGENTE: Gate 3 bloqueado {N} semanas. La BD migrada es prerrequisito del testing funcional. Agenda revisión inmediata. |

### Gate 4 — Testing Funcional (FASE CRÍTICA)

| Rol | Tipo | WARNING (> 2 sem) | CRITICAL (> 4 sem) |
|-----|------|--------------------|---------------------|
| BC | Ejecutor | Gate 4 lleva {N} semanas pendiente. Verifica el checklist funcional (9 items): flujos de venta, compra, inventario, contabilidad e integraciones. | URGENTE: Gate 4 lleva {N} semanas. Los checklists funcional y técnico deben completarse para avanzar. Coordina con Dev para resolver incidencias KO pendientes. |
| Dev | Ejecutor | Gate 4 lleva {N} semanas pendiente. Verifica el checklist técnico (7 items): módulos/logs/rendimiento, vistas y componentes JS/OWL, cron jobs, informes PDF, integraciones, emails, permisos. | URGENTE: Gate 4 lleva {N} semanas. Completa el checklist técnico y resuelve errores de log y rendimiento. El testing funcional es la fase más crítica. |
| TL | Aprobador | Gate 4 pendiente de tu aprobación desde hace {N} semanas. Revisa los criterios GO/NO-GO con BC y Dev. | URGENTE: Gate 4 bloqueado {N} semanas. Sin aprobación del testing funcional el proyecto no puede avanzar al rehearsal. Agenda sesión GO/NO-GO. |

### Gate 5 — Ensayo General (Rehearsal)

| Rol | Tipo | WARNING (> 2 sem) | CRITICAL (> 4 sem) |
|-----|------|--------------------|---------------------|
| Dev | Ejecutor | Gate 5 lleva {N} semanas pendiente. Verifica que el rehearsal se ha ejecutado, tiempos medidos, plan de rollback documentado y smoke tests funcionando. | URGENTE: Gate 5 lleva {N} semanas. Sin rehearsal aprobado no se puede hacer Go-Live. Ejecuta el ensayo y completa el checklist de infraestructura. |
| TL | Aprobador | Gate 5 pendiente de tu aprobación desde hace {N} semanas. Revisa los tiempos del rehearsal y firma el checklist de infraestructura. | URGENTE: Gate 5 bloqueado {N} semanas. Firma el checklist de infraestructura y valida que los tiempos caben en la ventana de mantenimiento. |

### Gate 6 — Puesta en Producción (Go-Live)

| Rol | Tipo | WARNING (> 2 sem) | CRITICAL (> 4 sem) |
|-----|------|--------------------|---------------------|
| BC | Ejecutor | Gate 6 lleva {N} semanas pendiente. Verifica prerrequisitos: checklists de Fase 4 y 5 completados, protocolo go-live aprobado y ventana comunicada al cliente. | URGENTE: Gate 6 lleva {N} semanas. El Go-Live está bloqueado. Confirma que todos los prerrequisitos están cumplidos y coordina con Dev la ejecución. |
| Dev | Ejecutor | Gate 6 lleva {N} semanas pendiente. Verifica: backup producción realizado, equipo de guardia confirmado y plan de ejecución listo (siguiendo tiempos del rehearsal). | URGENTE: Gate 6 lleva {N} semanas. Prepara backup, confirma equipo guardia y asegura que el plan de ejecución sigue los tiempos del rehearsal. |
| TL | Aprobador | Gate 6 pendiente de tu aprobación desde hace {N} semanas. Revisa prerrequisitos y protocolo go-live con BC y Dev. | URGENTE: Gate 6 bloqueado {N} semanas. Sin aprobación no se ejecuta el go-live. Valida prerrequisitos con el equipo. |
| CTO Ops | Aprobador | Gate 6 pendiente de tu aprobación desde hace {N} semanas. Revisa la infraestructura y el plan de go-live. | URGENTE: Gate 6 bloqueado {N} semanas. Tu aprobación es necesaria junto con TL para proceder al go-live. |

### Gate 7 — Cierre del Proyecto

| Rol | Tipo | WARNING (> 2 sem) | CRITICAL (> 4 sem) |
|-----|------|--------------------|---------------------|
| PM | Ejecutor | Gate 7 lleva {N} semanas pendiente. Verifica que el período de soporte (2 semanas) ha concluido, incidencias resueltas y retrospectiva realizada. | URGENTE: Gate 7 lleva {N} semanas. El cierre formal del proyecto está retrasado. Completa la retrospectiva y comunica el cierre al cliente. |
| TL | Ejecutor | Gate 7 lleva {N} semanas pendiente. Verifica que las lecciones aprendidas están documentadas y la guía de migración actualizada. | URGENTE: Gate 7 lleva {N} semanas. Documenta las lecciones aprendidas y actualiza la guía de migración para cerrar el proyecto. |
| BC | Ejecutor | Gate 7 lleva {N} semanas pendiente. Verifica que todas las incidencias post-migración están resueltas y el cliente está conforme. | URGENTE: Gate 7 lleva {N} semanas. Confirma con el cliente que la operación es normal y obtén su conformidad para el cierre formal. |
| COO | Aprobador | Gate 7 pendiente de tu aprobación desde hace {N} semanas. Revisa el resumen del proyecto (horas, desviación, incidencias) con PM. | URGENTE: Gate 7 bloqueado {N} semanas. El proyecto no se puede cerrar formalmente sin tu aprobación. Agenda revisión final. |

---

## 4. Alerta de proximidad de Go-Live

Cuando `xx_golive_date` está a < 4 semanas y quedan gates por aprobar:

### Mensaje general
> **Go-Live previsto en {N} días ({fecha}).** Quedan {M} gates por aprobar: {lista_gates}.

### Por fase actual
> Para cumplir con la fecha de Go-Live, la Fase {N} debería completarse antes del {fecha_límite_calculada}.

### Cálculo de fecha límite por fase

Distribuir el tiempo restante proporcionalmente entre las fases pendientes:

```
días_por_fase = días_hasta_golive / fases_pendientes
fecha_límite_fase_N = hoy + (días_por_fase × posición_relativa)
```

---

## 5. Patrón de escalación

| Nivel | Acción | Notificados |
|-------|--------|-------------|
| 1. WARNING | Recordatorio en informe | Ejecutor + Aprobador del gate |
| 2. CRITICAL | Recordatorio urgente en informe | Ejecutor + Aprobador + PM informado |
| 3. Go-Live < 2 semanas con gates pendientes | Alerta de riesgo de proyecto | + COO/CTO alertado |

---

## 6. Formato de inyección en informes F8/F9

Sección que se añade al final del informe cuando hay alertas:

```markdown
---

### Recordatorios de Gates

| Gate | Semanas pendiente | Nivel | Recordatorio |
|------|-------------------|-------|--------------|
| Gate 2 | 3.5 | WARNING | {mensaje personalizado según rol del usuario} |
| Gate 4 | 5.2 | CRITICAL | {mensaje personalizado según rol del usuario} |

**Go-Live:** {fecha} (en {N} días) — {M} gates pendientes
```

### Reglas de formato

1. Solo incluir esta sección si hay al menos una alerta activa
2. Ordenar gates por nivel de severidad (CRITICAL primero, luego WARNING)
3. Personalizar el mensaje según el rol del usuario que solicita el informe
4. Si hay alerta de Go-Live, SIEMPRE incluir la línea de Go-Live aunque no haya gates en WARNING/CRITICAL
5. Si el usuario no tiene rol asignado, usar el mensaje del ejecutor del gate
