## Verificacion Post-Revert Mejorada

Despues de la reversion, ejecutar TODAS estas verificaciones:

### Checklist Completo

```
[ ] 1. Todos los registros con prefijo TEST eliminados o neutralizados
[ ] 2. Saldos contables coinciden con snapshot
[ ] 3. Stock quantities coinciden con snapshot
[ ] 4. Secuencias restauradas a valores pre-test
[ ] 5. No quedan mail.mail residuales de test
[ ] 6. No quedan followers de test (count coincide con snapshot)
[ ] 7. No quedan activities de test (count coincide con snapshot)
[ ] 8. Integraciones restauradas a estado original
[ ] 9. No quedan registros cascade huerfanos
[ ] 10. No quedan ir.attachment de test
[ ] 11. stock.quant coherente (sin cantidades fantasma)
[ ] 12. No quedan account.partial.reconcile huerfanos
```

### 1. Verificar registros eliminados

```bash
# Para cada registro tracked:
RESULT=$(search_records.sh <model> '[["id","=",<id>]]' '["id"]' 1)
# Si devuelve resultado → FALLO, registro no eliminado

# Busqueda por prefijo en TODOS los modelos
search_records.sh sale.order '[["client_order_ref","like","[TEST-20260309-"]]' '["id","name","state"]' 0
search_records.sh purchase.order '[["partner_ref","like","[TEST-20260309-"]]' '["id","name","state"]' 0
search_records.sh account.move '[["ref","like","[TEST-20260309-"]]' '["id","name","state"]' 0
search_records.sh account.payment '[["ref","like","[TEST-20260309-"]]' '["id","name","state"]' 0
search_records.sh stock.picking '[["origin","like","[TEST-20260309-"]]' '["id","name","state"]' 0
search_records.sh crm.lead '[["name","like","[TEST-20260309-"]]' '["id","name"]' 0
search_records.sh project.task '[["name","like","[TEST-20260309-"]]' '["id","name"]' 0
search_records.sh mrp.production '[["origin","like","[TEST-20260309-"]]' '["id","name","state"]' 0
search_records.sh hr.payslip '[["name","like","[TEST-20260309-"]]' '["id","name","state"]' 0
search_records.sh hr.leave '[["name","like","[TEST-20260309-"]]' '["id","name"]' 0
search_records.sh hr.contract '[["name","like","[TEST-20260309-"]]' '["id","name","state"]' 0
search_records.sh mailing.mailing '[["subject","like","[TEST-20260309-"]]' '["id","subject","state"]' 0
search_records.sh pos.order '[["pos_reference","like","[TEST-20260309-"]]' '["id","name","state"]' 0
search_records.sh account.asset '[["name","like","[TEST-20260309-"]]' '["id","name"]' 0
search_records.sh helpdesk.ticket '[["name","like","[TEST-20260309-"]]' '["id","name"]' 0
search_records.sh quality.check '[["title","like","[TEST-20260309-"]]' '["id","title"]' 0
search_records.sh maintenance.request '[["name","like","[TEST-20260309-"]]' '["id","name"]' 0
search_records.sh documents.document '[["name","like","[TEST-20260309-"]]' '["id","name"]' 0
search_records.sh planning.slot '[["name","like","[TEST-20260309-"]]' '["id","name"]' 0
search_records.sh approval.request '[["name","like","[TEST-20260309-"]]' '["id","name"]' 0
search_records.sh fleet.vehicle.log.services '[["description","like","[TEST-20260309-"]]' '["id","description"]' 0
# Todos deben devolver lista vacia []
```

### 2. Comparar saldos contables

```bash
CURRENT_BALANCES=$(search_records.sh account.account '[]' '["code","current_balance"]' 0)
# Diff contra pre_state.account_balances
# Tolerancia: diferencia maxima 0.01 por cuenta
```

### 3. Comparar cantidades de stock

```bash
CURRENT_STOCK=$(search_records.sh product.product '[["type","=","product"]]' '["name","qty_available","virtual_available"]' 0)
# Diff contra pre_state.stock_quantities
# Tolerancia: diferencia maxima 0.001 por producto
```

### 4. Comparar secuencias

```bash
CURRENT_SEQUENCES=$(search_records.sh ir.sequence '[["code","in",["sale.order","purchase.order","account.move","stock.picking","mrp.production","pos.order","hr.payslip"]]]' '["id","code","number_next"]' 0)
# Diff contra pre_state.sequences
# Si number_next ha aumentado → restaurar con update_record si es seguro (ver seccion Secuencias)
```

### 5. Verificar mail.mail residuales

```bash
# Contar mail.mail actuales
CURRENT_MAIL_COUNT=$(search_records.sh mail.mail '[]' '["id"]' 0)
# Comparar con pre_state.mail_mail_count
# Si hay mas → buscar y borrar emails generados despues del timestamp del test
NEW_MAILS=$(search_records.sh mail.mail '[["create_date",">=","<test_start_datetime>"]]' '["id","subject","state"]' 0)
# Borrar emails de test
delete_record.sh mail.mail '[<new_mail_ids>]'
```

### 6. Verificar followers residuales

```bash
# Comparar conteos por modelo
CURRENT_FOLLOWERS=$(search_records.sh mail.followers '[["res_model","in",["sale.order","purchase.order","account.move","stock.picking","crm.lead","project.task","helpdesk.ticket"]]]' '["res_model"]' 0)
# Diff contra pre_state.followers_count
# Si hay mas followers → buscar los que se crearon despues del test
NEW_FOLLOWERS=$(search_records.sh mail.followers '[["create_date",">=","<test_start_datetime>"],["res_model","in",["sale.order","purchase.order"]]]' '["id","res_model","res_id"]' 0)
```

### 7. Verificar activities residuales

```bash
CURRENT_ACTIVITIES=$(search_records.sh mail.activity '[["res_model","in",["sale.order","purchase.order","account.move","stock.picking","crm.lead","project.task","helpdesk.ticket"]]]' '["id","res_model"]' 0)
# Diff contra pre_state.activities_count
```

### 8. Verificar integraciones restauradas

```bash
# Payment providers
search_records.sh payment.provider '[["state","!=","disabled"]]' '["id","name","state"]' 0
# Comparar con pre_state.integrations.payment_providers

# Delivery carriers
search_records.sh delivery.carrier '[["active","=",true],["prod_environment","=",true]]' '["id","name","prod_environment"]' 0
# Comparar con pre_state.integrations.delivery_carriers

# Automatizaciones
search_records.sh base.automation '[["active","=",true]]' '["id","name"]' 0
# Comparar con pre_state.integrations.automations_active

# Mail servers
search_records.sh ir.mail_server '[["active","=",true]]' '["id","name"]' 0
# Comparar con pre_state.integrations.mail_servers_active

# Fetchmail
search_records.sh fetchmail.server '[["active","=",true]]' '["id","name"]' 0
# Comparar con pre_state.integrations.fetchmail_servers_active
```

### 9. Verificar registros cascade huerfanos

```bash
# stock.move sin picking (posible huerfano)
search_records.sh stock.move '[["picking_id","=",false],["create_date",">=","<test_start_datetime>"]]' '["id","name","state"]' 0

# account.move.line sin move padre valido (raro pero posible)
search_records.sh account.move.line '[["create_date",">=","<test_start_datetime>"],["move_id.ref","like","[TEST-"]]' '["id","move_id"]' 0

# mailing.trace sin mailing padre
search_records.sh mailing.trace '[["create_date",">=","<test_start_datetime>"]]' '["id","mass_mailing_id"]' 0
```

### 10. Verificar adjuntos residuales

```bash
# Buscar attachments creados durante el test
search_records.sh ir.attachment '[["create_date",">=","<test_start_datetime>"],["res_model","in",["sale.order","purchase.order","account.move","stock.picking","crm.lead","project.task","hr.expense","pos.order","helpdesk.ticket"]]]' '["id","res_model","res_id","name"]' 0
# Borrar los que referencian registros ya eliminados
```

### 11. Verificar stock.quant coherente

```bash
# Comparar quants actuales con snapshot
search_records.sh stock.quant '[["location_id.usage","=","internal"]]' '["product_id","location_id","quantity","reserved_quantity"]' 0
# Diff contra pre_state.stock_quant_totals
# Tolerancia: 0.001 por producto/ubicacion
# Si hay diferencias → verificar que la devolucion se proceso correctamente
```

### 12. Verificar reconciliaciones huerfanas

```bash
# Buscar partials creados durante el test que sobrevivieron
search_records.sh account.partial.reconcile '[["create_date",">=","<test_start_datetime>"]]' '["id","debit_move_id","credit_move_id","amount"]' 0
# Si hay resultados → investigar si el pago/factura fue realmente desreconciliado
```

---

