## Limpieza de Audit Trail

Eliminar entradas de audit trail generadas durante testing.

### mail.tracking.value (siempre disponible)

```sql
-- Borrar tracking values asociados a mensajes de test
-- Ejecutar ANTES de borrar mail.message
DELETE FROM mail_tracking_value WHERE mail_message_id IN (
    SELECT id FROM mail_message
    WHERE create_date >= @test_start
      AND model IN ('sale.order', 'purchase.order', 'account.move',
                     'stock.picking', 'crm.lead', 'project.task')
      AND res_id IN (@record_ids_per_model)
);
```

### auditlog.log (si modulo auditlog instalado)

```sql
-- Verificar si el modulo auditlog esta instalado
SELECT name, state FROM ir_module_module WHERE name = 'auditlog' AND state = 'installed';

-- Si esta instalado, borrar logs de test
-- Primero las lineas de detalle
DELETE FROM auditlog_log_line WHERE log_id IN (
    SELECT id FROM auditlog_log
    WHERE create_date >= @test_start
      AND model_id IN (
          SELECT id FROM ir_model
          WHERE model IN ('sale.order', 'purchase.order', 'account.move',
                          'stock.picking', 'crm.lead', 'project.task')
      )
      AND res_id IN (@record_ids_per_model)
);

-- Luego los logs principales
DELETE FROM auditlog_log
WHERE create_date >= @test_start
  AND model_id IN (
      SELECT id FROM ir_model
      WHERE model IN ('sale.order', 'purchase.order', 'account.move',
                      'stock.picking', 'crm.lead', 'project.task')
  )
  AND res_id IN (@record_ids_per_model);
```

### Verificacion post-limpieza de audit trail

```sql
-- Verificar que no quedan tracking values huerfanos
SELECT 'tracking_sin_message' AS tipo, COUNT(*)
FROM mail_tracking_value mtv
WHERE NOT EXISTS (SELECT 1 FROM mail_message WHERE id = mtv.mail_message_id);
-- Debe ser 0

-- Verificar que no quedan auditlog_log_line huerfanos (si modulo instalado)
SELECT 'auditlog_line_sin_log' AS tipo, COUNT(*)
FROM auditlog_log_line all2
WHERE NOT EXISTS (SELECT 1 FROM auditlog_log WHERE id = all2.log_id);
-- Debe ser 0
```

---

