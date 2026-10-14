## 29. CONTABILIDAD EXTENDIDA

### RECURRING_ENTRY_CREATE — Crear Asiento Recurrente
**Modelo:** account.move
**Descripción:** Crea un asiento contable configurado como recurrente para generar automáticamente copias periódicas.
**Campos mínimos:**
```json
{
  "move_type": "entry",
  "journal_id": "<int>",
  "line_ids": [[0, 0, {
    "account_id": "<int>",
    "name": "Asiento recurrente",
    "debit": "<float>"
  }], [0, 0, {
    "account_id": "<int>",
    "name": "Contrapartida",
    "credit": "<float>"
  }]]
}
```
**Notas:**
- Se configura como recurrente vía los campos de recurrencia del asiento
- `auto_post`: "at_date" o "monthly"/"quarterly"/"yearly" para recurrencia
- `auto_post_until`: fecha hasta la cual generar
- El asiento se publica automáticamente cuando llega la fecha
- Se puede crear manualmente y configurar recurrencia
**Método/acción:** `create_record.sh account.move '...'`
**Precondiciones:** Diario tipo "Operaciones varias" o similar, cuentas contables válidas
**Resultado:** state = `draft`, configuración de recurrencia activa
**Verificación:** `state == 'draft'`, `auto_post != 'no'`
**Estrategia de reversión:** `delete_record.sh account.move '[<id>]'` — solo en draft

### FISCAL_YEAR_CLOSE — Asiento de Cierre de Ejercicio
**Modelo:** account.move
**Descripción:** Genera el asiento de cierre del ejercicio fiscal. En Odoo 18, no hay wizard específico; se crea un asiento manual de cierre.
**Pasos vía API:**
1. Crear asiento de cierre tipo "entry" con las contrapartidas de cierre (P&L → resultados del ejercicio)
2. Publicar: `execute_method.sh account.move action_post '[[<id>]]' '{}'`
3. Configurar bloqueo fiscal: `update_record.sh res.company '[<company_id>]' '{"fiscalyear_lock_date": "YYYY-MM-DD"}'`
**Notas:**
- El bloqueo fiscal impide crear/modificar asientos anteriores a la fecha
- Requiere permisos de administrador contable
- Las cuentas de P&L (income/expense) se saldan contra la cuenta de resultados del ejercicio
**Precondiciones:** Todas las operaciones del período contabilizadas, conciliaciones completadas
**Resultado:** Asiento de cierre publicado, fecha de bloqueo establecida
**⚠️ PUNTO DE NO RETORNO:** La fecha de bloqueo impide modificaciones posteriores
**Registros cascada generados:** account.move (1), account.move.line (N)

---

