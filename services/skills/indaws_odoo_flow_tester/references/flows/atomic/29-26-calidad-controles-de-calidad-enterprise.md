## 26. CALIDAD — Controles de Calidad [Enterprise]

### QC_CHECK_CREATE — Crear Control de Calidad
**Modelo:** quality.check
**Descripción:** Crea un control de calidad manual. Normalmente se crean automáticamente desde puntos de control, pero se pueden crear vía API.
**Campos mínimos:**
```json
{
  "team_id": "<int>",
  "product_id": "<int>",
  "test_type_id": "<int>",
  "title": "Control de calidad test"
}
```
**Notas:**
- `team_id`: equipo de calidad (quality.alert.team, required)
- `product_id`: producto a controlar (opcional, storable)
- `test_type_id`: tipo de test (quality.point.test_type, required) — ej: "pass_fail", "measure", "picture"
- `title`: título del control
- `point_id`: punto de control (quality.point, opcional — auto-genera checks)
- `picking_id`: albarán asociado (opcional)
- `lot_id`: lote/serie (opcional)
- `quality_state`: 'none' (To do), 'pass' (Passed), 'fail' (Failed)
- `company_id`: compañía (required, default)
**Método/acción:** `create_record.sh quality.check '...'`
**Precondiciones:** Módulo `quality_control` instalado, equipo de calidad configurado
**Resultado:** quality_state = `none` (To do)
**Verificación:** `quality_state == 'none'`
**Estrategia de reversión:** `delete_record.sh quality.check '[<id>]'`
**Registros cascada generados:** Ninguno

### QC_CHECK_PASS — Aprobar Control de Calidad
**Modelo:** quality.check
**Descripción:** Marca el control como aprobado (passed).
**Método:** `execute_method.sh quality.check do_pass '[[<id>]]' '{}'`
**Precondiciones:** quality_state == 'none'
**Resultado:** quality_state → `pass`, control_date establecido, user_id establecido
**Verificación:** `quality_state == 'pass'`, `control_date != False`
**Estrategia de reversión:** No hay reversión estándar. Se podría forzar: `update_record.sh quality.check '[<id>]' '{"quality_state": "none"}'`

### QC_CHECK_FAIL — Rechazar Control de Calidad
**Modelo:** quality.check
**Descripción:** Marca el control como fallido (failed).
**Método:** `execute_method.sh quality.check do_fail '[[<id>]]' '{}'`
**Precondiciones:** quality_state == 'none'
**Resultado:** quality_state → `fail`, control_date establecido, user_id establecido
**Efectos secundarios:**
- Puede crear quality.alert automáticamente según configuración del punto de control
- Si el control está vinculado a un picking, puede bloquear la validación
**Verificación:** `quality_state == 'fail'`, `control_date != False`
**Estrategia de reversión:** No hay reversión estándar.

---

