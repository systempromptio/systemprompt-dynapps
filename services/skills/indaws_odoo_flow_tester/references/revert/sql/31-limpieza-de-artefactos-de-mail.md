## Limpieza de Artefactos de Mail

Eliminar registros de mail generados durante testing. Ejecutar DESPUES de eliminar los
registros principales pero ANTES de la verificacion final.

### Limpieza via ORM (Nivel 1 — preferido)

```bash
# Buscar y eliminar mail.mail generados durante el test
# @test_start = timestamp ISO del inicio del test
# @model_list = lista de modelos afectados
# @record_ids = IDs de los registros de test

# 1. Mail.mail (emails encolados/enviados)
MAIL_IDS=$(search_records.sh mail.mail '[
    ["create_date",">=","'$test_start'"],
    ["model","in",'$model_list'],
    ["res_id","in",'$record_ids']
]' '["id"]' 0)
delete_record.sh mail.mail "[$MAIL_IDS]"

# 2. Mail.message (mensajes del chatter)
MSG_IDS=$(search_records.sh mail.message '[
    ["create_date",">=","'$test_start'"],
    ["model","in",'$model_list'],
    ["res_id","in",'$record_ids']
]' '["id"]' 0)
delete_record.sh mail.message "[$MSG_IDS]"

# 3. Mail.followers (seguidores automaticos)
FOLLOWER_IDS=$(search_records.sh mail.followers '[
    ["res_model","in",'$model_list'],
    ["res_id","in",'$record_ids']
]' '["id"]' 0)
delete_record.sh mail.followers "[$FOLLOWER_IDS]"

# 4. Mail.tracking.value (valores de seguimiento de cambios)
TRACKING_IDS=$(search_records.sh mail.tracking.value '[
    ["mail_message_id","in",'$MSG_IDS']
]' '["id"]' 0)
delete_record.sh mail.tracking.value "[$TRACKING_IDS]"
```

### Limpieza via SQL (Nivel 3 — fallback)

```sql
BEGIN;

-- @test_start = timestamp del inicio del test (formato: 'YYYY-MM-DD HH:MM:SS')
-- @test_record_ids = IDs de registros de test por modelo

-- 1. mail.tracking.value (depende de mail.message)
DELETE FROM mail_tracking_value WHERE mail_message_id IN (
    SELECT id FROM mail_message
    WHERE create_date >= @test_start
      AND model IN ('sale.order', 'purchase.order', 'account.move', 'stock.picking',
                     'crm.lead', 'project.task', 'hr.expense.sheet', 'mrp.production')
      AND (
          (model = 'sale.order' AND res_id IN (@so_ids))
          OR (model = 'purchase.order' AND res_id IN (@po_ids))
          OR (model = 'account.move' AND res_id IN (@move_ids))
          OR (model = 'stock.picking' AND res_id IN (@picking_ids))
          OR (model = 'crm.lead' AND res_id IN (@lead_ids))
          OR (model = 'project.task' AND res_id IN (@task_ids))
          OR (model = 'hr.expense.sheet' AND res_id IN (@sheet_ids))
          OR (model = 'mrp.production' AND res_id IN (@production_ids))
      )
);

-- 2. mail.mail (emails encolados/enviados)
DELETE FROM mail_mail WHERE id IN (
    SELECT mm.id FROM mail_mail mm
    JOIN mail_message msg ON mm.mail_message_id = msg.id
    WHERE msg.create_date >= @test_start
      AND msg.model IN ('sale.order', 'purchase.order', 'account.move', 'stock.picking')
      AND (
          (msg.model = 'sale.order' AND msg.res_id IN (@so_ids))
          OR (msg.model = 'purchase.order' AND msg.res_id IN (@po_ids))
          OR (msg.model = 'account.move' AND msg.res_id IN (@move_ids))
          OR (msg.model = 'stock.picking' AND msg.res_id IN (@picking_ids))
      )
);

-- 3. mail.message
DELETE FROM mail_message
WHERE create_date >= @test_start
  AND model IN ('sale.order', 'purchase.order', 'account.move', 'stock.picking',
                 'crm.lead', 'project.task', 'hr.expense.sheet', 'mrp.production')
  AND (
      (model = 'sale.order' AND res_id IN (@so_ids))
      OR (model = 'purchase.order' AND res_id IN (@po_ids))
      OR (model = 'account.move' AND res_id IN (@move_ids))
      OR (model = 'stock.picking' AND res_id IN (@picking_ids))
      OR (model = 'crm.lead' AND res_id IN (@lead_ids))
      OR (model = 'project.task' AND res_id IN (@task_ids))
      OR (model = 'hr.expense.sheet' AND res_id IN (@sheet_ids))
      OR (model = 'mrp.production' AND res_id IN (@production_ids))
  );

-- 4. mail.followers
DELETE FROM mail_followers
WHERE (
    (res_model = 'sale.order' AND res_id IN (@so_ids))
    OR (res_model = 'purchase.order' AND res_id IN (@po_ids))
    OR (res_model = 'account.move' AND res_id IN (@move_ids))
    OR (res_model = 'stock.picking' AND res_id IN (@picking_ids))
    OR (res_model = 'crm.lead' AND res_id IN (@lead_ids))
    OR (res_model = 'project.task' AND res_id IN (@task_ids))
    OR (res_model = 'hr.expense.sheet' AND res_id IN (@sheet_ids))
    OR (res_model = 'mrp.production' AND res_id IN (@production_ids))
);

ROLLBACK; -- Cambiar a COMMIT solo tras verificacion
```

---

