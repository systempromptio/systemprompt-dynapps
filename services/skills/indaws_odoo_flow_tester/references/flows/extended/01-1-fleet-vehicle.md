## 1. fleet.vehicle

### Estados
No tiene campo `state` tipo Selection. Usa `state_id` (Many2one a `fleet.vehicle.state`), un modelo configurable por el usuario. Default: referencia XML `fleet.fleet_vehicle_state_new_request`.

### Campos Minimos para create()
| Campo | Tipo | Requerido | Default | Notas |
|-------|------|-----------|---------|-------|
| model_id | Many2one (fleet.vehicle.model) | Si | - | Modelo del vehiculo, obligatorio |
| odometer_unit | Selection | Si | 'kilometers' | km o mi |
| power_unit | Selection | Si | 'power' | kW o Horsepower |

### Flujos Atomicos

#### FLEET_VEHICLE_CREATE
- **Modelo:** fleet.vehicle
- **Accion:** create
- **Campos minimos:** `{'model_id': <id>}`
- **Resultado:** registro creado, name computado automaticamente como "Brand/Model/Plate"
- **Verificacion:** id > 0, name contiene '/'

#### FLEET_VEHICLE_UPDATE_STATE
- **Modelo:** fleet.vehicle
- **Accion:** write
- **Campos:** `{'state_id': <id_estado>}`
- **Nota:** Los estados son registros de `fleet.vehicle.state`, no una selection fija. Se cambian con write directo.

#### FLEET_VEHICLE_ACCEPT_DRIVER
- **Modelo:** fleet.vehicle
- **Accion:** action_accept_driver_change()
- **Precondicion:** future_driver_id definido
- **Resultado:** driver_id = future_driver_id, future_driver_id = False

#### FLEET_VEHICLE_DELETE
- **Modelo:** fleet.vehicle
- **Accion:** unlink
- **Restricciones:** Sin restricciones especiales en unlink. No hay `@api.ondelete` ni override de `unlink()`.

### Registros Cascade
| Modelo hijo | Campo FK | ondelete |
|------------|----------|----------|
| (ninguno con cascade) | - | - |

**Nota:** Los One2many (log_services, log_contracts, log_drivers) no tienen ondelete='cascade' explicito en el FK. El vehiculo se puede archivar (`active=False`) lo que desactiva contratos y servicios asociados via write().

**⚠️ IMPORTANTE para reversion:** `fleet.vehicle.log.services` tiene FK `vehicle_id` SIN ondelete='cascade' (usa restrict o set null). Esto significa que intentar borrar un `fleet.vehicle` que tenga logs de servicios asociados fallara con error de FK. **Se deben borrar los registros de `fleet.vehicle.log.services` ANTES de borrar el vehiculo.**

---

