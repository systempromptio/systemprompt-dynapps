## 2. fleet.vehicle.log.services

### Estados
```python
state = fields.Selection([
    ('new', 'New'),
    ('running', 'Running'),
    ('done', 'Done'),
    ('cancelled', 'Cancelled'),
], default='new')
```

### Campos Minimos para create()
| Campo | Tipo | Requerido | Default | Notas |
|-------|------|-----------|---------|-------|
| vehicle_id | Many2one (fleet.vehicle) | Si | - | Vehiculo asociado |
| service_type_id | Many2one (fleet.service.type) | Si | ref('fleet.type_service_service_7') | Tipo de servicio |

### Flujos Atomicos

#### FLEET_SERVICE_CREATE
- **Modelo:** fleet.vehicle.log.services
- **Accion:** create
- **Campos minimos:** `{'vehicle_id': <id>, 'service_type_id': <id>}`
- **Resultado:** registro con state='new'
- **Verificacion:** id > 0, state == 'new'

#### FLEET_SERVICE_CHANGE_STATE
- **Modelo:** fleet.vehicle.log.services
- **Accion:** write
- **Campos:** `{'state': 'running'|'done'|'cancelled'}`
- **Nota:** No hay metodos action_* para transicion; se cambia con write directo.

#### FLEET_SERVICE_DELETE
- **Modelo:** fleet.vehicle.log.services
- **Accion:** unlink
- **Restricciones:** Sin restricciones especiales.

### Registros Cascade
| Modelo hijo | Campo FK | ondelete |
|------------|----------|----------|
| (ninguno) | - | - |

**Nota:** `fleet.vehicle.log.fuel` NO EXISTE en Odoo 18.0. El modelo fue eliminado; los combustibles se gestionan como servicios.

---

