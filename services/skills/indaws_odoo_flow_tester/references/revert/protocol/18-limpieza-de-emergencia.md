## Limpieza de Emergencia

Procedimiento a seguir cuando la reversion estandar (Tier 1 + Tier 2 + Tier 3) falla completamente
o cuando el tracking JSON se ha perdido.

### Paso 1: Busqueda por prefijo

Localizar todos los registros de test usando el prefijo `[TEST-YYYYMMDD-HHMM]`:

```bash
# Buscar en TODOS los modelos conocidos (25+)
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
```

### Paso 2: Reversion ORM por orden de dependencias

Con los registros encontrados, ejecutar la reversion siguiendo el orden de procesamiento (seccion superior).
Para cada modelo, usar la estrategia de Tier 1 correspondiente.

### Paso 3: Si ORM falla → escalar a Tier 2/3

Si la reversion ORM no funciona para algun registro, consultar `references/revert/sql/` para niveles 2 y 3.

### Paso 4: Restaurar integraciones

Si se desactivaron integraciones durante el test (servidores de correo, payment providers,
automatizaciones, etc.), restaurarlas independientemente del resultado de la limpieza.

> Ver `integration_safety.md` para el protocolo completo de restauracion

```bash
# Restaurar servidores de correo
update_record.sh ir.mail_server '[<ids>]' '{"active": true}'

# Restaurar payment providers
update_record.sh payment.provider '[<ids>]' '{"state": "<estado_original>"}'

# Restaurar automatizaciones
update_record.sh base.automation '[<ids>]' '{"active": true}'

# Restaurar delivery carriers
update_record.sh delivery.carrier '[<ids>]' '{"prod_environment": true}'

# Restaurar fetchmail servers
update_record.sh fetchmail.server '[<ids>]' '{"active": true}'
```

### Paso 5: Verificar saldos contables

```bash
search_records.sh account.account '[]' '["code","name","current_balance"]' 0
# Comparar con pre_state
# Si hay diferencias, identificar asientos causantes
search_records.sh account.move '[["ref","like","[TEST-"]]' '["id","name","ref","state","amount_total"]' 0
```

### Paso 6: Verificar niveles de stock

```bash
search_records.sh product.product '[["type","=","product"]]' '["name","qty_available","virtual_available"]' 0
# Comparar con pre_state
# Si hay diferencias, verificar quants
search_records.sh stock.quant '[["location_id.usage","=","internal"]]' '["product_id","location_id","quantity"]' 0
```

> Ver `email_safety.md` para limpieza de emails generados durante el test

---

