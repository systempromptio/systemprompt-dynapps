# Playbook: Validación de PDFs con cliente

Tras la migración, los reports PDF clave (factura, presupuesto, albarán,
extracto, contabilidad, COA, etiquetas) deben validarse contra los originales v15.

## Requisitos del contexto
- `to_version`: cualquiera

## Aplicabilidad
- Cliente con reports custom o layout custom
- Especialmente si hubo `modules/custom_report_to_docx.md`

## Flujo
1. Generar muestras en BD v18 migrada para una selección de records reales (no demo)
2. Generar las mismas muestras desde la BD v15 original (BD `analyzer` con dump original)
3. Comparar visualmente — checklist:
   - Logo, footer
   - Datos fiscales (NIF, dirección)
   - Importes y desglose de impuestos
   - Líneas de detalle (descripción, cantidad, precio)
   - Totales (subtotal, impuestos, total)
   - Pies de página (paginación, condiciones, IBAN si aplica)
4. Cliente firma o reporta diferencias → bug ticket
5. Ack en `indaws-migrate ack pdf_validation --user <cliente> --note "OK con observaciones X,Y"`

## Helpers a invocar
- Sin step técnico — proceso manual con validación humana

## Pitfalls
- Diferencias de orden de impuestos: en v18 los impuestos se ordenan por
  `sequence` del tax, no por orden de aplicación
- `tax_totals` (dict v18) vs `tax_totals_json` (str v15) tienen estructura diferente
- BS5 puede romper `text-right`, `pull-right` — verificar en navegadores reales
