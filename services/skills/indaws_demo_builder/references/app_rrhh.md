# RRHH (Human Resources) - Guia de Configuracion y Datos para Demo

## Modulos

| Modulo | Descripcion | Cuando instalar |
|--------|-------------|-----------------|
| hr | Empleados base | Siempre (si app seleccionada) |
| hr_holidays | Ausencias/Vacaciones | Si gestiona ausencias |
| hr_expense | Gastos de empleado | Si tiene notas de gasto |
| hr_recruitment | Seleccion de personal | Si tiene procesos de seleccion |
| hr_attendance | Control de presencia | Si usa fichaje |
| hr_contract | Contratos | Si gestiona contratos laborales |
| hr_payroll | Nominas | Si procesa nominas (requiere localizacion) |

## Configuracion

### Ajustes Principales
- **Departamentos**: Crear 2-3 departamentos (Administracion, Comercial, Produccion/Tecnico)
- **Puestos de trabajo**: 3-5 puestos relevantes al sector
- **Tipos de ausencia**: Vacaciones, Baja medica, Asuntos propios, Permiso retribuido
- **Asignacion de vacaciones**: 22-25 dias laborables/ano por empleado

### Features Opcionales
- Portal del empleado: Para autoservicio de ausencias/gastos
- Aprobacion multi-nivel: Si hay jerarquia
- Fichaje: Via app movil o terminal
- Evaluaciones: Si tiene proceso de evaluacion de desempeno

## Datos Maestros

### Empleados (5-10)
- Distribuidos en 2-3 departamentos
- Al menos 1 manager por departamento
- Datos completos: nombre, email, telefono, departamento, puesto, manager
- Foto/avatar en al menos los managers
- Contratos asociados (si modulo activo)

### Departamentos (2-3)
- Administracion / Finanzas
- Comercial / Ventas
- Operaciones / Produccion / Tecnico

### Puestos de Trabajo (3-5)
- Director (1-2)
- Responsable de area (2-3)
- Tecnico/Operario (3-5)

## Flujo de Negocio - Distribucion por Estado

### Ausencias (hr.leave)
| Estado | % | Descripcion |
|--------|---|-------------|
| Borrador | 20% | Pendientes de solicitar |
| Confirmada | 30% | Solicitadas, esperando aprobacion |
| Validada | 40% | Aprobadas |
| Rechazada | 10% | Para ver flujo completo |

### Gastos (hr.expense)
| Estado | % | Descripcion |
|--------|---|-------------|
| Borrador | 25% | Pendientes de enviar |
| Enviado | 30% | Esperando aprobacion |
| Aprobado | 25% | Pendiente de pago |
| Pagado | 20% | Reembolsados |

### Asistencia (hr.attendance) - si activo
- Registros de los ultimos 5 dias laborables
- 1 empleado con hora extra (para demo)
- 1 empleado con ausencia sin justificar (para alertas)

## Tips para Demo Impactante
- Organigrama visual con departamentos y managers
- 1 solicitud de ausencia pendiente de aprobar para hacerlo en demo
- Gastos con diferentes categorias (viaje, comida, material)
- Si hay asistencia, mostrar el dashboard de hoy con entradas/salidas
- Tener 1 empleado con vacaciones proximas (aparece en calendario)
- Mostrar vista calendario de ausencias del equipo

## Comandos API (odoo-pilot)

```bash
# Crear departamento
create_record.sh hr.department '{"name": "Comercial"}'

# Crear empleado
create_record.sh hr.employee '{"name": "Maria Garcia", "department_id": DEPT_ID, "job_id": JOB_ID, "work_email": "m.garcia@empresa.com", "parent_id": MANAGER_EMPLOYEE_ID}'

# Crear solicitud de ausencia
create_record.sh hr.leave '{"employee_id": EMP_ID, "holiday_status_id": LEAVE_TYPE_ID, "date_from": "2026-03-01 08:00:00", "date_to": "2026-03-05 17:00:00"}'

# Crear gasto
create_record.sh hr.expense '{"name": "Viaje cliente Madrid", "employee_id": EMP_ID, "product_id": EXPENSE_PROD_ID, "total_amount": 150.0}'
```
