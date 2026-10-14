# Referencia de Privacidad de Datos - Employee Lookup

**CONFIDENTIAL - HR TEAM ONLY**
**Document ID:** HR-SKILL-006-REF-001
**Normativa aplicable:** RGPD (UE 2016/679), LOPDGDD (LO 3/2018)

---

## Clasificacion de Campos

### Campos PERMITIDOS (se pueden mostrar al equipo de RRHH)

| Campo | Fuente HiBob | Notas |
|-------|--------------|-------|
| Nombre completo | `root.fullName` | Siempre visible |
| Puesto | `work.title` | Cargo actual |
| Departamento | `work.department` | Departamento actual |
| Manager directo | `work.reportsTo` | Nombre y puesto del manager |
| Ubicacion / Sede | `work.site`, `work.siteId` | Ciudad u oficina |
| Fecha de incorporacion | `work.startDate` | Formato DD/MM/YYYY |
| Email corporativo | `email` | Solo el email de empresa |
| Telefono de oficina | `work.phone` | Solo extension o linea de oficina |
| Tipo de contrato | `work.employeeType` | Indefinido, temporal, etc. |
| Estado | `root.status` | Activo / Inactivo |
| Antiguedad | Calculado | Desde `startDate` hasta hoy |
| ID de empleado | `root.id` | Identificador interno HiBob |

### Campos PROHIBIDOS (NUNCA mostrar)

| Campo | Motivo de proteccion | Categoria RGPD |
|-------|----------------------|----------------|
| NIF / NIE | Dato identificativo | Art. 87 LOPDGDD |
| Numero de pasaporte | Dato identificativo | Art. 87 LOPDGDD |
| Numero de Seguridad Social | Dato identificativo | Art. 87 LOPDGDD |
| Salario bruto / neto | Dato financiero | Legitimo interes limitado |
| Cuenta bancaria / IBAN | Dato financiero | Legitimo interes limitado |
| Email personal | Dato personal | Minimizacion de datos |
| Telefono personal | Dato personal | Minimizacion de datos |
| Direccion particular | Dato personal | Minimizacion de datos |
| Fecha de nacimiento | Dato personal | Minimizacion de datos |
| Estado civil | Dato personal sensible | Art. 9 RGPD |
| Numero de hijos | Dato personal | Minimizacion de datos |
| Religion | Dato sensible | Art. 9 RGPD |
| Orientacion sexual | Dato sensible | Art. 9 RGPD |
| Afiliacion sindical | Dato sensible | Art. 9 RGPD |
| Datos de salud | Dato sensible | Art. 9 RGPD |
| Reconocimientos medicos | Dato sensible | Art. 9 RGPD |
| Discapacidad | Dato sensible | Art. 9 RGPD |
| Genero | Dato personal | Minimizacion de datos |
| Nacionalidad | Dato personal | Minimizacion de datos |
| Permiso de residencia | Dato identificativo | Art. 87 LOPDGDD |
| Foto de perfil | Dato biometrico | Art. 9 RGPD |

---

## Reglas de Acceso por Rol

| Accion | RRHH | Manager directo | Empleado | Otros |
|--------|------|-----------------|----------|-------|
| Ficha basica (nombre, puesto, dept) | Si | Si | Si (propia) | No |
| Ficha completa (+ ubicacion, fecha) | Si | Si (reports) | Si (propia) | No |
| Linea de reporte | Si | Si | Si (propia) | No |
| Historial laboral | Si | Si (reports) | Si (propio) | No |
| Busqueda por departamento | Si | Si (su dept) | No | No |
| Listado completo de empleados | Si | No | No | No |

---

## Protocolo ante Solicitud de Datos Protegidos

Si el usuario solicita un campo prohibido, responder **exactamente**:

> "Estos datos estan protegidos por el RGPD (Reglamento General de Proteccion de Datos) y la LOPDGDD. No es posible mostrarlos a traves de esta herramienta. Para acceder a esta informacion, contacte con el departamento de RRHH directamente (rrhh@indaws.es)."

---

## Buenas Practicas

1. **Minimizacion**: Mostrar solo los campos necesarios para la consulta solicitada
2. **Temporalidad**: No almacenar resultados de busqueda en archivos persistentes
3. **Trazabilidad**: Los JSONs intermedios de la API son temporales y deben eliminarse
4. **Sanitizacion**: Usar `report_utils.sanitize_employee_data()` para cualquier procesamiento en batch
5. **Proporcionalidad**: En busquedas por departamento, mostrar solo nombre + puesto + email corporativo
