# Restriccion CRITICA sobre odoo-pilot

Esta skill usa `odoo-pilot` **EXCLUSIVAMENTE EN MODO LECTURA**. Operaciones permitidas:

| Operacion | Permitida | Motivo |
|-----------|-----------|--------|
| `auth.sh` | ✅ | Solo autentica |
| `search_records.sh` | ✅ | Read-only |
| `execute_method.sh` con `read`, `search_read`, `search_count`, `fields_get`, `name_search`, `name_get` | ✅ | Read-only |
| `execute_method.sh` con `create`, `write`, `unlink`, `copy` | ❌ PROHIBIDO | Mutacion |
| `create_record.sh`, `update_record.sh`, `delete_record.sh` | ❌ PROHIBIDO | Mutacion |
| Cualquier metodo custom que mute estado (button_*, action_*, etc.) | ❌ PROHIBIDO | Side-effects |

Razon: el inventariado es auditoria. NUNCA debe alterar la BD del cliente.
