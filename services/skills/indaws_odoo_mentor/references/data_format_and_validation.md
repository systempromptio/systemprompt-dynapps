# Data Formatting, Validations & Success Criteria

Reference document with formatting rules, validation checklist, and success criteria.

---

## Datos y Formato

- Porcentajes: 1 decimal (ej: 73.2%)
- Horas: 2 decimales (ej: 12.50h)
- Health Score: entero 0-100
- Fechas: DD/MM/YYYY en el output
- Iconos de estado: 🟢 (>=80) 🟡 (70-79) 🟠 (40-69) 🔴 (<40)
- Tendencias: ↑ (subió >=5pts), ↓ (bajó >=5pts), → (estable)

## Validaciones

1. Credenciales cargadas y conexión verificada
2. Rol detectado correctamente (o impersonado)
3. Proyectos/CRM descubiertos según el rol
4. Health scores calculados con los scripts existentes (nunca manualmente)
5. Nudges clasificados por `calculate_nudge.py`
6. Reglas anti-molestia aplicadas
7. Estado actualizado después de cada check
8. Chatter notes publicadas solo para CRITICAL/WARNING (no en modo impersonación)
9. Loop configurado en sesión matutina
10. MEMORY.md actualizado al final del día

## Success Criteria

- El usuario recibe un briefing matutino completo adaptado a su rol
- Los checks de mediodía/tarde son breves o silenciosos si no hay cambios
- Las alertas CRITICAL se comunican inmediatamente con acción concreta
- Se sugieren los skills correctos para cada situación detectada
- El resumen EOD refleja la evolución del día
- Los viernes se genera revisión semanal y se sugiere `/project-weekly-update`
- El estado persiste entre sesiones via `state.json` y `MEMORY.md`
- En modo admin, se puede impersonar cualquier empleado sin efectos secundarios
