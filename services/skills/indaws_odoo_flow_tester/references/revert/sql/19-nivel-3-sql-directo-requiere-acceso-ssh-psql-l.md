### Limpieza SQL de hr.leave (verificado)

```sql
BEGIN;
-- Variables: @leave_ids = IDs de hr_leave a borrar

-- Paso 1: Borrar resource.calendar.leaves asociados
DELETE FROM resource_calendar_leaves WHERE holiday_id IN (@leave_ids);

-- Paso 2: Limpiar auxiliares
DELETE FROM mail_message WHERE model = 'hr.leave' AND res_id IN (@leave_ids);

-- Paso 3: Borrar hr.leave
DELETE FROM hr_leave WHERE id IN (@leave_ids);

ROLLBACK;
```

