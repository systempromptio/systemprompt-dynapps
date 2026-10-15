## 3. maintenance.equipment

### Estados
No tiene campo `state`. No tiene workflow de estados.

### Campos Minimos para create()
| Campo | Tipo | Requerido | Default | Notas |
|-------|------|-----------|---------|-------|
| name | Char | Si | - | Nombre del equipo |
| effective_date | Date | Si | Date.today | Heredado de maintenance.mixin |

### Flujos Atomicos

#### MAINT_EQUIPMENT_CREATE
- **Modelo:** maintenance.equipment
- **Accion:** create
- **Campos minimos:** `{'name': 'Test Equipment'}`
- **Resultado:** registro creado
- **Verificacion:** id > 0

#### MAINT_EQUIPMENT_DELETE
- **Modelo:** maintenance.equipment
- **Accion:** unlink
- **Restricciones:** Sin restricciones especiales de unlink. PERO la categoria (`maintenance.equipment.category`) tiene `@api.ondelete` que impide borrar si tiene equipos o requests asociados.

### Registros Cascade
| Modelo hijo | Campo FK | ondelete |
|------------|----------|----------|
| (ninguno con cascade explicito) | - | - |

**SQL Constraints:**
- `serial_no` UNIQUE

---

