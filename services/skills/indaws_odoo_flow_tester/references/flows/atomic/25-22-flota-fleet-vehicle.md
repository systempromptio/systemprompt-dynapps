## 22. FLOTA (fleet.vehicle)

### VEHICLE_CREATE — Crear Vehículo
**Modelo:** fleet.vehicle
**Descripción:** Registra un vehículo en la flota de la empresa.
**Campos mínimos:**
```json
{
  "model_id": "<int>",
  "license_plate": "1234 ABC"
}
```
**Notas:**
- `model_id`: modelo del vehículo (fleet.vehicle.model, required)
- `license_plate`: matrícula (no required en modelo pero sí recomendado)
- `driver_id` / `driver_employee_id`: conductor asignado (res.partner / hr.employee)
- `state_id`: estado del vehículo (fleet.vehicle.state — Nuevo, Activo, etc.)
- `odometer_unit`: "kilometers" (default), "miles"
- `tag_ids`: etiquetas
- `acquisition_date`: fecha de adquisición
- `company_id`: compañía (default: compañía del usuario)
**Método/acción:** `create_record.sh fleet.vehicle '...'`
**Precondiciones:** Al menos un modelo de vehículo (fleet.vehicle.model) y marca (fleet.vehicle.model.brand) configurados
**Resultado:** Vehículo creado
**Verificación:** `model_id == <id>`, `active == True`
**Estrategia de reversión:** `delete_record.sh fleet.vehicle '[<id>]'` o archivar: `update_record.sh fleet.vehicle '[<id>]' '{"active": false}'`
**Registros cascada generados:** Ninguno

### VEHICLE_LOG_FUEL — Registrar Repostaje
**Modelo:** fleet.vehicle.odometer
**Descripción:** Registra una lectura del odómetro (asociada a un repostaje u otro evento).
**Campos mínimos:**
```json
{
  "vehicle_id": "<int>",
  "value": "<float>",
  "date": "YYYY-MM-DD"
}
```
**Notas:**
- `vehicle_id`: vehículo (required)
- `value`: lectura del odómetro (km o millas)
- `date`: fecha del registro
- Para registrar el repostaje con coste, se crea un `fleet.vehicle.log.services` con service_type_id de tipo combustible
**Método/acción:** `create_record.sh fleet.vehicle.odometer '...'`
**Resultado:** Registro creado, actualiza odometer del vehículo
**Verificación:** `value == <valor>`, `vehicle_id == <id>`
**Estrategia de reversión:** `delete_record.sh fleet.vehicle.odometer '[<id>]'`

### VEHICLE_LOG_SERVICE — Registrar Servicio de Vehículo
**Modelo:** fleet.vehicle.log.services
**Descripción:** Registra un servicio de mantenimiento, reparación o repostaje de un vehículo.
**Campos mínimos:**
```json
{
  "vehicle_id": "<int>",
  "service_type_id": "<int>",
  "date": "YYYY-MM-DD",
  "amount": "<float>"
}
```
**Notas:**
- `vehicle_id`: vehículo (required)
- `service_type_id`: tipo de servicio (fleet.service.type, required) — ej: "Cambio de aceite", "Revisión ITV"
- `amount`: coste total del servicio
- `vendor_id`: proveedor del servicio (res.partner)
- `odometer`: lectura del odómetro al momento del servicio
- `notes`: notas adicionales
**Método/acción:** `create_record.sh fleet.vehicle.log.services '...'`
**Resultado:** Registro de servicio creado
**Verificación:** `vehicle_id == <id>`, `amount == <valor>`
**Estrategia de reversión:** `delete_record.sh fleet.vehicle.log.services '[<id>]'`
**Registros cascada generados:** fleet.vehicle.odometer (1, si se indica odometer)

---

