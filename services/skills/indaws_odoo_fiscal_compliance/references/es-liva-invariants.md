# Invariantes IVA (España) — el ORÁCULO de "correcto" (general, no REAV-céntrico)

Este fichero es el **oráculo de "correcto"** del diferencial A/B: qué base/cuota/total exige la
ley para cada operación. Dos planos:
- **Régimen general (LA BASE, intocable)**: la contraprestación estándar (art. 78) al tipo que
  toque (art. 90-91). Cualquier módulo, sea del régimen que sea, **debe dejar estas operaciones
  exactamente igual**. Es el lado no-objetivo del diferencial.
- **Régimen especial (el OBJETIVO del módulo bajo revisión)**: la desviación que la ley permite/
  obliga SOLO para su subconjunto (margen REBU/REAV, ISP, recargo, IGIC/IPSI, criterio de caja…).
  REAV es el ejemplo de hoy; el catálogo de abajo vale para cualquier casuística futura.

Cada invariante lleva **cita**. Verificar SIEMPRE contra fuente antes de emitir un check
(la normativa cambia; los artículos aquí son punto de partida, no dogma). Fuente normativa:
Ley 37/1992 (LIVA), RD 1624/1992 (RIVA), y las páginas oficiales AEAT
(`sede.agenciatributaria.gob.es`). Fuente de implementación: Odoo `l10n_es*` (vía
`references/shared/odoo-source.md`).

## Base imponible
- **Regla general**: base = contraprestación (art. 78 LIVA). Descuentos y suplidos según art. 78.Tres.
- **Régimen especial de bienes usados (REBU)**: base = margen (art. 137 LIVA).
- **Régimen especial agencias de viajes (REAV)**: base = margen bruto (art. 145 LIVA);
  fórmula AEAT `BI = (P.vta − coste)/1,21`. **Invariante de total**: el total facturado al
  cliente = precio de venta; la cuota va COMPRENDIDA en el precio (art. 142), NO se añade
  encima. (Caso real: inyectar la base del margen sobre líneas a precio de venta infla el total.)
- **Margen ≤ 0** → base = 0 (no negativa).

## Cuota
- **Tipo**: general 21 %, reducido 10 %, superreducido 4 % (art. 90-91 LIVA). Los regímenes de
  margen no fijan tipo propio: aplican el general salvo excepción.
- **Incluido vs excluido**: si el precio lleva IVA incluido, cuota = importe − importe/(1+tipo).
  En Odoo, `price_include_override='tax_included'` o `amount_type='division'`.
- **Redondeo**: por documento vs por línea según config; verificar contra el criterio AEAT
  aplicable y contra cómo lo hace el estándar. No asumir.

## Deducibilidad del soportado
- **No deducible** el IVA de adquisiciones que benefician directamente al viajero en REAV
  (art. 146 LIVA); marcar el impuesto de compra como `no_deducible`.
- ISP / inversión del sujeto pasivo (art. 84): el destinatario autorrepercute; base y cuota
  con repartición correcta, sin repercutir al proveedor.
- Recargo de equivalencia (art. 148-163): recargo adicional al IVA en ventas a minoristas RE.

## No desglose / mención (enlaza con invoice-edi-invariants.md)
- **No desglose de cuota** cuando el régimen lo impone (art. 142 LIVA para REAV): la cuota no
  se consigna por separado, se entiende incluida en el precio.
- Mención obligatoria del régimen en factura: **RD 1619/2012, art. 6.1** (no el art. de la LIVA
  que regula el no desglose — son cosas distintas).

## Territorio
- Península/Baleares: IVA (plan `es_common_mainland`). Canarias: IGIC. Ceuta/Melilla: IPSI.
  El impuesto y la posición fiscal deben colgar del plan correcto.

## Cómo verificar (resumen)
Para cada invariante: (1) confirmar el artículo/FAQ AEAT con `WebFetch`; (2) leer cómo lo
implementa el **estándar** en fuente Odoo — ése es el valor legalmente correcto de la base que el
módulo NO debe alterar; (3) diferencial A/B: la operación **no-objetivo** (régimen general) debe
dar `amount_untaxed`/`amount_tax`/`amount_total`/base/cuota **idénticos** con y sin el módulo; la
operación **objetivo** debe desviarse solo lo que su artículo permite. Ver `runtime-verification.md`.
