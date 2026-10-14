# Gate Detection — Phase Detection via Gates

## Detection Logic

```
1. Search [GATE] tasks in the project:
   domain = [
     ['project_id', '=', project_id],
     '|',
     ['name', 'ilike', '[GATE]'],
     ['name', 'ilike', 'Gate ']
   ]
   fields = ['id', 'name', 'stage_id', 'date_last_stage_update']

2. Classify each gate as:
   - APPROVED: stage contains "done", "hecho", "approved", "cerrado"
   - PENDING: any other stage

3. Determine current phase:
   - Gate 0 approved → current phase = 1
   - Gate 1 approved → current phase = 2
   - Gate 2 approved → current phase = 3
   - ...
   - No gates approved → current phase = 0
   - All gates (0-7) approved → project closed (phase 7 completed)

4. Report to user:
   "Según los gates en Odoo, estáis en Fase {N} ({phase_name}).
    Gates aprobados: 0, 1, 2. Pendiente: Gate 3."
```

## Gate → Phase Mapping Table

| Last gate approved | Current phase | Name |
|---|---|---|
| None | Phase 0 | Kick-off, Alcance y Propuesta |
| Gate 0 | Phase 1 | Preparación Técnica |
| Gate 1 | Phase 2 | Migración de Código |
| Gate 2 | Phase 3 | Migración de Base de Datos |
| Gate 3 | Phase 4 | Testing Funcional |
| Gate 4 | Phase 5 | Ensayo General (Rehearsal) |
| Gate 5 | Phase 6 | Puesta en Producción |
| Gate 6 | Phase 7 | Soporte Post-migración y Cierre |
| Gate 7 | (closed) | Proyecto completado |
