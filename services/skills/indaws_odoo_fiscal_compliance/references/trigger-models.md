# Modelos y hooks Odoo cuyo cambio es fiscal-sensible (define el scope del diferencial)

Si un desarrollo **extiende, computa, mapea, sobreescribe o carga por `@template`** cualquiera
de estos modelos/campos/hooks, **altera un cálculo legalmente sensible** → hay que medir su
**radio de impacto** con el diferencial A/B (Step 1 = qué cómputo altera y cuál es su subconjunto
objetivo; el resto es la batería no-objetivo que debe quedar intacta). La regla operativa: *tocar
el importe, la base, la cuota, el tipo, el régimen, la fecha fiscal, la numeración o el
registro/declaración de una operación ⇒ verificar que el cambio no se filtra fuera de su objetivo*.

## 1. Impuestos y base/cuota (núcleo)
| Modelo / campo | Por qué dispara | Qué revisar |
|---|---|---|
| `account.tax` (`amount`, `amount_type`, `price_include(_override)`, `l10n_es_type`, `l10n_es_applicability`) | Define tipo, base y si va incluido | Tipo correcto, incluido vs excluido, aplicabilidad EDI |
| `account.tax.repartition.line` | Reparto de cuota / no deducible / retenciones | Signo, cuentas, % repartición, deducibilidad |
| `account.tax.group` | Agrupa/ordena cuotas en el total | Agrupación correcta por tipo |
| `account.fiscal.position` + `map_tax` / `map_account` / registration keys | Sustituye impuestos y **fija el régimen** | Mapeo, clave de régimen (SII/VF), `auto_apply` |
| `account.chart.template` + `@template(... "account.tax"/"account.fiscal.position")` | Carga/recarga de impuestos y posiciones del plan | Datos correctos, plan correcto (mainland vs canary) |

## 2. Factura / apunte (documento)
| Modelo / campo | Por qué dispara | Qué revisar |
|---|---|---|
| `account.move` (`amount_total`, `amount_untaxed`, `amount_tax`, `narration`, `invoice_date`, `date`, `l10n_es_*`) | Es el documento fiscal | Invariante de total, base, fecha devengo/expedición, menciones |
| `account.move.line` (`price_unit`, `tax_ids`, `discount`, `display_type`, `tax_base_amount`) | Origen de base y cuota | Base por línea, filtro `display_type=='product'` — **en v16-v19, no solo v19**. v19 añade `line_subsection` y tres `non_deductible_*` a la selection |
| `account.move.reversal` / rectificativas | Abonos y correcciones | Tipo rectificativa, referencia, signo |
| `account.journal` (secuencia/serie) | Numeración legal | Series, correlatividad, no huecos |
| `account.payment.term` | Devengo/recargo, pagos fraccionados | Fechas de devengo, recargo equivalencia |

## 3. EDI / declaraciones (España)
| Modelo / hook | Por qué dispara | Qué revisar |
|---|---|---|
| `l10n_es_edi_sii.*`, `account.edi.format`, `l10n_es_sii_registration_key(_sale/_purchase)`, `_l10n_es_get_regime_code` | Suministro Inmediato SII | `ClaveRegimenEspecialOTrascendencia`, una por documento, de dónde sale de verdad |
| `l10n_es_edi_verifactu.document`, `account.move` campos verifactu, `_l10n_es_edi_verifactu_*_clave_regimen*`, `_..._get_tax_applicability` | Veri*Factu | Clave régimen, aplicabilidad ('01' IVA/'03' IGIC), **cadena de hash / inmutabilidad tras registro** |
| `l10n_es_edi_facturae.*` | Facturae (AAPP) | Estructura, régimen, firma |
| Modelos AEAT `l10n_es_aeat_mod*` (303/347/349/390), `l10n_es.aeat.*` | Modelos de declaración/libros | Coherencia base/cuota con la factura, casillas |

## 4. Cabecera y producto (config que arrastra fiscalidad)
| Modelo / campo | Por qué dispara | Qué revisar |
|---|---|---|
| `res.company` (país fiscal, régimen, `l10n_es_edi_verifactu_*`, config fiscal) | Config que condiciona todo el cálculo | Régimen activo, obligatoriedad EDI |
| `product.template` / `product.product` (`taxes_id`, `supplier_taxes_id`) | Impuestos por defecto de la operación | Impuestos correctos, no deducible |
| `sale.order(.line)` / `purchase.order(.line)` cuando computan impuesto/posición/coste fiscal | Origen de la operación que se factura | Propagación correcta a la factura |

## 5. Señales de código (aunque el modelo parezca inocuo)
Revisar SIEMPRE si el desarrollo hace alguna de estas cosas:
- Override/compute de `price_unit`, `amount_tax`, `tax_base_amount`, `price_total`, o del cálculo de impuestos (`_compute_tax*`, `_prepare_*_line_for_taxes*`).
- Nuevo `@template` sobre `account.tax` o `account.fiscal.position`.
- Override de `_l10n_es_*`, `_get_regime_code`, `_l10n_es_edi_*`, o inyección/edición de líneas de factura o de `tax_line_id`.
- Lógica de **régimen especial**: margen (REBU art. 138 / REAV arts. 141-147), ISP (art. 84), IGIC/IPSI, recargo de equivalencia, exenciones (arts. 20/21/25), criterio de caja.
- Redondeo de importes/cuotas, importes negativos (abonos), o "total incluido vs añadido".
- Cualquier cosa que afecte a **fecha de operación/devengo**, numeración de factura, o al **registro inmutable** (Veri*Factu).

> Regla de oro: si al leer el diff aparece un impuesto, una posición fiscal, una clave de
> régimen, una base, una cuota, una mención de factura o un modelo AEAT — **para y revisa
> cumplimiento**. Instalar limpio y tener tests verdes NO exime (ver `runtime-verification.md`).
