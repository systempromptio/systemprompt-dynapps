# Mapeo Departamento → Cuentas Contables

## Cuentas por departamento

| Departamento | Clave interna | Cuenta Sueldos (640xxx) | Cuenta SS Empresa (642xxx) |
|---|---|---|---|
| Consultoría | `consultoria` | 640001 | 642010 |
| Marketing interno | `marketing_interno` | 640003 | 642012 |
| Programación / Desarrollo | `programacion` | 640004 | 642013 |
| Administración | `administracion` | 640005 | 642014 |
| Comercial | `comercial` | 640007 | 642016 |
| Marketing externo | `marketing_externo` | 640008 | 642017 |
| Personas, Talento y Cultura | `personas_talento_cultura` | 640009 | 642019 |
| Nearshoring | `nearshoring` | 640010 | 642018 |
| Dirección | `direccion` | 640011 | 642020 |

## Cuentas fijas (no dependen del departamento)

| Cuenta | Nombre | Tipo | Descripción |
|---|---|---|---|
| 475100 | Retenciones IRPF | Credit | Suma total de la columna "Retención" de todos los empleados |
| 465000 | Remuneraciones pendientes de pago | Credit | Base IRPF - Retención - SS Trabajador |
| 476000 | Seguridad Social deudora | Credit | Cuadre: Total Debe - Total Haber parcial |

## Lógica de clasificación de departamentos

El script `process_payroll.py` clasifica los nombres de departamentos de Odoo usando palabras clave (sin acentos, en minúsculas):

| Palabra clave | Departamento asignado |
|---|---|
| `consult` | Consultoría |
| `marketing` + `intern/int` | Marketing interno |
| `marketing` + `extern/ext` | Marketing externo |
| `programac`, `desarrollo`, `dev` | Programación / Desarrollo |
| `administr` | Administración |
| `comercial`, `ventas`, `sales` | Comercial |
| `direcc`, `direction`, `gerencia`, `negocio` | Dirección |
| `nearshoring`, `near` | Nearshoring |
| `persona`, `talento`, `cultura` | Personas, Talento y Cultura |

## Estructura del asiento contable

```
DEBE (Debit):
  640xxx - Sueldos por departamento (suma de Base IRPF por dept)
  642xxx - SS Empresa por departamento (suma de SS Empresa por dept)

HABER (Credit):
  475100 - Retenciones IRPF (suma total Retención)
  465000 - Remuneraciones pendientes (Base IRPF - Retención - SS Trabajador)
  476000 - SS deudora (cuadre para que Debe = Haber)
```

## Cómo añadir un nuevo departamento

1. Editar `DEPARTMENT_ACCOUNT_MAP` en `process_payroll.py`
2. Añadir la nueva clave con sus cuentas `sueldos` y `ss_empresa`
3. Añadir regla de clasificación en `classify_department()`
4. Actualizar esta tabla de referencia
