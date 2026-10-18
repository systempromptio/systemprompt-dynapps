# Plan General Contable Español — Referencia

## Grupos principales

| Grupo | Descripción | Tipo Balance |
|-------|-------------|-------------|
| 1 | Financiación básica | Pasivo no corriente |
| 2 | Inmovilizado | Activo no corriente |
| 3 | Existencias | Activo corriente |
| 4 | Acreedores y deudores | Activo/Pasivo corriente |
| 5 | Cuentas financieras | Activo/Pasivo corriente |
| 6 | Compras y gastos | Gastos |
| 7 | Ventas e ingresos | Ingresos |
| 8 | Gastos imputados al patrimonio neto | Patrimonio neto |
| 9 | Ingresos imputados al patrimonio neto | Patrimonio neto |

## Subgrupos principales (2 dígitos)

### Grupo 1 — Financiación básica
- 10: Capital
- 11: Reservas y otros instrumentos de patrimonio
- 12: Resultados pendientes de aplicación (129: resultado del ejercicio)
- 13: Subvenciones, donaciones y legados
- 14: Provisiones
- 15: Deudas a largo plazo con características especiales
- 16: Deudas a largo plazo con partes vinculadas
- 17: Deudas a largo plazo por préstamos y otros conceptos
- 18: Pasivos por fianzas, garantías y otros conceptos a largo plazo
- 19: Situaciones transitorias de financiación

### Grupo 2 — Inmovilizado
- 20: Inmovilizaciones intangibles
- 21: Inmovilizaciones materiales
- 22: Inversiones inmobiliarias
- 23: Inmovilizaciones materiales en curso
- 24: Inversiones financieras a largo plazo en partes vinculadas
- 25: Otras inversiones financieras a largo plazo
- 26: Fianzas y depósitos constituidos a largo plazo
- 28: Amortización acumulada del inmovilizado
- 29: Deterioro de valor de activos no corrientes

### Grupo 3 — Existencias
- 30: Comerciales
- 31: Materias primas
- 32: Otros aprovisionamientos
- 33: Productos en curso
- 34: Productos semiterminados
- 35: Productos terminados
- 36: Subproductos, residuos y materiales recuperados
- 39: Deterioro de valor de existencias

### Grupo 4 — Acreedores y deudores
- 40: Proveedores
- 41: Acreedores varios
- 43: Clientes
- 44: Deudores varios
- 46: Personal
- 47: Administraciones públicas
- 48: Ajustes por periodificación
- 49: Deterioro de valor de créditos comerciales

### Grupo 5 — Cuentas financieras
- 50: Empréstitos, deudas con características especiales y otras emisiones análogas a corto plazo
- 51: Deudas a corto plazo con partes vinculadas
- 52: Deudas a corto plazo por préstamos recibidos y otros conceptos
- 53: Inversiones financieras a corto plazo en partes vinculadas
- 54: Otras inversiones financieras a corto plazo
- 55: Otras cuentas no bancarias
- 56: Fianzas y depósitos recibidos y constituidos a corto plazo
- 57: Tesorería (570: Caja, 572: Bancos)

### Grupo 6 — Compras y gastos
- 60: Compras
- 61: Variación de existencias
- 62: Servicios exteriores
- 63: Tributos
- 64: Gastos de personal (640: Sueldos, 642: SS empresa)
- 65: Otros gastos de gestión
- 66: Gastos financieros
- 67: Pérdidas procedentes de activos no corrientes y gastos excepcionales
- 68: Dotaciones para amortizaciones (681/682)
- 69: Pérdidas por deterioro y otras dotaciones

### Grupo 7 — Ventas e ingresos
- 70: Ventas
- 71: Variación de existencias
- 73: Trabajos realizados para la empresa
- 74: Subvenciones, donaciones y legados
- 75: Otros ingresos de gestión
- 76: Ingresos financieros
- 77: Beneficios procedentes de activos no corrientes e ingresos excepcionales
- 79: Excesos y aplicaciones de provisiones y deterioros

## Cuentas clave para auditoría

| Cuenta | Descripción | Checks especiales |
|--------|-------------|-------------------|
| 129 | Resultado del ejercicio | Debe quedar a 0 tras reparto |
| 400xxx | Proveedores | Payable, saldo acreedor normal |
| 430xxx | Clientes | Receivable, saldo deudor normal |
| 465xxx | Remuneraciones pendientes de pago | Provisión pagas extras |
| 472xxx | HP IVA soportado | Debe tener tax_ids |
| 475xxx | HP acreedora (IRPF) | 4751: retenciones practicadas |
| 476xxx | Organismos SS acreedores | Pago mensual SS |
| 477xxx | HP IVA repercutido | Debe tener tax_ids |
| 570xxx | Caja | Tesorería |
| 572xxx | Bancos | Conciliación bancaria |
| 640xxx | Sueldos y salarios | Nóminas mensuales |
| 642xxx | SS a cargo empresa | Nóminas mensuales |
| 681/682 | Amortización inmovilizado | Periódico |
