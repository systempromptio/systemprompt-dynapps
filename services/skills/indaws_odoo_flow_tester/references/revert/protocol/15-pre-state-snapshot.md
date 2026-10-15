## Pre-State Snapshot

Antes de ejecutar en produccion, capturar TODOS estos datos:

### Saldos Contables
```bash
search_records.sh account.account '[]' '["code","name","current_balance"]' 0
```

### Balances por cuenta (account.move.line)
```bash
# Sumas de debito/credito por cuenta para moves publicados
search_records.sh account.move.line '[["parent_state","=","posted"]]' '["account_id","debit","credit","balance"]' 0
```

### Cantidades de Stock
```bash
search_records.sh product.product '[["type","=","product"]]' '["name","qty_available","virtual_available"]' 0
```

### Stock Quant Totals
```bash
# Cantidades por producto y ubicacion interna
search_records.sh stock.quant '[["location_id.usage","=","internal"]]' '["product_id","location_id","quantity","reserved_quantity"]' 0
```

### Secuencias (ir.sequence)
```bash
search_records.sh ir.sequence '[["code","in",["sale.order","purchase.order","account.move","stock.picking","mrp.production","pos.order","hr.payslip"]]]' '["id","code","name","number_next","prefix"]' 0
```

### Secuencias con rangos de fecha
```bash
search_records.sh ir.sequence.date_range '[["sequence_id.code","in",["sale.order","purchase.order","account.move"]]]' '["id","sequence_id","date_from","date_to","number_next"]' 0
```

### Conteo de emails (mail.mail)
```bash
# Contar mail.mail existentes para detectar emails generados durante test
search_records.sh mail.mail '[]' '["id"]' 0
# Guardar COUNT para comparar post-revert
```

### Conteo de Seguidores
```bash
search_records.sh mail.followers '[["res_model","in",["sale.order","purchase.order","account.move","stock.picking","crm.lead","project.task","helpdesk.ticket"]]]' '["res_model"]' 0
```

### Actividades Activas
```bash
search_records.sh mail.activity '[["res_model","in",["sale.order","purchase.order","account.move","stock.picking","crm.lead","project.task","helpdesk.ticket"]]]' '["id","res_model","activity_type_id","date_deadline"]' 0
```

### Estados de Integraciones
```bash
# Payment providers
search_records.sh payment.provider '[["state","!=","disabled"]]' '["id","name","state"]' 0

# Delivery carriers en produccion
search_records.sh delivery.carrier '[["active","=",true],["prod_environment","=",true]]' '["id","name","prod_environment"]' 0

# Automatizaciones activas
search_records.sh base.automation '[["active","=",true]]' '["id","name","trigger","active"]' 0

# Servidores de correo activos
search_records.sh ir.mail_server '[["active","=",true]]' '["id","name","active"]' 0

# Fetchmail servers activos
search_records.sh fetchmail.server '[["active","=",true]]' '["id","name","active"]' 0
```

### Formato del Snapshot

```json
{
  "timestamp": "2026-03-09T10:30:00Z",
  "prefix": "[TEST-20260309-1030]",
  "pre_state": {
    "account_balances": [
      {"id": 1, "code": "100000", "name": "Caja", "balance": 15000.00}
    ],
    "account_move_line_totals": {
      "total_debit": 500000.00,
      "total_credit": 500000.00
    },
    "stock_quantities": [
      {"id": 5, "name": "Mesa Roble", "qty_available": 25.0, "virtual_available": 25.0}
    ],
    "stock_quant_totals": [
      {"product_id": 5, "location_id": 8, "quantity": 25.0, "reserved_quantity": 0.0}
    ],
    "sequences": [
      {"id": 10, "code": "sale.order", "number_next": 42},
      {"id": 11, "code": "account.move", "number_next": 156}
    ],
    "sequence_date_ranges": [
      {"id": 20, "sequence_id": 11, "date_from": "2026-01-01", "date_to": "2026-12-31", "number_next": 89}
    ],
    "mail_mail_count": 1250,
    "followers_count": {
      "sale.order": 85,
      "purchase.order": 32,
      "account.move": 210,
      "stock.picking": 45,
      "crm.lead": 28,
      "project.task": 67,
      "helpdesk.ticket": 15
    },
    "activities_count": {
      "sale.order": 5,
      "crm.lead": 12,
      "project.task": 8,
      "helpdesk.ticket": 3
    },
    "integrations": {
      "payment_providers": [{"id": 1, "name": "Stripe", "state": "enabled"}],
      "delivery_carriers": [{"id": 3, "name": "FedEx", "prod_environment": true}],
      "automations_active": [{"id": 5, "name": "Enviar email al confirmar"}],
      "mail_servers_active": [{"id": 1, "name": "Servidor principal"}],
      "fetchmail_servers_active": [{"id": 2, "name": "Bandeja entrada"}]
    }
  }
}
```

---

