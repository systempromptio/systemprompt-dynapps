# Reglas de Auditoría — accounting-auditor

## Severidades

| Nivel | Icono | Significado | Impacto en Score |
|-------|-------|-------------|-----------------|
| Crítico | 🔴 | Error que afecta a la fiabilidad de las cuentas | -5 puntos por hallazgo |
| Aviso | 🟡 | Incidencia que debe revisarse | -2 puntos por hallazgo |
| Info | ℹ️ | Información relevante sin impacto directo | 0 puntos |

## Score de Salud Contable

- Base: 100 puntos
- Se restan puntos por hallazgo según severidad
- Mínimo: 0, Máximo: 100
- Clasificación:
  - 90-100: Excelente
  - 75-89: Bueno
  - 50-74: Necesita atención
  - 25-49: Deficiente
  - 0-24: Crítico

## Categorías y Checks

### 1. Plan General Contable (7 checks)

1. **Longitud inconsistente**: Detectar longitud predominante y listar cuentas diferentes
2. **Numeración fuera del PGC**: Subgrupos no existentes en PGC (ver pgc_chart.md)
3. **Cuentas nuevas sin movimientos**: Creadas manualmente (no l10n_es) sin apuntes
4. **Cuentas duplicadas**: Misma descripción en mismo subgrupo
5. **Tipo de cuenta incorrecto**: Receivable fuera de 43x, Payable fuera de 40x
6. **Cuentas IVA mal configuradas**: 472/477 sin tax_ids
7. **Conciliación en gastos/ingresos**: reconcile=True en 6xx/7xx

### 2. Períodos y cierres (4 checks)

1. **Períodos no bloqueados**: Meses pasados sin lock
2. **Fechas de bloqueo inconsistentes**: tax_lock > period_lock
3. **Borradores en periodos bloqueados**: Draft con fecha < lock date
4. **Reparto de resultados**: 129 no a 0 tras cierre ejercicio anterior

### 3. Asientos contables (7 checks)

1. **Borradores antiguos**: Draft > 30 días
2. **Asientos descuadrados**: SUM(debit) ≠ SUM(credit)
3. **Asientos sin líneas**: move sin line_ids
4. **Fecha futura**: date > hoy
5. **Posibles duplicados**: mismo partner+importe+fecha+diario
6. **Líneas 43x/40x sin partner**: Cuentas clientes/proveedores sin partner_id
7. **Secuencia rota**: Huecos en numeración por diario

### 4. Conciliación bancaria (4 checks)

1. **Pendientes de conciliar**: 572xxx con amount_residual ≠ 0
2. **Pendientes antiguas >90 días**: Sin conciliar > 90 días
3. **Extractos sin validar**: state ≠ confirm
4. **Saldo contable vs extracto**: Diferencia entre 572 y extracto

### 5. IVA y retenciones (5 checks)

1. **Facturas sin impuesto**: Venta/compra sin IVA
2. **IVA soportado sin factura**: 472xxx sin factura
3. **Cuadre retenciones IRPF (4751)**: Saldo vs pagos a Hacienda
4. **Rectificativas mal contabilizadas**: Signos incorrectos
5. **Cuadre IVA trimestral**: 477 - 472 por trimestre

### 6. Clientes y proveedores (6 checks)

1. **Saldos negativos clientes**: 430xxx saldo acreedor
2. **Saldos negativos proveedores**: 400xxx saldo deudor
3. **Partidas abiertas >365 días**: Facturas impagadas > 1 año
4. **Partners sin NIF/CIF**: Con facturas pero sin vat
5. **Partners sin país**: Con facturas pero sin country_id
6. **Facturas sin partner**: Venta/compra sin partner_id

### 7. Nóminas y Seguridad Social (6 checks)

1. **Asientos nómina mensuales**: Existencia de apuntes 640xxx cada mes
2. **Meses sin nómina**: Meses en período sin apuntes sueldos
3. **Cuadre SS empresa vs pago**: 476xxx saldo vs pagos
4. **Provisión pagas extras**: Apuntes regulares en 465xxx
5. **IRPF nóminas vs 4751**: Retenciones nóminas = saldo 4751
6. **Liquidación SS mensual**: Pago mensual contra 476xxx

### 8. Inmovilizado y amortización (4 checks)

1. **Activos sin amortización**: 2xxx con saldo sin 681/682
2. **Amortización acumulada coherente**: 28xxx ≤ 2xxx
3. **Altas sin contrapartida**: 2xxx sin contrapartida en 57x/17x/52x
4. **Bajas incompletas**: Crédito en 2xxx sin baja de 28xxx y resultado 771/671

### 9. Cuentas prohibidas (2 checks)

1. **Movimientos en 700000**: La cuenta 700000 no debe tener apuntes. Los ingresos por ventas deben imputarse a cuentas específicas (7050xx). Severidad: 🔴 Crítico
2. **Movimientos en 705000**: La cuenta 705000 no debe tener apuntes. Los ingresos por prestación de servicios deben imputarse a subcuentas específicas (ej. 705013 Otros trabajos ERP). Severidad: 🔴 Crítico

### 10. Coherencia general (6 checks)

1. **Diarios mal configurados**: Sin cuenta por defecto
2. **Moneda incorrecta**: amount_currency en divisa extraña
3. **Resultado del ejercicio**: 129 vs (grupo 7 - grupo 6)
4. **Variaciones interanuales >50%**: Cuentas con variación anómala
5. **Gastos/ingresos excepcionales**: Apuntes en 678/778
6. **Localización española**: l10n_es instalado, módulos AEAT
