# La evidencia corrige el informe

Bajar de «162 líneas mal» a «enséñame las 162» destapa falsos positivos: en las primeras pasadas
reales tumbó nueve hallazgos y reescribió otro. No es control de calidad, es parte de construir
el hallazgo.

## El anexo de evidencias (`excel_evidencias.py`)

Por cada hallazgo, **todas** las filas, con columnas `Punto · Prioridad · Tipo de registro ·
Registro · Fecha · Importe · Qué falta · Dónde se corrige en Odoo · Hecho · Notas`.

- «Dónde se corrige» es una **ruta de menú de Odoo**, no un nombre de modelo.
- «Qué falta» dice qué hacer **en ese registro**, no qué significa el hallazgo.
- «Hecho» con lista Sí/No y fondo amarillo; el Resumen cuenta con `COUNTIF`/`COUNTIFS`.
- Tres hojas: **Cómo usarlo**, **Resumen** (un punto por fila, responsable, plazo, avance) y
  **Evidencias** (filtro `A4:J<n>`, paneles fijos en `C5`).
- Más de ~50 filas en un punto: las 50 de más importe y una fila «y otras N en el mismo caso».
- Importes en formato español.

## Falsos positivos que hay que purgar antes de escribir nada

- **Gastos sin analítica**: no es regla plana (`alcance-y-usos.md`). La plana habría entregado
  «442 líneas de gasto sin analítica» en una base con cero reglas de aplicabilidad.
- **Asientos manuales**: los escritos a mano en un diario de tipo general. Los del banco o la
  conciliación no lo son (684 contados frente a 457 reales).
- **Antigüedad de deuda sin conciliar**: si la cartera no está cruzada, el pendiente de cada
  apunte es su importe íntegro. Es una cartera ilegible, no vencida: NO MIRADO con motivo.
  Llegó a salir «8.013.749 € vencidos sin deteriorar».
- **Seguridad Social, nóminas (465) e IRPF (4751)**: el saldo nunca es cero (se devenga un mes y
  se paga al siguiente). Comparar el saldo vivo con el devengo medio de un mes (un trimestre para
  el 111) y **decir de qué año viene el saldo**: un saldo antiguo no es un impago en curso.
- **Cuentas de proveedor**: 41x es tan válida como 40x.
- **Tipos de cambio**: Odoo aplica el último **anterior** a la fecha; no exigir el de la fecha.
- **Reparto de impuesto sin cuenta o etiqueta**: solo impuestos en uso y con tipo distinto de cero.
- **Cuentas duplicadas por nombre**: exigir mismo prefijo de tres dígitos (282000/682000 y
  291/292 comparten nombre en es_pymes y son correctas).
- **`account_type` contra código PGC**: equiparar familias (income/income_other,
  payable/current, expense/expense_other). Perseguir matices de la plantilla es criticar a Odoo.
- **Diario de banco con cuenta que no es 57x**: validar por `account_type` (una póliza de crédito
  usa `liability_credit_card`).
- **Cuentas de tesorería sin diario**: excluir `suspense_account_id`, `profit_account_id` y
  `loss_account_id`.
- **Antigüedad de saldo**: solo residuos positivos; uno negativo es un abono sin aplicar.
- **Base sembrada de golpe**: asientos con fecha muy anterior a su creación son un artefacto. La
  referencia del backdating es la fecha del informe anterior; sin informe anterior, NO MIRADO.
- **Asientos sin desglose contra la 129**: la aplicación del resultado es de dos líneas contra la
  129 por naturaleza. Llegaron a entregarse dos al cliente como hallazgo.
- **«La cuenta recoge la regularización»** decidido por movimientos en la 129: falso verde, las
  aplicaciones del resultado también los tienen.
- **Saldo deudor de la 129 sin regularizar**: recomendación, no hallazgo (`cierre-y-129.md`).
- **«Todavía no hay asiento de cierre»** no es motivo de NO APLICA para el impuesto sobre
  beneficios.
