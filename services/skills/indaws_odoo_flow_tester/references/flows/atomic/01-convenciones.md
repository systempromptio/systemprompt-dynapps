## Convenciones

### Acciones API
| Acción | Script | Ejemplo |
|--------|--------|---------|
| create | create_record.sh | `create_record.sh sale.order '{"partner_id": 1}'` |
| read | search_records.sh | `search_records.sh sale.order '[["id","=",5]]' '["state"]'` |
| update | update_record.sh | `update_record.sh sale.order '[5]' '{"note": "test"}'` |
| delete | delete_record.sh | `delete_record.sh sale.order '[5]'` |
| execute | execute_method.sh | `execute_method.sh sale.order action_confirm '[[5]]' '{}'` |

### Estados de Verificación
- ✅ PASS: El campo tiene el valor esperado
- ❌ FAIL: El campo NO tiene el valor esperado → PARAR cadena
- ⚠️ WARN: Valor inesperado pero no bloqueante

### Context Flags para Testing

Flags de contexto que Odoo usa internamente en tests para evitar side effects. Usar en TODAS las operaciones de test:

| Flag | Efecto | Uso Recomendado |
|------|--------|-----------------|
| `tracking_disable` | Desactiva tracking de campos en chatter | Siempre en test |
| `mail_create_nolog` | No crea log de creacion en chatter | Siempre en test |
| `mail_notrack` | No genera mensajes de tracking | Siempre en test |
| `no_reset_password` | No envia email de reset password al crear usuarios | Crear res.users |
| `disable_cancel_warning` | Salta wizard de cancelacion en sale.order | Cancelar ventas |
| `skip_backorder` | No pregunta por backorder al validar picking parcial (stock.picking) | Validar pickings |
| `picking_ids_not_to_backorder` | Array de picking IDs para no crear backorder | Validar pickings parciales |
| `cancel_backorder` | Cancela backorders en vez de crearlos (usado por `_action_done`) | Validar pickings parciales |
| `skip_sanity_check` | Salta validaciones de sanidad en `button_validate` | Testing pickings |
| `mo_ids_to_backorder` | Lista de MO IDs que deben generar backorder en MRP (NO usar `skip_backorder` para MRP) | Producir MOs parciales |
| `force_delete` | Bypassa audit trail en unlink de account.move | Borrar asientos protegidos |
| `skip_invoice_sync` | Evita sincronizacion de invoice lines | Operaciones de pago |
| `skip_sale_auto_invoice_send` | Evita envio automatico de factura | Registro de pagos |
| `force_send` | Fuerza envio inmediato de email (NO usar en test) | NUNCA en test |
| `include_draft_documents` | Incluye documentos draft en logging | Debugging |

**Contexto base recomendado para TODAS las operaciones:**
```json
{"tracking_disable": true, "mail_create_nolog": true, "mail_notrack": true}
```

**Contexto extendido para operaciones contables:**
```json
{"tracking_disable": true, "mail_create_nolog": true, "mail_notrack": true, "skip_invoice_sync": true}
```

---

